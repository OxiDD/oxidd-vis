use std::collections::HashMap;

use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::layouts::{
            layered_layout::LayerOrdering,
            util::layered::layer_orderer::{EdgeMap, Order},
        },
        grouped_graph_structure::GroupedGraphStructure,
    },
    wasm_interface::NodeGroupID,
};

pub struct DummyLayerOrdering;
impl<T: Tag> LayerOrdering<T> for DummyLayerOrdering {
    fn order_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        dummy_group_start_id: NodeGroupID,
        dummy_edge_start_id: NodeGroupID,
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order> {
        layers.clone()
    }
}
