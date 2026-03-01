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
    wasm_interface::NodeID,
};

pub struct EdgeToAdjuster<G: GraphStructure> {
    graph: G,
    remove_edges: HashSet<(NodeID, EdgeType<G::T>)>,

    event_writer: GraphEventsWriter,
    graph_events: GraphEventsReader,
}

impl<G: GraphStructure> EdgeToAdjuster<G> {
    pub fn new(mut graph: G) -> Self {
        EdgeToAdjuster {
            graph_events: graph.create_event_reader(),
            graph,
            event_writer: GraphEventsWriter::new(),
            remove_edges: HashSet::new(),
        }
    }

    pub fn set_remove_to_edges(
        &mut self,
        edges: impl Iterator<Item = (NodeID, EdgeType<G::T>)>,
    ) -> () {
        self.process_graph_changes();
        let remove_edges = self.remove_edges.clone();
        for node in self.get_affected_nodes(remove_edges) {
            self.event_writer
                .write(Change::NodeConnectionsChange { node: node });
        }

        self.remove_edges = edges.collect();
        let remove_edges = self.remove_edges.clone();
        for node in self.get_affected_nodes(remove_edges) {
            self.event_writer
                .write(Change::NodeConnectionsChange { node: node });
        }
    }

    fn get_affected_nodes(
        &mut self,
        remove_edges: HashSet<(NodeID, EdgeType<G::T>)>,
    ) -> HashSet<NodeID> {
        let affected_parents = remove_edges.iter().flat_map(|(to, edge_type)| {
            self.graph
                .get_known_parents(*to)
                .into_iter()
                .filter(|(from_edge_type, _from)| from_edge_type == edge_type)
                .map(|(_, from)| from)
                .collect_vec()
                .into_iter()
        });
        let affected_children = remove_edges.iter().map(|&(to, _)| to);
        let affected = affected_parents
            .chain(affected_children)
            .collect::<HashSet<_>>();
        affected
    }

    fn process_graph_changes(&mut self) {
        let events = self.graph.consume_events(&self.graph_events);
        for event in events {
            match event {
                _ => self.event_writer.write(event),
            }
        }
    }
}

impl<G: GraphStructure> GraphStructure for EdgeToAdjuster<G> {
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
        self.process_graph_changes();
        let parents = self.graph.get_known_parents(node);
        if self.remove_edges.len() == 0 {
            return parents;
        }

        parents
            .into_iter()
            .filter(|(e, _)| !self.remove_edges.contains(&(node, e.clone())))
            .collect()
    }

    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<G::T>, NodeID)> {
        self.process_graph_changes();
        let children = self.graph.get_children(node);
        if self.remove_edges.len() == 0 {
            return children;
        }

        children
            .into_iter()
            .filter(|(e, to)| !self.remove_edges.contains(&(*to, e.clone())))
            .collect()
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

impl<G: GraphStructure + StateStorage> StateStorage for EdgeToAdjuster<G>
where
    G::T: Serializable,
{
    fn write(&self, stream: &mut std::io::Cursor<&mut Vec<u8>>) -> std::io::Result<()> {
        self.graph.write(stream)?;

        let count = self.remove_edges.len();
        stream.write_u32::<LittleEndian>(count as u32)?;
        for (node_id, edge) in &self.remove_edges {
            stream.write_u32::<LittleEndian>(*node_id as u32)?;
            stream.write_i32::<LittleEndian>(edge.index)?;
            edge.tag.serialize(stream)?;
        }
        Ok(())
    }
    fn read(&mut self, stream: &mut std::io::Cursor<&Vec<u8>>) -> std::io::Result<()> {
        self.graph.read(stream)?;

        let count = stream.read_u32::<LittleEndian>()?;
        let mut remove_edges = HashSet::new();
        for _ in 0..count {
            let to = stream.read_u32::<LittleEndian>()? as usize;
            let index = stream.read_i32::<LittleEndian>()?;
            let tag = G::T::deserialize(stream)?;
            remove_edges.insert((to, EdgeType { tag, index }));
        }
        self.remove_edges = remove_edges;

        Ok(())
    }
}
