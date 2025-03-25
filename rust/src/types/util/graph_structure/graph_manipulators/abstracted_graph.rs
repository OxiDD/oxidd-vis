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

// A graph wrapper that abstracts over exact typeing
pub struct AbstractedGraph<T: DrawTag, NL: Clone, LL: Clone> {
    graph: Box<dyn StateGraphStructure<T, NL, LL>>,
    tag: PhantomData<T>,
    node_label: PhantomData<NL>,
    level_label: PhantomData<LL>,
}

trait StateGraphStructure<T: DrawTag, NL: Clone, LL: Clone>:
    GraphStructure<T, NL, LL> + StateStorage
{
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL> + StateStorage>
    StateGraphStructure<T, NL, LL> for G
{
}

impl<T: DrawTag, NL: Clone, LL: Clone> AbstractedGraph<T, NL, LL> {
    pub fn new<G: GraphStructure<T, NL, LL> + StateStorage + 'static>(
        graph: G,
    ) -> AbstractedGraph<T, NL, LL> {
        AbstractedGraph {
            graph: Box::new(graph),
            tag: PhantomData,
            node_label: PhantomData,
            level_label: PhantomData,
        }
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone> StateStorage for AbstractedGraph<T, NL, LL> {
    fn read(&mut self, stream: &mut std::io::Cursor<&Vec<u8>>) -> std::io::Result<()> {
        self.graph.read(stream)
    }
    fn write(&self, stream: &mut std::io::Cursor<&mut Vec<u8>>) -> std::io::Result<()> {
        self.graph.write(stream)
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone> GraphStructure<T, NL, LL> for AbstractedGraph<T, NL, LL> {
    fn get_roots(&self) -> Vec<NodeID> {
        self.graph.get_roots()
    }

    fn get_terminals(&self) -> Vec<NodeID> {
        self.graph.get_terminals()
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)> {
        self.graph.get_known_parents(node)
    }

    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)> {
        self.graph.get_children(node)
    }

    fn get_level(&mut self, node: NodeID) -> LevelNo {
        self.graph.get_level(node)
    }

    fn get_node_label(&self, node: NodeID) -> NL {
        self.graph.get_node_label(node)
    }

    fn get_level_label(&self, level: LevelNo) -> LL {
        self.graph.get_level_label(level)
    }

    fn create_event_reader(&mut self) -> GraphEventsReader {
        self.graph.create_event_reader()
    }

    fn consume_events(&mut self, reader: &GraphEventsReader) -> Vec<Change> {
        self.graph.consume_events(reader)
    }

    fn local_nodes_to_sources(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.local_nodes_to_sources(nodes)
    }

    fn source_nodes_to_local(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.source_nodes_to_local(nodes)
    }
}
