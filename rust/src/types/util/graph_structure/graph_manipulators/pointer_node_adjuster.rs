use std::{
    collections::{HashMap, HashSet},
    iter::repeat,
    marker::PhantomData,
    ops::Deref,
};

use itertools::{Either, Itertools};
use oxidd::NodeID;

use crate::{
    types::util::{
        graph_structure::graph_structure::{
            Change, DrawTag, EdgeType, GraphEventsReader, GraphEventsWriter, GraphStructure,
        },
        storage::state_storage::StateStorage,
    },
    util::{free_id_manager::FreeIdManager, logging::console},
};

/// The LabelNodeAdjuster inserts new nodes with some label text to be used as pointers, according to pointer labels provided for each node
///
// We distinguish 2 different nodeID kinds:
// - source node IDs, corresponding to the ID of the underlying graph(s)
// - output node IDs, corresponding to the IDs used to interface with this graph
//
// The source node IDs are distinguished into 2 labeled kinds:
// - left node IDs, corresponding to the underlying graph we are wrapping
// - right node IDs, corresponding to the created pointer nodes
pub struct PointerNodeAdjuster<
    T: DrawTag + 'static,
    NL: Clone,
    LL: Clone,
    G: GraphStructure<T, NL, LL>,
> {
    graph: G,
    event_writer: GraphEventsWriter,
    graph_events: GraphEventsReader,
    node_label: PhantomData<NL>,
    level_label: PhantomData<LL>,

    pointer_edge: EdgeType<T>,
    transfer_root_pointers: bool,
    dummy_level_label: LL,

    pointers_of: HashMap<NodeID, HashSet<NodeID>>, // Maps left nodes to right nodes
    pointers: HashMap<NodeID, PointerNode>,        // Maps right nodes to their pointer data
    free_id: FreeIdManager<usize>,
}

pub trait WithPointerLabels {
    fn get_pointer_labels(&self) -> Vec<String>;
}

type SourcedNodeID = Either<NodeID, NodeID>;
fn to_sourced(id: NodeID) -> SourcedNodeID {
    if id % 2 == 0 {
        Either::Left(id / 2)
    } else {
        Either::Right(id / 2)
    }
}
fn from_sourced(id: SourcedNodeID) -> NodeID {
    match id {
        Either::Left(id) => id * 2,
        Either::Right(id) => id * 2 + 1,
    }
}

#[derive(Clone)]
pub struct PointerNode {
    text: String,
    pointer_for: NodeID,
}

#[derive(Clone)]
pub enum PointerLabel<NL: Clone> {
    Node(NL),
    Pointer(String),
}

impl<NL: Clone> Into<Option<String>> for PointerLabel<NL> {
    fn into(self) -> Option<String> {
        match self {
            PointerLabel::Node(_) => None,
            PointerLabel::Pointer(text) => Some(text),
        }
    }
}

impl<
        T: DrawTag + 'static,
        NL: Clone + WithPointerLabels,
        LL: Clone,
        G: GraphStructure<T, NL, LL>,
    > PointerNodeAdjuster<T, NL, LL, G>
{
    /// Creates a new pointer node adjuster, which creates dedicated nodes from the given point labels.
    ///
    /// `pointer_edge` indicates the edge type that should be used for pointer edges
    /// `transfer_root_pointers` indicates whether roots with pointers should be removed, and their points should become the roots instead
    /// `dummy_level_label` indicates the label to use for the newly inserted level (inserted to make space for an initial label)
    pub fn new(
        mut graph: G,
        pointer_edge: EdgeType<T>,
        transfer_root_pointers: bool,
        dummy_level_label: LL,
    ) -> PointerNodeAdjuster<T, NL, LL, G> {
        let mut adjuster = PointerNodeAdjuster {
            graph_events: graph.create_event_reader(),
            graph,
            event_writer: GraphEventsWriter::new(),
            node_label: PhantomData,
            level_label: PhantomData,
            pointer_edge,
            transfer_root_pointers,
            dummy_level_label,
            pointers_of: HashMap::new(),
            pointers: HashMap::new(),
            free_id: FreeIdManager::new(0),
        };
        for node in adjuster.graph.get_roots() {
            adjuster.add_labels(node);
        }
        adjuster
    }

    fn process_graph_changes(&mut self) {
        let events = self.graph.consume_events(&self.graph_events).clone();
        for event in events {
            match event {
                Change::NodeLabelChange { node } => {
                    let pointer_ids = self
                        .pointers_of
                        .get(&node)
                        .cloned()
                        .unwrap_or_else(|| HashSet::new());
                    let pointers = pointer_ids
                        .iter()
                        .map(|&id| (id, self.pointers.get(&id).unwrap().clone()));
                    let pointer_texts = self.graph.get_node_label(node).get_pointer_labels();
                    let new_pointer_texts = pointer_texts
                        .iter()
                        .cloned()
                        .filter(|text| !pointers.clone().any(|(_, pointer)| pointer.text == *text))
                        .collect_vec();
                    let removed_pointers = pointers
                        .filter(|(_, pointer)| {
                            !pointer_texts.iter().any(|text| pointer.text == *text)
                        })
                        .collect_vec();

                    for text in new_pointer_texts {
                        self.add_pointer(node, text);
                    }
                    for (id, _) in removed_pointers {
                        self.remove_pointer(id);
                    }
                    self.event_writer.write(Change::NodeLabelChange {
                        node: from_sourced(Either::Left(node)),
                    });
                }
                Change::LevelChange { node } => {
                    self.event_writer.write(Change::LevelChange {
                        node: from_sourced(Either::Left(node)),
                    });
                    if let Some(pointers) = self.pointers_of.get(&node) {
                        for &id in pointers {
                            self.event_writer.write(Change::LevelChange {
                                node: from_sourced(Either::Right(id)),
                            });
                        }
                    }
                }
                Change::LevelLabelChange { level } => {
                    self.event_writer.write(Change::LevelLabelChange { level });
                }
                Change::NodeConnectionsChange { node } => {
                    self.event_writer.write(Change::NodeConnectionsChange {
                        node: from_sourced(Either::Left(node)),
                    });
                }
                Change::ParentDiscover { child } => {
                    self.event_writer.write(Change::ParentDiscover {
                        child: from_sourced(Either::Left(child)),
                    });
                }
                Change::NodeRemoval { node } => {
                    self.event_writer.write(Change::NodeRemoval {
                        node: from_sourced(Either::Left(node)),
                    });
                    if let Some(pointers) = self.pointers_of.get(&node) {
                        for id in pointers.clone().iter().cloned() {
                            self.remove_pointer(id);
                        }
                    }
                }
                Change::NodeInsertion { node, source } => {
                    self.event_writer.write(Change::NodeInsertion {
                        node: from_sourced(Either::Left(node)),
                        source: source.map(|s| from_sourced(Either::Left(s))),
                    });
                    self.add_labels(node);
                }
            }
        }
    }

    fn add_labels(&mut self, node: NodeID) {
        let pointer_texts = self.graph.get_node_label(node).get_pointer_labels();
        for text in pointer_texts {
            self.add_pointer(node, text);
        }
    }

    fn add_pointer(&mut self, to: NodeID, text: String) {
        let pointer = PointerNode {
            pointer_for: to,
            text,
        };
        let id = self.free_id.get_next();

        self.pointers.insert(id, pointer);
        self.pointers_of
            .entry(to)
            .or_insert_with(|| HashSet::new())
            .insert(id);

        self.event_writer.write(Change::NodeInsertion {
            node: from_sourced(Either::Right(id)),
            source: None,
        });
        self.event_writer.write(Change::ParentDiscover {
            child: from_sourced(Either::Left(to)),
        });
    }

    fn remove_pointer(&mut self, id: NodeID) {
        let Some(pointer) = self.pointers.get(&id) else {
            return;
        };
        let target = pointer.pointer_for;
        self.pointers.remove(&id);
        self.free_id.make_available(id);

        let pointers_of_target = self.pointers_of.get_mut(&target).unwrap();
        pointers_of_target.remove(&id);
        if pointers_of_target.len() == 0 {
            self.pointers_of.remove(&id);
        }

        self.event_writer.write(Change::NodeRemoval {
            node: from_sourced(Either::Right(id)),
        });
    }
}

impl<
        T: DrawTag + 'static,
        NL: Clone + WithPointerLabels,
        LL: Clone,
        G: GraphStructure<T, NL, LL>,
    > GraphStructure<T, PointerLabel<NL>, LL> for PointerNodeAdjuster<T, NL, LL, G>
{
    fn get_roots(&self) -> Vec<crate::wasm_interface::NodeID> {
        if self.transfer_root_pointers {
            let p = self
                .graph
                .get_roots()
                .iter()
                .flat_map(|&node| match self.pointers_of.get(&node) {
                    Some(labels) => labels
                        .iter()
                        .map(|&p| from_sourced(Either::Right(p)))
                        .collect_vec(),
                    None => vec![from_sourced(Either::Left(node))],
                })
                .collect_vec();
            console::log!("roots: {}", p.iter().join(", "));
            p
        } else {
            self.graph
                .get_roots()
                .iter()
                .map(|&node| from_sourced(Either::Left(node)))
                .collect()
        }
    }

    fn get_terminals(&self) -> Vec<crate::wasm_interface::NodeID> {
        self.graph
            .get_terminals()
            .iter()
            .map(|&node| from_sourced(Either::Left(node)))
            .collect()
    }

    fn get_known_parents(
        &mut self,
        node: crate::wasm_interface::NodeID,
    ) -> Vec<(
        crate::types::util::graph_structure::graph_structure::EdgeType<T>,
        crate::wasm_interface::NodeID,
    )> {
        self.process_graph_changes();
        match to_sourced(node) {
            Either::Left(node) => {
                let or_parents = self
                    .graph
                    .get_known_parents(node)
                    .into_iter()
                    .map(|(edge, node)| (edge, from_sourced(Either::Left(node))));
                match self.pointers_of.get(&node) {
                    Some(pointers) => or_parents
                        .chain(
                            repeat(self.pointer_edge)
                                .zip(pointers.iter().map(|&p| from_sourced(Either::Right(p)))),
                        )
                        .collect(),
                    None => or_parents.collect(),
                }
            }
            Either::Right(_) => vec![],
        }
    }

    fn get_children(
        &mut self,
        node: crate::wasm_interface::NodeID,
    ) -> Vec<(
        crate::types::util::graph_structure::graph_structure::EdgeType<T>,
        crate::wasm_interface::NodeID,
    )> {
        self.process_graph_changes();
        match to_sourced(node) {
            Either::Left(node) => self
                .graph
                .get_children(node)
                .into_iter()
                .map(|(edge, node)| (edge, from_sourced(Either::Left(node))))
                .collect(),
            Either::Right(node) => match self.pointers.get(&node) {
                Some(pointer) => vec![(
                    self.pointer_edge,
                    from_sourced(Either::Left(pointer.pointer_for)),
                )],
                None => vec![],
            },
        }
    }

    fn get_level(&mut self, node: crate::wasm_interface::NodeID) -> oxidd::LevelNo {
        match to_sourced(node) {
            Either::Left(node) => self.graph.get_level(node) + 1,
            Either::Right(node) => match self.pointers.get(&node) {
                Some(pointer) => self.graph.get_level(pointer.pointer_for),
                None => 0,
            },
        }
    }

    fn get_node_label(&self, node: crate::wasm_interface::NodeID) -> PointerLabel<NL> {
        match to_sourced(node) {
            Either::Left(node) => PointerLabel::Node(self.graph.get_node_label(node)),
            Either::Right(node) => match self.pointers.get(&node) {
                Some(pointer) => PointerLabel::Pointer(pointer.text.clone()),
                None => PointerLabel::Pointer("".to_string()),
            },
        }
    }

    fn get_level_label(&self, level: oxidd::LevelNo) -> LL {
        if level > 0 {
            self.graph.get_level_label(level - 1)
        } else {
            self.dummy_level_label.clone()
        }
    }

    fn create_event_reader(
        &mut self,
    ) -> crate::types::util::graph_structure::graph_structure::GraphEventsReader {
        self.event_writer.create_reader()
    }

    fn consume_events(
        &mut self,
        reader: &crate::types::util::graph_structure::graph_structure::GraphEventsReader,
    ) -> Vec<crate::types::util::graph_structure::graph_structure::Change> {
        self.process_graph_changes();
        self.event_writer.read(reader)
    }

    fn local_nodes_to_sources(
        &self,
        nodes: Vec<crate::wasm_interface::NodeID>,
    ) -> Vec<crate::wasm_interface::NodeID> {
        nodes
            .into_iter()
            .filter_map(|node| match to_sourced(node) {
                Either::Left(node) => Some(node),
                Either::Right(node) => self.pointers.get(&node).map(|pointer| pointer.pointer_for),
            })
            .collect()
    }

    fn source_nodes_to_local(
        &self,
        nodes: Vec<crate::wasm_interface::NodeID>,
    ) -> Vec<crate::wasm_interface::NodeID> {
        // For each node, map it to the output format and add its pointers
        nodes
            .into_iter()
            .flat_map(|node| {
                Some(from_sourced(Either::Left(node))).into_iter().chain(
                    self.pointers_of
                        .get(&node)
                        .iter()
                        .flat_map(|pointers| {
                            pointers
                                .iter()
                                .map(|&pointer| from_sourced(Either::Right(pointer)))
                        })
                        .collect_vec()
                        .into_iter(),
                )
            })
            .collect()
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL> + StateStorage> StateStorage
    for PointerNodeAdjuster<T, NL, LL, G>
{
    fn read(&mut self, stream: &mut std::io::Cursor<&Vec<u8>>) -> std::io::Result<()> {
        self.graph.read(stream)
    }
    fn write(&self, stream: &mut std::io::Cursor<&mut Vec<u8>>) -> std::io::Result<()> {
        self.graph.write(stream)
    }
}
