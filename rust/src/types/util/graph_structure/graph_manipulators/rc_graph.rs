use std::{
    cell::{Ref, RefMut},
    marker::PhantomData,
    ops::Deref,
};

use oxidd::LevelNo;

use crate::{
    types::util::{
        graph_structure::graph_structure::{
            Change, DrawTag, EdgeType, GraphEventsReader, GraphStructure,
        },
        storage::state_storage::StateStorage,
    },
    util::rc_refcell::MutRcRefCell,
    wasm_interface::NodeID,
};

// A cloneable graph such that multiple places can share ownership
pub struct RCGraph<G: GraphStructure> {
    graph: MutRcRefCell<G>,
}
impl<G: GraphStructure> Clone for RCGraph<G> {
    fn clone(&self) -> Self {
        Self {
            graph: self.graph.clone(),
        }
    }
}

impl<G: GraphStructure> RCGraph<G> {
    pub fn new(graph: G) -> RCGraph<G> {
        RCGraph {
            graph: MutRcRefCell::new(graph),
        }
    }

    pub fn read<'a>(&'a self) -> Ref<'a, G> {
        self.graph.read()
    }

    pub fn get<'a>(&'a self) -> RefMut<'a, G> {
        self.graph.get()
    }
}

impl<G: GraphStructure> StateStorage for RCGraph<G>
where
    G: StateStorage,
{
    fn read(&mut self, stream: &mut std::io::Cursor<&Vec<u8>>) -> std::io::Result<()> {
        self.graph.get().read(stream)
    }
    fn write(&self, stream: &mut std::io::Cursor<&mut Vec<u8>>) -> std::io::Result<()> {
        self.graph.read().write(stream)
    }
}

impl<G: GraphStructure> GraphStructure for RCGraph<G> {
    type T = G::T;
    type NL = G::NL;
    type LL = G::LL;
    fn get_roots(&self) -> Vec<NodeID> {
        self.graph.read().get_roots()
    }

    fn get_terminals(&self) -> Vec<NodeID> {
        self.graph.read().get_terminals()
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<Self::T>, NodeID)> {
        self.graph.get().get_known_parents(node)
    }

    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<Self::T>, NodeID)> {
        self.graph.get().get_children(node)
    }

    fn get_level(&mut self, node: NodeID) -> LevelNo {
        self.graph.get().get_level(node)
    }

    fn get_node_label(&self, node: NodeID) -> Self::NL {
        self.graph.read().get_node_label(node)
    }

    fn get_level_label(&self, level: LevelNo) -> Self::LL {
        self.graph.read().get_level_label(level)
    }

    fn create_event_reader(&mut self) -> GraphEventsReader {
        self.graph.get().create_event_reader()
    }

    fn consume_events(&mut self, reader: &GraphEventsReader) -> Vec<Change> {
        self.graph.get().consume_events(reader)
    }

    fn local_nodes_to_sources(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.read().local_nodes_to_sources(nodes)
    }

    fn source_nodes_to_local(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.read().source_nodes_to_local(nodes)
    }
}
