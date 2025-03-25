use std::collections::HashMap;

use itertools::Itertools;
use js_sys::Math::random;
use oxidd_core::Tag;
use seeded_random::{Random, Seed};

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
    util::logging::console,
    wasm_interface::NodeGroupID,
};

pub struct PseudoRandomLayerOrdering {
    swaps: usize,
    seed: usize,
}
impl PseudoRandomLayerOrdering {
    pub fn new(swaps_per_node: usize, seed: usize) -> PseudoRandomLayerOrdering {
        PseudoRandomLayerOrdering {
            swaps: swaps_per_node,
            seed: seed,
        }
    }
    pub fn set_seed(&mut self, seed: usize) -> () {
        self.seed = seed;
    }
}
impl<T: DrawTag, GL, LL> LayerOrdering<T, GL, LL> for PseudoRandomLayerOrdering {
    fn order_nodes(
        &self,
        _graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        _edges: &EdgeMap,
        _dummy_group_start_id: NodeGroupID,
        _dummy_edge_start_id: NodeGroupID,
        _owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order> {
        let mut rng = Random::from_seed(Seed::unsafe_new(self.seed as u64));
        layers
            .iter()
            .map(|layer| {
                let mut layer = layer.clone();
                for _ in 0..self.swaps * layer.len() {
                    swap_in_layer(&mut layer, &mut rng);
                }
                layer
            })
            .collect()
    }
}

fn swap_in_layer(order: &mut Order, rng: &mut Random) {
    let keys = order.keys().sorted().collect_vec();
    let index1 = rng.range(0, keys.len() as u32) as usize;
    let key1 = **keys.get(index1).unwrap();
    let pos1 = *order.get(&key1).unwrap();
    let index2 = rng.range(0, keys.len() as u32) as usize;
    let key2 = **keys.get(index2).unwrap();
    let pos2 = *order.get(&key2).unwrap();
    order.insert(key1, pos2);
    order.insert(key2, pos1);
}
