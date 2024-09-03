use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Ref, RefCell},
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
    rc::Rc,
};

use itertools::{Either, Itertools};
use multimap::MultiMap;
use oxidd::{LevelNo, NodeID};
use wasm_bindgen::prelude::*;

use crate::{
    types::util::graph_structure::graph_structure::{
        Change, DrawTag, EdgeType, GraphEventsReader, GraphEventsWriter, GraphStructure,
    },
    util::{free_id_manager::FreeIdManager, logging::console},
};

/// We distinguish 2 different nodeID kinds:
/// - source node IDs, corresponding to the ID of the underlying graph(s)
/// - output node IDs, corresponding to the IDs used to interface with this graph
///
/// The source node IDs are distinguished into 2 labeled kinds:
/// - left node IDs, corresponding to the underlying graph we are wrapping
/// - right node IDs, corresponding to the created virtual nodes
pub struct NodePresenceAdjuster<
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

    /*  All the adjustment data */
    adjustments: HashMap<NodeID, PresenceGroups<T>>, // Specifies the adjustments for the left source node ID
    sources: HashMap<NodeID, NodeID>, // Maps the right source nodeID to the corresponding left source node ID
    images: MultiMap<NodeID, NodeID>, // Maps the left source nodeID to all of the corresponding right source node IDs
    // node_group: HashMap<NodeID, PresenceGroup>, // Maps the left source nodeID to the presence group it represents
    replacements: HashMap<(NodeID, EdgeConstraint<T>, NodeID), NodeID>, // For a combination of parent output nodeID and a child left source nodeID, the replacement child right source nodeID
    parent_nodes: HashMap<NodeID, HashSet<NodeID>>, // The parent nodes (output node IDs) of a right source nodeID.
    known_parents: HashMap<NodeID, Vec<(EdgeType<T>, NodeID)>>, // The parents (output node IDs) and edge type of a right source nodeID. Note that these are the known parents, because we may for sure these are the only parents that can exist for the created node, but can not be sure these are the only edge types.
    children: HashMap<NodeID, Vec<(EdgeType<T>, NodeID)>>, // The children (output node IDs) and edge type of a output nodeID
    free_id: FreeIdManager<usize>,
}

#[derive(Eq, PartialEq, Clone)]
pub struct PresenceGroups<T: DrawTag> {
    // A set of "parent groups" where for every parent group a unique node is created, NodeID here refers to an output NodeID
    groups: Vec<Vec<(EdgeConstraint<T>, NodeID)>>,
    // The way to handle how the presence for any parent node in any of the above defined groups
    remainder: PresenceRemainder,
}
impl<T: DrawTag> PresenceGroups<T> {
    pub fn new(
        groups: Vec<Vec<(EdgeConstraint<T>, NodeID)>>,
        remainder: PresenceRemainder,
    ) -> PresenceGroups<T> {
        PresenceGroups { groups, remainder }
    }

    pub fn remainder(remainder: PresenceRemainder) -> PresenceGroups<T> {
        PresenceGroups::new(Vec::new(), remainder)
    }
}

// #[derive(Eq, PartialEq, Clone)]
// pub enum PresenceGroup {
//     SharedRemainder,      // The original graph's node is used
//     DuplicateRemainder,   // One node is created per discovered parent
//     Parents(Vec<NodeID>), // One node is created for the specified parents (output node IDs)
// }
// Default presence group if not specified: PresenceGroups(Vec::from([PresenceGroup::SharedRemainder]))

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum EdgeConstraint<T: DrawTag> {
    Exact(EdgeType<T>),
    Any,
}

#[wasm_bindgen]
#[derive(Eq, PartialEq, Clone)]
pub enum PresenceRemainder {
    // Show this unique terminal the regular way (default)
    Show,
    // Hide this terminal
    Hide,
    // Make a unique instance of every occurrence of this terminal
    Duplicate,
    // Make a unique instance of every parent this terminal (multiple edges from the same parent share a single duplication)
    DuplicateParent,
}

// Values on the right side should only be used for nodes that are being adjusted to be duplicated, everything else retains the left version of the ID
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

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>>
    NodePresenceAdjuster<T, NL, LL, G>
{
    pub fn new(mut graph: G) -> NodePresenceAdjuster<T, NL, LL, G> {
        NodePresenceAdjuster {
            graph_events: graph.create_event_reader(),
            graph,
            event_writer: GraphEventsWriter::new(),
            adjustments: HashMap::new(),
            sources: HashMap::new(),
            images: MultiMap::new(),
            replacements: HashMap::new(),
            parent_nodes: HashMap::new(),
            known_parents: HashMap::new(),
            children: HashMap::new(),
            free_id: FreeIdManager::new(0),
            level_label: PhantomData,
            node_label: PhantomData,
        }
    }

    pub fn set_node_presence(&mut self, out_node: NodeID, presence: PresenceGroups<T>) {
        let owner = self.get_owner_id(out_node);

        // Create events for removal of the old node (connections) and images
        let node_copies = self.get_all_copies(owner);
        for copy in node_copies {
            self.add_remove_node_events(copy);
        }

        // Delete the old images
        let maybe_images = self.images.get_vec(&owner).cloned();
        if let Some(images) = maybe_images {
            for image in images {
                self.delete_replacement(image);
            }
        }

        // Determine the new images of the node
        {
            self.adjustments.insert(owner, presence.clone());

            // This automatically creates events for the created replacements
            for group in presence.groups {
                self.create_replacement(group, owner);
            }

            // Make sure that for all possible parents, the children are determined (and hence replacements are calculated if needed)
            let source_parents = self.graph.get_known_parents(owner);
            let parents = source_parents
                .iter()
                .flat_map(|(_, parent)| self.get_all_copies(*parent))
                .collect_vec();
            for parent in parents {
                self.update_children(parent);
            }
        }

        // Create an event for the replaced node
        let owner_out = from_sourced(Either::Left(owner));
        if presence.remainder == PresenceRemainder::Show {
            self.add_insert_node_events(owner_out, owner_out);
        }
    }

    fn process_graph_changes(&mut self) {
        let events = self.graph.consume_events(&self.graph_events);

        for event in events {
            match event {
                Change::NodeLabelChange { node } => {
                    for node_copy in self.get_all_copies(node) {
                        self.event_writer
                            .write(Change::NodeLabelChange { node: node_copy });
                    }
                }
                Change::LevelChange { node } => {
                    for node_copy in self.get_all_copies(node) {
                        self.event_writer
                            .write(Change::LevelChange { node: node_copy });
                    }
                }
                Change::LevelLabelChange { level } => {
                    self.event_writer.write(Change::LevelLabelChange { level });
                }
                Change::NodeConnectionsChange { node } => {
                    for node_copy in self.get_all_copies(node) {
                        self.event_writer
                            .write(Change::NodeConnectionsChange { node: node_copy });

                        self.update_children(node_copy);
                        if let Either::Right(copy_id) = to_sourced(node_copy) {
                            self.update_parents(copy_id);
                        }
                    }
                }
                Change::NodeRemoval { node } => {
                    for node_copy in self.get_all_copies(node) {
                        self.event_writer
                            .write(Change::NodeRemoval { node: node_copy });
                        if let Either::Right(copy_id) = to_sourced(node_copy) {
                            self.delete_replacement(copy_id);
                        }
                    }
                }
                Change::NodeInsertion { node, source } => {
                    for node_copy in self.get_all_copies(node) {
                        self.event_writer.write(Change::NodeInsertion {
                            node: node_copy,
                            source,
                        });
                    }
                }
                Change::ParentDiscover { child } => {
                    self.event_writer.write(Change::ParentDiscover {
                        child: from_sourced(Either::Right(child)),
                    });
                }
            }
        }
    }

    fn add_neighbor_connection_change_events(&mut self, out_node: NodeID) {
        let parents = self.get_known_parents(out_node);
        let children = self.get_children(out_node);
        for (_edge, parent) in parents {
            self.event_writer
                .write(Change::NodeConnectionsChange { node: parent });
        }

        for (_edge, child) in children {
            self.event_writer
                .write(Change::NodeConnectionsChange { node: child });
        }
    }

    fn add_remove_node_events(&mut self, out_node: NodeID) {
        self.add_neighbor_connection_change_events(out_node);

        self.event_writer
            .write(Change::NodeRemoval { node: out_node });
    }

    fn add_insert_node_events(&mut self, out_node: NodeID, source: NodeID) {
        self.add_neighbor_connection_change_events(out_node);

        self.event_writer.write(Change::NodeInsertion {
            node: out_node,
            source: Some(source),
        });
    }

    fn get_owner_id(&self, id: NodeID) -> NodeID {
        match to_sourced(id) {
            Either::Left(id) => id,
            Either::Right(id) => {
                let Some(original_id) = self.sources.get(&id) else {
                    return 0; // Case should not be reachable
                };
                *original_id
            }
        }
    }

    fn create_replacement(
        &mut self,
        parents: Vec<(EdgeConstraint<T>, NodeID)>,
        child_to_be_replaced: NodeID,
    ) -> NodeID {
        let id = {
            let id = self.free_id.get_next();

            // Store the mapping
            self.sources.insert(id, child_to_be_replaced);
            self.images.insert(child_to_be_replaced, id);
            for (constraint, parent) in &parents {
                self.replacements
                    .insert((*parent, constraint.clone(), child_to_be_replaced), id);
            }

            // Store the parents
            self.parent_nodes
                .insert(id, parents.iter().map(|(_, parent)| *parent).collect());

            id
        };

        // Calculate the connections
        self.update_parents(id);
        let out_id = from_sourced(Either::Right(id));
        self.update_children(out_id);

        // Create a creation event
        self.add_insert_node_events(out_id, from_sourced(Either::Left(child_to_be_replaced)));

        id
    }

    fn delete_replacement(&mut self, node: NodeID) {
        let parents = self.get_known_parents(from_sourced(Either::Right(node)));
        let Some(&source) = self.sources.get(&node) else {
            return;
        };

        for (edge, parent) in parents {
            let r1 = self
                .replacements
                .remove(&(parent, EdgeConstraint::Exact(edge), source));
            let r2 = self
                .replacements
                .remove(&(parent, EdgeConstraint::Any, source));
        }

        self.sources.remove(&node);
        if let Some(images) = self.images.get_vec_mut(&source) {
            images.retain(|&e| e != node);
            if images.len() == 0 {
                self.images.remove(&source);
            }
        }
        self.children.remove(&node);
        self.parent_nodes.remove(&node);
        self.known_parents.remove(&node);
        self.free_id.make_available(node);
    }

    fn update_parents(&mut self, right_node_id: NodeID) {
        let source_id = self.get_owner_id(from_sourced(Either::Right(right_node_id)));

        let parent_images: MultiMap<NodeID, NodeID> = {
            let parent_nodes = self.parent_nodes.get(&right_node_id).unwrap();
            parent_nodes
                .iter()
                .map(|&parent| (self.get_owner_id(parent), parent))
                .sorted()
                .dedup()
                .collect()
        };

        let source_parents = self.graph.get_known_parents(source_id);
        let mut out_parents = Vec::new();
        for (edge, source_parent) in source_parents {
            let Some(parent_images) = parent_images.get_vec(&source_parent) else {
                continue;
            };
            for &parent in parent_images {
                if self
                    .replacements
                    .get(&(parent, EdgeConstraint::Exact(edge), source_id))
                    == Some(&right_node_id)
                    || self
                        .replacements
                        .get(&(parent, EdgeConstraint::Any, source_id))
                        == Some(&right_node_id)
                {
                    out_parents.push((edge, parent));
                }
            }
        }

        // console::log!(
        //     "update parents for {}: {}",
        //     right_node_id,
        //     out_parents.iter().map(|(_, k)| k.to_string()).join(", ")
        // );

        self.known_parents.insert(right_node_id, out_parents);
    }

    fn update_children(&mut self, out_node_id: NodeID) {
        let source_id = self.get_owner_id(out_node_id);

        // This is the only place that graph.get_children is called. Here we should also update our own "known_parents" accordingly
        let children = self.graph.get_children(source_id);

        let mut out = Vec::new();
        // Analyze the children and store them for future use
        for (edge_type, child) in children {
            let out_child = from_sourced(Either::Left(child));
            let remainder = {
                if let Some(&replacement) =
                    self.replacements
                        .get(&(out_node_id, EdgeConstraint::Exact(edge_type), child))
                {
                    self.update_parents(replacement);
                    out.push((edge_type, from_sourced(Either::Right(replacement))));
                    continue;
                }

                if let Some(&replacement) =
                    self.replacements
                        .get(&(out_node_id, EdgeConstraint::Any, child))
                {
                    self.update_parents(replacement);
                    out.push((edge_type, from_sourced(Either::Right(replacement))));
                    continue;
                }

                let Some(adjustment) = self.adjustments.get(&child) else {
                    out.push((edge_type, out_child));
                    continue;
                };
                adjustment.remainder.clone()
            };

            match remainder {
                PresenceRemainder::Show => out.push((edge_type, out_child)),
                PresenceRemainder::Hide => {}
                PresenceRemainder::Duplicate => out.push((
                    edge_type,
                    from_sourced(Either::Right(self.create_replacement(
                        Vec::from([(EdgeConstraint::Exact(edge_type), out_node_id)]),
                        child,
                    ))),
                )),
                PresenceRemainder::DuplicateParent => out.push((
                    edge_type,
                    from_sourced(Either::Right(self.create_replacement(
                        Vec::from([(EdgeConstraint::Any, out_node_id)]),
                        child,
                    ))),
                )),
            }
        }

        self.children.insert(out_node_id, out);
    }

    fn get_all_copies(&self, left_source_node: NodeID) -> Vec<NodeID> {
        let source_out = from_sourced(Either::Left(left_source_node));
        let maybe_images = self.images.get_vec(&left_source_node).cloned();
        if let Some(images) = maybe_images {
            let mut images = images
                .into_iter()
                .map(|image| from_sourced(Either::Right(image)))
                .collect_vec();
            images.push(source_out);
            images
        } else {
            vec![source_out]
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct PresenceLabel<LL> {
    pub original_label: LL,
    pub original_id: NodeID,
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>>
    GraphStructure<T, PresenceLabel<NL>, LL> for NodePresenceAdjuster<T, NL, LL, G>
{
    fn get_roots(&self) -> Vec<NodeID> {
        self.graph
            .get_roots()
            .iter()
            .map(|&root| from_sourced(Either::Left(root)))
            .collect()
    }
    fn get_terminals(&self) -> Vec<NodeID> {
        self.graph
            .get_terminals()
            .iter()
            .flat_map(|t| self.get_all_copies(*t))
            .collect()
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)> {
        self.process_graph_changes();
        let parents = match to_sourced(node) {
            Either::Left(id) => {
                let known_parents = self.graph.get_known_parents(id);
                // Filter parents to remove any parents that use a replacement node instead
                known_parents
                    .into_iter()
                    .map(|(edge, parent)| (edge, from_sourced(Either::Left(parent))))
                    .filter(|&(edge, out_parent)| {
                        let replaced = self.replacements.contains_key(&(
                            out_parent,
                            EdgeConstraint::Exact(edge.clone()),
                            id,
                        )) || self.replacements.contains_key(&(
                            out_parent,
                            EdgeConstraint::Any,
                            id,
                        ));
                        !replaced
                    })
                    .collect()
            }
            Either::Right(id) => self
                .known_parents
                .get(&id)
                .cloned()
                .unwrap_or_else(|| Vec::new()),
        };
        // console::log!(
        //     "{} parents: {}",
        //     node,
        //     parents.iter().map(|(_, v)| v.to_string()).join(", ")
        // );
        parents
    }

    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)> {
        self.process_graph_changes();
        if let Some(children) = self.children.get(&node) {
            // console::log!(
            //     "{} children: {}",
            //     node,
            //     children.iter().map(|(_, v)| v.to_string()).join(", ")
            // );
            // if let Either::Left(_) = to_sourced(node) {
            //     let data = (*self.node_data).borrow();
            //     console::log!(
            //         "{} children: {}",
            //         node,
            //         data.children
            //             .get(&node)
            //             .unwrap()
            //             .iter()
            //             .map(|(_, v)| v.to_string())
            //             .join(", ")
            //     );
            // }
            return children.clone();
        }

        match to_sourced(node) {
            Either::Left(_) => {
                self.update_children(node);

                // console::log!(
                //     "{} children: {}",
                //     node,
                //     data.children
                //         .get(&node)
                //         .unwrap()
                //         .iter()
                //         .map(|(_, v)| v.to_string())
                //         .join(", ")
                // );
                return self.children.get(&node).cloned().unwrap();
            }
            Either::Right(_) => {
                // This should not be able to happen, since any such node should have registered children
                return Vec::new();
            }
        }
    }

    fn get_level(&mut self, node: NodeID) -> LevelNo {
        let id = self.get_owner_id(node);
        // if self.graph.get_level(id) == 0 {
        //     console::log!(
        //         "node: {}, id: {}, level: {}",
        //         node,
        //         id,
        //         self.graph.get_level(id)
        //     );
        // }
        // if let Either::Right(_) = to_sourced(node) {
        //     console::log!(
        //         "node: {}, id: {}, level: {}",
        //         node,
        //         id,
        //         self.graph.get_level(id)
        //     );
        // }

        // TODO: store custom levels for terminals
        self.graph.get_level(id)
    }

    fn get_node_label(&self, node: NodeID) -> PresenceLabel<NL> {
        let id = self.get_owner_id(node);
        PresenceLabel {
            original_id: id,
            original_label: self.graph.get_node_label(id),
        }
    }

    fn get_level_label(&self, level: LevelNo) -> LL {
        self.graph.get_level_label(level)
    }

    fn create_event_reader(&mut self) -> GraphEventsReader {
        self.event_writer.create_reader()
    }
    fn consume_events(&mut self, reader: &GraphEventsReader) -> Vec<Change> {
        self.process_graph_changes();
        self.event_writer.read(reader)
    }

    fn local_nodes_to_sources(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.local_nodes_to_sources(
            nodes
                .into_iter()
                .map(|node| self.get_owner_id(node))
                .collect(),
        )
    }

    fn source_nodes_to_local(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph
            .source_nodes_to_local(nodes)
            .into_iter()
            .flat_map(|node| self.get_all_copies(node))
            .collect()
    }
}
