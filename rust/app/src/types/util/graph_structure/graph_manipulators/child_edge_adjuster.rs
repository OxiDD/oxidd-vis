use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use itertools::Itertools;

use crate::{
    types::util::{
        graph_structure::graph_structure::{
            Change, DrawTag, EdgeType, GraphEventsReader, GraphEventsWriter, GraphStructure,
        },
        storage::state_storage::{Serializable, StateStorage},
    },
    util::logging::console,
    wasm_interface::NodeID,
};

///
/// Can remove edges to children, or swap which edge goes to which child. Is not allowed to add new children.
pub struct ChildEdgeAdjuster<G: GraphStructure> {
    graph: G,

    adjuster: fn(Vec<(EdgeType<G::T>, NodeID, G::NL)>) -> Option<Vec<(EdgeType<G::T>, NodeID)>>,
    enabled: bool,
    event_writer: GraphEventsWriter,
    graph_events: GraphEventsReader,
    replacement_cache: HashMap<NodeID, Option<Vec<(EdgeType<G::T>, NodeID)>>>,
}

impl<G: GraphStructure> ChildEdgeAdjuster<G> {
    pub fn new(
        mut graph: G,
        replacer: fn(Vec<(EdgeType<G::T>, NodeID, G::NL)>) -> Option<Vec<(EdgeType<G::T>, NodeID)>>,
    ) -> Self {
        ChildEdgeAdjuster {
            graph_events: graph.create_event_reader(),
            graph,

            adjuster: replacer,
            enabled: true,
            event_writer: GraphEventsWriter::new(),
            replacement_cache: HashMap::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;

        let mut node_queue = self.graph.get_roots();
        let mut seen = HashSet::new();
        self.process_graph_changes();
        while let Some(node) = node_queue.pop() {
            if seen.contains(&node) {
                continue;
            }
            seen.insert(node);

            let children = self.graph.get_children(node);
            for &(_, neighbor) in self
                .graph
                .get_known_parents(node)
                .iter()
                .chain(children.iter())
            {
                node_queue.push(neighbor);
            }

            let matches = self.get_children_replacement(node);
            if let Some(replacement) = matches {
                self.event_writer
                    .write(Change::NodeConnectionsChange { node: node });
                for &child in children
                    .iter()
                    .map(|(_, child)| child)
                    .chain(replacement.iter().map(|(_, child)| child))
                    .sorted()
                    .dedup()
                {
                    self.event_writer
                        .write(Change::NodeConnectionsChange { node: child });
                }
            }
        }
    }

    fn get_children_replacement(&mut self, node: NodeID) -> Option<Vec<(EdgeType<G::T>, NodeID)>> {
        if let Some(hit) = self.replacement_cache.get(&node) {
            return hit.clone();
        }
        let children = self.graph.get_children(node);
        let result = (self.adjuster)(
            children
                .into_iter()
                .map(|(edge, child)| (edge, child, self.graph.get_node_label(child)))
                .collect(),
        );
        self.replacement_cache.insert(node, result.clone());
        return result;
    }

    fn process_graph_changes(&mut self) {
        let events = self.graph.consume_events(&self.graph_events);
        for event in events {
            match event {
                Change::NodeConnectionsChange { node } => {
                    self.replacement_cache.remove(&node);
                }
                Change::NodeInsertion { node, source: _ } => {
                    self.replacement_cache.remove(&node);
                }
                Change::NodeRemoval { node } => {
                    self.replacement_cache.remove(&node);
                }
                _ => {}
            };
            self.event_writer.write(event);
        }
    }
}

impl<G: GraphStructure> GraphStructure for ChildEdgeAdjuster<G> {
    type T = G::T;
    type NL = G::NL;
    type LL = G::LL;

    fn get_roots(&self) -> Vec<NodeID> {
        self.graph.get_roots()
    }

    fn get_terminals(&self) -> Vec<NodeID> {
        self.graph.get_terminals()
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<G::T>, NodeID)> {
        let parents = self.graph.get_known_parents(node);
        if !self.enabled {
            return parents;
        }
        self.process_graph_changes();

        // Check if any of the parents of this node have made any replacements, otherwise simply return the parents
        let (replacement_parent_nodes, regular_parent_nodes): (Vec<_>, Vec<_>) = parents
            .into_iter()
            .partition(|&(_, parent)| self.get_children_replacement(parent).is_some());
        if replacement_parent_nodes.len() == 0 {
            return regular_parent_nodes;
        }
        console::log!("Get known parents");

        // Filter out the parents that made replacements from the original results, and add newly calculated edges based on the child edges of the parents that made replacements
        regular_parent_nodes
            .into_iter()
            .chain(
                replacement_parent_nodes
                    .iter()
                    .sorted_by_key(|(_, id)| id)
                    .dedup()
                    .flat_map(|&(_, replacement_parent)| {
                        let children = self.get_children_replacement(replacement_parent).unwrap();
                        children.into_iter().filter_map(move |(edge, child)| {
                            if child == node {
                                Some((edge, replacement_parent))
                            } else {
                                None
                            }
                        })
                    }),
            )
            .collect()
    }

    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<G::T>, NodeID)> {
        let children = self.graph.get_children(node);
        if !self.enabled {
            return children;
        }
        self.process_graph_changes();

        self.get_children_replacement(node).unwrap_or(children)
    }

    fn get_level(&mut self, node: NodeID) -> oxidd::LevelNo {
        self.graph.get_level(node)
    }

    fn get_node_label(&self, node: NodeID) -> G::NL {
        self.graph.get_node_label(node)
    }

    fn get_level_label(&self, level: oxidd::LevelNo) -> G::LL {
        self.graph.get_level_label(level)
    }

    fn create_event_reader(&mut self) -> GraphEventsReader {
        self.event_writer.create_reader()
    }

    fn consume_events(
        &mut self,
        reader: &GraphEventsReader,
    ) -> Vec<crate::types::util::graph_structure::graph_structure::Change> {
        self.process_graph_changes();
        self.event_writer.read(reader)
    }

    fn local_nodes_to_sources(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.local_nodes_to_sources(nodes)
    }

    fn source_nodes_to_local(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.source_nodes_to_local(nodes)
    }
}

impl<G: GraphStructure + StateStorage> StateStorage for ChildEdgeAdjuster<G>
where
    G::T: Serializable,
{
    fn write(&self, stream: &mut std::io::Cursor<&mut Vec<u8>>) -> std::io::Result<()> {
        self.graph.write(stream)?;

        stream.write_u8(self.enabled as u8)?;
        Ok(())
    }
    fn read(&mut self, stream: &mut std::io::Cursor<&Vec<u8>>) -> std::io::Result<()> {
        self.graph.read(stream)?;

        self.enabled = stream.read_u8()? > 0;
        Ok(())
    }
}
