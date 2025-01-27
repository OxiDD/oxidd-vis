use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::{util::logging::console, wasm_interface::NodeID};

pub trait LayerOrderer {
    ///
    /// Orders the given layer, using the info of the given next layer.
    /// Note that edges may contain more edges than just the ones relevant for the nodes in the given order
    fn order(&self, layer: &Order, next_layer: &Order, edges: &EdgeMap) -> Order;

    ///
    /// Retrieves partial order information for the given full ordering of the given layer. This output consists of an set of equivalently ordered NodeID groups. Every group here is given in the order that it appeared in for the total order
    fn get_equal_groups(
        &self,
        layer: &Order,
        next_layer: &Order,
        edges: &EdgeMap,
    ) -> Vec<Vec<NodeID>>;

    /// Checks whether the layer is correctly sorted/ordered already
    fn is_sorted(&self, layer: &Order, next_layer: &Order, edges: &EdgeMap) -> bool;
}

pub type Order = HashMap<NodeID, usize>; // A mapping from node id to index in the order, hence this map should be a bijection from some X subset of NodeID, to the set of [0..|X|-1]
pub type EdgeMap = HashMap<NodeID, HashMap<NodeID, usize>>; // A mapping from node to node, with a given weight

pub type OrderedEdgeMap = HashMap<NodeID, Vec<NodeID>>;

// Counts the number of crossings between two layers
pub fn count_crossings(order: (&Order, &Order), edges: &EdgeMap) -> usize {
    let ordered_nodes = get_sequence(&order.0);
    let sorted_edge_indices: HashMap<NodeID, Vec<(NodeID, usize)>> = order
        .0
        .keys()
        .map(|node| {
            (
                *node,
                edges
                    .get(node)
                    .map(|weights| get_edge_index_sequence(weights.iter(), &order.1))
                    .unwrap_or_else(|| Vec::new()),
            )
        })
        .collect();

    let mut total = 0;
    let len = ordered_nodes.len();
    if len == 0 {
        return 0; // Prevent underflow in the loop
    }
    for i in 0..len - 1 {
        for j in i + 1..len {
            total += count_pair_crossings((
                sorted_edge_indices.get(&ordered_nodes[i]).unwrap(),
                sorted_edge_indices.get(&ordered_nodes[j]).unwrap(),
            ));
        }
    }
    total
}

pub fn swap_edges(edges: &EdgeMap) -> EdgeMap {
    let mut out = HashMap::new();
    for (from, node_edges) in edges {
        for (to, &weight) in node_edges {
            out.entry(*to)
                .or_insert_with(|| HashMap::new())
                .insert(*from, weight);
        }
    }
    out
}

pub fn count_pair_crossings(
    (node_edges, next_node_edges): (&Vec<(usize, usize)>, &Vec<(usize, usize)>),
) -> usize {
    let mut cross_count = 0;
    for &(edge, weight) in node_edges {
        for &(next_edge, next_weight) in next_node_edges {
            if next_edge >= edge {
                break;
            }
            // Every edge of the next node (on the right) that starts before the main node's edge (on the left), must cross
            cross_count += weight * next_weight;
        }
    }
    cross_count
}

pub fn get_edge_index_sequence<'a, I: Iterator<Item = (&'a NodeID, &'a usize)>>(
    edges: I,
    order: &Order,
) -> Vec<(usize, usize)> {
    edges
        .filter_map(|(to, &weight)| order.get(to).map(|&index| (index, weight)))
        .sorted()
        .collect()
}
pub fn get_ordered_edge_map(edge_map: &EdgeMap, orders: &Vec<Order>) -> OrderedEdgeMap {
    let mut out: OrderedEdgeMap = HashMap::new();

    if orders.len() == 0 {
        return out;
    }
    for i in 0..orders.len() - 1 {
        let layer = &orders[i];
        let next_layer = &orders[i + 1];
        for node in layer.keys() {
            if let Some(edges) = edge_map.get(node) {
                out.insert(
                    *node,
                    edges
                        .keys()
                        .filter_map(|to| next_layer.get(to).map(|index| (to, index)))
                        .sorted_by_key(|(_, &index)| index)
                        .map(|(to, _)| *to)
                        .collect(),
                );
            }
        }
    }
    out
}

pub fn get_sequence(order: &Order) -> Vec<NodeID> {
    let mut out = vec![0; order.len()];
    for (&node, index) in order {
        out[*index as usize] = node;
    }
    out
}

// 5->0, 7->0, 8->2, 4->0, 4->1, 0->0, 6->0, 2->0, 2->2
// 0->5, 0->4, 7->8, 6->4, 8->2, 3->7, 3->0, 3->6, 5->2
