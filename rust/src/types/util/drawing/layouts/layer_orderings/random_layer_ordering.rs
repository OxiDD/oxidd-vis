use std::collections::HashMap;

use itertools::Itertools;
use js_sys::Math::random;
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

pub struct RandomLayerOrdering {
    swaps: usize,
}
impl RandomLayerOrdering {
    pub fn new(swaps_per_node: usize) -> RandomLayerOrdering {
        RandomLayerOrdering {
            swaps: swaps_per_node,
        }
    }
}
impl<T: DrawTag, GL, LL> LayerOrdering<T, GL, LL> for RandomLayerOrdering {
    fn order_nodes(
        &self,
        _graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        _edges: &EdgeMap,
        _dummy_group_start_id: NodeGroupID,
        _dummy_edge_start_id: NodeGroupID,
        _owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order> {
        layers
            .iter()
            .map(|layer| {
                let mut layer = layer.clone();
                for _ in 0..self.swaps * layer.len() {
                    swap_in_layer(&mut layer);
                }
                layer
            })
            .collect()
    }
}

fn swap_in_layer(order: &mut Order) {
    let keys = order.keys().collect_vec();
    let index1 = (random() * keys.len() as f64).floor() as usize;
    let key1 = **keys.get(index1).unwrap();
    let pos1 = *order.get(&key1).unwrap();
    let index2 = (random() * keys.len() as f64).floor() as usize;
    let key2 = **keys.get(index2).unwrap();
    let pos2 = *order.get(&key2).unwrap();
    order.insert(key1, pos2);
    order.insert(key2, pos1);
}
