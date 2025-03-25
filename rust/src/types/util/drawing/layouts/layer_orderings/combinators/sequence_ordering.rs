use std::{collections::HashMap, marker::PhantomData};

use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::layouts::{
            layered_layout_traits::LayerOrdering,
            util::layered::layer_orderer::{EdgeMap, Order},
        },
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
    },
    wasm_interface::NodeGroupID,
};

pub struct SequenceOrdering<
    T: DrawTag,
    GL,
    LL,
    O1: LayerOrdering<T, GL, LL>,
    O2: LayerOrdering<T, GL, LL>,
> {
    ordering1: O1,
    ordering2: O2,
    tag: PhantomData<T>,
    group_label: PhantomData<GL>,
    level_label: PhantomData<LL>,
}
impl<T: DrawTag, GL, LL, O1: LayerOrdering<T, GL, LL>, O2: LayerOrdering<T, GL, LL>>
    SequenceOrdering<T, GL, LL, O1, O2>
{
    pub fn new(o1: O1, o2: O2) -> SequenceOrdering<T, GL, LL, O1, O2> {
        SequenceOrdering {
            ordering1: o1,
            ordering2: o2,
            tag: PhantomData,
            group_label: PhantomData,
            level_label: PhantomData,
        }
    }
    pub fn get_ordering1(&mut self) -> &mut O1 {
        &mut self.ordering1
    }
    pub fn get_ordering2(&mut self) -> &mut O2 {
        &mut self.ordering2
    }
}
impl<T: DrawTag, GL, LL, O1: LayerOrdering<T, GL, LL>, O2: LayerOrdering<T, GL, LL>>
    LayerOrdering<T, GL, LL> for SequenceOrdering<T, GL, LL, O1, O2>
{
    fn order_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        dummy_group_start_id: NodeGroupID,
        dummy_edge_start_id: NodeGroupID,
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order> {
        let o1 = self.ordering1.order_nodes(
            graph,
            layers,
            edges,
            dummy_group_start_id,
            dummy_edge_start_id,
            owners,
        );
        self.ordering2.order_nodes(
            graph,
            &o1,
            edges,
            dummy_group_start_id,
            dummy_edge_start_id,
            owners,
        )
    }
}
