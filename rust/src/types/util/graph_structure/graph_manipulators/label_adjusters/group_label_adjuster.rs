use std::{marker::PhantomData, vec::IntoIter};

use oxidd::LevelNo;

use crate::{
    types::util::graph_structure::{
        graph_structure::DrawTag,
        grouped_graph_structure::{EdgeCountData, GroupedGraphStructure},
    },
    util::rc_refcell::MutRcRefCell,
    wasm_interface::{NodeGroupID, NodeID},
};

pub struct GroupLabelAdjuster<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, NGL, NLL> {
    graph: MutRcRefCell<G>,
    node_adjuster: Box<dyn Fn(GL) -> NGL>,
    level_adjuster: Box<dyn Fn(LL) -> NLL>,
    tag: PhantomData<T>,
    group_label: PhantomData<GL>,
    new_group_label: PhantomData<NGL>,
    level_label: PhantomData<LL>,
    new_level_label: PhantomData<NLL>,
}

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, NGL, NLL>
    GroupLabelAdjuster<T, GL, LL, G, NGL, NLL>
{
    pub fn new<A: Fn(GL) -> NGL + 'static, B: Fn(LL) -> NLL + 'static>(
        graph: G,
        node_adjuster: A,
        level_adjuster: B,
    ) -> GroupLabelAdjuster<T, GL, LL, G, NGL, NLL> {
        GroupLabelAdjuster::new_shared(MutRcRefCell::new(graph), node_adjuster, level_adjuster)
    }
    pub fn new_shared<A: Fn(GL) -> NGL + 'static, B: Fn(LL) -> NLL + 'static>(
        graph: MutRcRefCell<G>,
        node_adjuster: A,
        level_adjuster: B,
    ) -> GroupLabelAdjuster<T, GL, LL, G, NGL, NLL> {
        GroupLabelAdjuster {
            graph,
            node_adjuster: Box::new(node_adjuster),
            level_adjuster: Box::new(level_adjuster),
            tag: PhantomData,
            group_label: PhantomData,
            new_group_label: PhantomData,
            level_label: PhantomData,
            new_level_label: PhantomData,
        }
    }
}

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, NGL, NLL>
    GroupedGraphStructure<T, NGL, NLL> for GroupLabelAdjuster<T, GL, LL, G, NGL, NLL>
{
    type Tracker = G::Tracker;

    fn get_roots(&self) -> Vec<NodeGroupID> {
        self.graph.read().get_roots()
    }

    fn get_all_groups(&self) -> Vec<NodeGroupID> {
        self.graph.read().get_all_groups()
    }

    fn get_hidden(&self) -> Option<NodeGroupID> {
        self.graph.read().get_hidden()
    }

    fn get_group(&self, node: NodeID) -> NodeGroupID {
        self.graph.read().get_group(node)
    }

    fn get_group_label(&self, node: NodeID) -> NGL {
        (self.node_adjuster)(self.graph.read().get_group_label(node))
    }

    fn get_parents(&self, group: NodeGroupID) -> IntoIter<EdgeCountData<T>> {
        self.graph.read().get_parents(group)
    }

    fn get_children(
        &self,
        group: crate::wasm_interface::NodeGroupID,
    ) -> IntoIter<EdgeCountData<T>> {
        self.graph.read().get_children(group)
    }

    fn get_nodes_of_group(&self, group: NodeGroupID) -> IntoIter<NodeID> {
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
