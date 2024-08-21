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

pub struct GroupLabelAdjuster<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, NGL> {
    graph: MutRcRefCell<G>,
    adjuster: Box<dyn Fn(GL) -> NGL>,
    tag: PhantomData<T>,
    group_label: PhantomData<GL>,
    new_group_label: PhantomData<NGL>,
    level_label: PhantomData<LL>,
}

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, NGL>
    GroupLabelAdjuster<T, GL, LL, G, NGL>
{
    pub fn new<A: Fn(GL) -> NGL + 'static>(
        graph: G,
        adjuster: A,
    ) -> GroupLabelAdjuster<T, GL, LL, G, NGL> {
        GroupLabelAdjuster::new_shared(MutRcRefCell::new(graph), adjuster)
    }
    pub fn new_shared<A: Fn(GL) -> NGL + 'static>(
        graph: MutRcRefCell<G>,
        adjuster: A,
    ) -> GroupLabelAdjuster<T, GL, LL, G, NGL> {
        GroupLabelAdjuster {
            graph,
            adjuster: Box::new(adjuster),
            tag: PhantomData,
            group_label: PhantomData,
            new_group_label: PhantomData,
            level_label: PhantomData,
        }
    }
}

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, NGL> GroupedGraphStructure<T, NGL, LL>
    for GroupLabelAdjuster<T, GL, LL, G, NGL>
{
    type Tracker = G::Tracker;

    fn get_root(&self) -> NodeGroupID {
        self.graph.read().get_root()
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
        (self.adjuster)(self.graph.read().get_group_label(node))
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

    fn get_level_label(&self, level: LevelNo) -> LL {
        self.graph.read().get_level_label(level)
    }

    fn refresh(&mut self) {
        self.graph.get().refresh()
    }

    fn get_source_reader(&mut self) -> Self::Tracker {
        self.graph.get().get_source_reader()
    }
}
