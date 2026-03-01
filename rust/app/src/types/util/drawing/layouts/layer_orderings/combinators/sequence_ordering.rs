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

pub struct SequenceOrdering<G: GroupedGraphStructure, O1: LayerOrdering<G>, O2: LayerOrdering<G>> {
    ordering1: O1,
    ordering2: O2,
    graph: PhantomData<G>,
}
impl<G: GroupedGraphStructure, O1: LayerOrdering<G>, O2: LayerOrdering<G>>
    SequenceOrdering<G, O1, O2>
{
    pub fn new(o1: O1, o2: O2) -> SequenceOrdering<G, O1, O2> {
        SequenceOrdering {
            ordering1: o1,
            ordering2: o2,
            graph: PhantomData,
        }
    }
    pub fn get_ordering1(&mut self) -> &mut O1 {
        &mut self.ordering1
    }
    pub fn get_ordering2(&mut self) -> &mut O2 {
        &mut self.ordering2
    }
}
impl<G: GroupedGraphStructure, O1: LayerOrdering<G>, O2: LayerOrdering<G>> LayerOrdering<G>
    for SequenceOrdering<G, O1, O2>
{
    fn order_nodes(
        &self,
        graph: &G,
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
