use std::{
    cell::{Ref, RefMut},
    marker::PhantomData,
    ops::Deref,
};

use oxidd::LevelNo;

use crate::{
    types::util::graph_structure::graph_structure::{
        Change, DrawTag, EdgeType, GraphEventsReader, GraphStructure,
    },
    util::rc_refcell::MutRcRefCell,
    wasm_interface::NodeID,
};

// A cloneable graph such that multiple places can share ownership
pub struct RCGraph<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> {
    graph: MutRcRefCell<G>,
    tag: PhantomData<T>,
    node_label: PhantomData<NL>,
    level_label: PhantomData<LL>,
}
impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> Clone
    for RCGraph<T, NL, LL, G>
{
    fn clone(&self) -> Self {
        Self {
            graph: self.graph.clone(),
            tag: self.tag.clone(),
            node_label: self.node_label.clone(),
            level_label: self.level_label.clone(),
        }
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> RCGraph<T, NL, LL, G> {
    pub fn new(graph: G) -> RCGraph<T, NL, LL, G> {
        RCGraph {
            graph: MutRcRefCell::new(graph),
            tag: PhantomData,
            node_label: PhantomData,
            level_label: PhantomData,
        }
    }

    pub fn read<'a>(&'a self) -> Ref<'a, G> {
        self.graph.read()
    }

    pub fn get<'a>(&'a self) -> RefMut<'a, G> {
        self.graph.get()
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> GraphStructure<T, NL, LL>
    for RCGraph<T, NL, LL, G>
{
    fn get_roots(&self) -> Vec<NodeID> {
        self.graph.read().get_roots()
    }

    fn get_terminals(&self) -> Vec<NodeID> {
        self.graph.read().get_terminals()
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)> {
        self.graph.get().get_known_parents(node)
    }

    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)> {
        self.graph.get().get_children(node)
    }

    fn get_level(&mut self, node: NodeID) -> LevelNo {
        self.graph.get().get_level(node)
    }

    fn get_node_label(&self, node: NodeID) -> NL {
        self.graph.read().get_node_label(node)
    }

    fn get_level_label(&self, level: LevelNo) -> LL {
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
