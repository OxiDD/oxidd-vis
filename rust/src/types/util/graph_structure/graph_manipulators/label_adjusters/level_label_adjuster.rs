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

pub struct LevelLabelAdjuster<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, NLL> {
    graph: MutRcRefCell<G>,
    adjuster: Box<dyn Fn(LL) -> NLL>,
    tag: PhantomData<T>,
    group_label: PhantomData<GL>,
    level_label: PhantomData<LL>,
    new_level_label: PhantomData<NLL>,
}

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, NLL>
    LevelLabelAdjuster<T, GL, LL, G, NLL>
{
    pub fn new<A: Fn(LL) -> NLL + 'static>(
        graph: G,
        adjuster: A,
    ) -> LevelLabelAdjuster<T, GL, LL, G, NLL> {
        LevelLabelAdjuster::new_shared(MutRcRefCell::new(graph), adjuster)
    }
    pub fn new_shared<A: Fn(LL) -> NLL + 'static>(
        graph: MutRcRefCell<G>,
        adjuster: A,
    ) -> LevelLabelAdjuster<T, GL, LL, G, NLL> {
        LevelLabelAdjuster {
            graph,
            adjuster: Box::new(adjuster),
            tag: PhantomData,
            group_label: PhantomData,
            level_label: PhantomData,
            new_level_label: PhantomData,
        }
    }
}

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, NLL> GroupedGraphStructure<T, GL, NLL>
    for LevelLabelAdjuster<T, GL, LL, G, NLL>
{
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

    fn get_group_label(&self, node: NodeID) -> GL {
        self.graph.read().get_group_label(node)
    }

    fn get_parents(&self, group: NodeGroupID) -> Vec<EdgeCountData<T>> {
        self.graph.read().get_parents(group)
    }

    fn get_children(&self, group: NodeGroupID) -> Vec<EdgeCountData<T>> {
        self.graph.read().get_children(group)
    }

    fn get_nodes_of_group(&self, group: NodeGroupID) -> Vec<NodeID> {
        self.graph.read().get_nodes_of_group(group)
    }

    fn get_level_range(&self, group: NodeGroupID) -> (LevelNo, LevelNo) {
        self.graph.read().get_level_range(group)
    }

    fn get_level_label(&self, level: LevelNo) -> NLL {
        (self.adjuster)(self.graph.read().get_level_label(level))
    }

    fn refresh(&mut self) {
        self.graph.get().refresh()
    }

    fn create_node_tracker(&mut self) -> Self::Tracker {
        self.graph.get().create_node_tracker()
    }
}

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, NLL> StateStorage
    for LevelLabelAdjuster<T, GL, LL, G, NLL>
where
    G: StateStorage,
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
