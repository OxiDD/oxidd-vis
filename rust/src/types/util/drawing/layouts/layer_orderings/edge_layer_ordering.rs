use std::collections::HashMap;

use itertools::Itertools;
use js_sys::Math::random;
use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::layouts::{
            layered_layout_traits::LayerOrdering,
            util::layered::layer_orderer::{swap_edges, EdgeMap, Order},
        },
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
    },
    wasm_interface::NodeGroupID,
};

/// Sorts nodes according to incoming and outgoing edge ordering values
pub struct EdgeLayerOrdering;
impl<G: GroupedGraphStructure> LayerOrdering<G> for EdgeLayerOrdering {
    fn order_nodes(
        &self,
        _graph: &G,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        _dummy_group_start_id: NodeGroupID,
        _dummy_edge_start_id: NodeGroupID,
        _owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order> {
        let reverse_edges = swap_edges(edges);
        layers
            .iter()
            .map(|layer| {
                layer
                    .iter()
                    .map(|(&node, &index)| {
                        (
                            node,
                            get_node_order_weight(node, edges, &reverse_edges),
                            index,
                        )
                    })
                    .sorted_by_key(|&(_, order, prev_index)| (order, prev_index)) // Use nodeID as a secondary sort condition for stability
                    // .sorted_by_key(|(_, center)| *center)
                    .enumerate()
                    .map(|(index, (node, _, _))| (node, index))
                    .collect()
            })
            .collect()
    }
}

fn get_node_order_weight(node: usize, edges: &EdgeMap, reverse_edges: &EdgeMap) -> (i32, i32) {
    let incoming = reverse_edges
        .get(&node)
        .map(|e| e.iter().fold(0, |sum, (_, t)| sum + t.order))
        .unwrap_or_default();
    let outgoing = edges
        .get(&node)
        .map(|e| e.iter().fold(0, |sum, (_, t)| sum + t.order))
        .unwrap_or_default();
    (incoming, outgoing)
}
