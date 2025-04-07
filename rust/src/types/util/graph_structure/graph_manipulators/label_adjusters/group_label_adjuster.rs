use std::{
    io::{Cursor, Result},
    marker::PhantomData,
    vec::IntoIter,
};

use oxidd::LevelNo;

use crate::{
    types::util::{
        graph_structure::{
            graph_structure::DrawTag,
            grouped_graph_structure::{EdgeCountData, GroupedGraphStructure},
        },
        storage::state_storage::StateStorage,
    },
    util::rc_refcell::MutRcRefCell,
    wasm_interface::{NodeGroupID, NodeID},
};

pub struct GroupLabelAdjuster<NGL, NLL, G: GroupedGraphStructure> {
    graph: MutRcRefCell<G>,
    node_adjuster: Box<dyn Fn(G::GL) -> NGL>,
    level_adjuster: Box<dyn Fn(G::LL) -> NLL>,
    new_group_label: PhantomData<NGL>,
    new_level_label: PhantomData<NLL>,
}

impl<G: GroupedGraphStructure, NGL, NLL> GroupLabelAdjuster<NGL, NLL, G> {
    pub fn new<A: Fn(G::GL) -> NGL + 'static, B: Fn(G::LL) -> NLL + 'static>(
        graph: G,
        node_adjuster: A,
        level_adjuster: B,
    ) -> GroupLabelAdjuster<NGL, NLL, G> {
        GroupLabelAdjuster::new_shared(MutRcRefCell::new(graph), node_adjuster, level_adjuster)
    }
    pub fn new_shared<A: Fn(G::GL) -> NGL + 'static, B: Fn(G::LL) -> NLL + 'static>(
        graph: MutRcRefCell<G>,
        node_adjuster: A,
        level_adjuster: B,
    ) -> GroupLabelAdjuster<NGL, NLL, G> {
        GroupLabelAdjuster {
            graph,
            node_adjuster: Box::new(node_adjuster),
            level_adjuster: Box::new(level_adjuster),
            new_group_label: PhantomData,
            new_level_label: PhantomData,
        }
    }
}

impl<G: GroupedGraphStructure, NGL, NLL> GroupedGraphStructure for GroupLabelAdjuster<NGL, NLL, G> {
    type T = G::T;
    type GL = NGL;
    type LL = NLL;
    type Tracker = G::Tracker;

    fn get_roots(&self) -> Vec<NodeGroupID> {
        self.graph.read().get_roots()
    }

    fn get_all_groups(&self) -> Vec<NodeGroupID> {
        self.graph.read().get_all_groups()
    }

    fn get_hidden(&self) -> Vec<NodeGroupID> {
        self.graph.read().get_hidden()
    }

    fn get_group(&self, node: NodeID) -> NodeGroupID {
        self.graph.read().get_group(node)
    }

    fn get_group_label(&self, node: NodeID) -> NGL {
        (self.node_adjuster)(self.graph.read().get_group_label(node))
    }

    fn get_parents(&self, group: NodeGroupID) -> Vec<EdgeCountData<G::T>> {
        self.graph.read().get_parents(group)
    }

    fn get_children(&self, group: NodeGroupID) -> Vec<EdgeCountData<G::T>> {
        self.graph.read().get_children(group)
    }

    fn get_nodes_of_group(&self, group: NodeGroupID) -> Vec<NodeID> {
        self.graph.read().get_nodes_of_group(group)
    }

    fn get_level_range(&self, group: NodeGroupID) -> (LevelNo, LevelNo) {
        self.graph.read().get_level_range(group)
    }

    fn get_level_label(&self, level: LevelNo) -> NLL {
        (self.level_adjuster)(self.graph.read().get_level_label(level))
    }

    fn refresh(&mut self) {
        self.graph.get().refresh()
    }

    fn create_node_tracker(&mut self) -> Self::Tracker {
        self.graph.get().create_node_tracker()
    }
}

impl<G: GroupedGraphStructure + StateStorage, NGL, NLL> StateStorage
    for GroupLabelAdjuster<NGL, NLL, G>
{
    fn write(&self, stream: &mut Cursor<&mut Vec<u8>>) -> Result<()> {
        self.graph.read().write(stream)?;
        Ok(())
    }

    fn read(&mut self, stream: &mut Cursor<&Vec<u8>>) -> Result<()> {
        self.graph.get().read(stream)?;
        Ok(())
    }
}
