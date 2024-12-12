use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use num_rational::Ratio;

use crate::{util::logging::console, wasm_interface::NodeID};

use super::layer_orderer::{
    count_crossings, get_sequence, swap_edges, EdgeMap, LayerOrderer, Order,
};

pub struct BarycenterOrdering;

impl LayerOrderer for BarycenterOrdering {
    fn order(&self, layer: &Order, next_layer: &Order, edges: &EdgeMap) -> Order {
        apply_single_barycenter_order(layer, next_layer, edges)
    }

    fn get_equal_groups(
        &self,
        layer: &Order,
        next_layer: &Order,
        edges: &EdgeMap,
    ) -> Vec<Vec<NodeID>> {
        get_equal_barycenter_groups(layer, next_layer, edges)
    }

    fn is_sorted(&self, layer: &Order, next_layer: &Order, edges: &EdgeMap) -> bool {
        is_barycenter_sorted(layer, next_layer, edges)
    }
}

// Applies a single barycenter sort on the second row of the order
fn apply_single_barycenter_order(layer: &Order, next_layer: &Order, edges: &EdgeMap) -> Order {
    layer
        .iter()
        .map(|(&node, &index)| (node, get_barycenter(node, next_layer, edges), index))
        .sorted_by_key(|&(_, center, prev_index)| (center, prev_index)) // Use nodeID as a secondary sort condition for stability
        // .sorted_by_key(|(_, center)| *center)
        .enumerate()
        .map(|(index, (node, _, _))| (node, index))
        .collect()
}

fn get_barycenter(node: NodeID, other_layer: &Order, edges: &EdgeMap) -> Ratio<usize> {
    let mut sum = 0;
    let Some(edges) = edges.get(&node) else {
        return Ratio::new(0, 1);
    };

    let mut total_weights = 0;
    for (to, weight) in edges {
        if let Some(node_pos) = other_layer.get(to) {
            sum += *node_pos * weight;
            total_weights += weight;
        }
    }
    Ratio::new(sum, total_weights)
}

fn get_equal_barycenter_groups(
    layer: &Order,
    next_layer: &Order,
    edges: &EdgeMap,
) -> Vec<Vec<NodeID>> {
    let mut equal_groups: HashMap<Ratio<usize>, Vec<NodeID>> = HashMap::new();
    for node in get_sequence(layer) {
        let barycenter = get_barycenter(node, next_layer, edges);
        equal_groups
            .entry(barycenter)
            .or_insert_with(|| Vec::new())
            .push(node);
    }

    equal_groups
        .iter()
        .map(|(_, group)| group.clone())
        .filter(|group| group.len() <= 1)
        .collect()
}

fn is_barycenter_sorted(layer: &Order, other_layer: &Order, edges: &EdgeMap) -> bool {
    let seq = get_sequence(layer);
    let Some(first_node) = seq.get(0) else {
        return true;
    };
    let mut prev_barycenter = get_barycenter(*first_node, other_layer, edges);
    for node in seq.iter().skip(1) {
        let barycenter = get_barycenter(*node, other_layer, edges);
        if prev_barycenter > barycenter {
            return false;
        }
        prev_barycenter = barycenter;
    }
    return true;
}

///
/// An unused (and untested) function that performs the two layer barycenter ordering approach
fn two_layer_barycenter_order(
    init_order: (&Order, &Order),
    edges: &EdgeMap,
    max_phase1_iterations: usize,
    max_phase2_iterations: usize,
) -> (Order, Order) {
    let reversed_edges = &swap_edges(edges);
    let mut order = (init_order.0.clone(), init_order.1.clone());
    for _ in 0..max_phase2_iterations {
        order = apply_barycenter_order(
            (init_order.0.clone(), init_order.1.clone()),
            edges,
            reversed_edges,
            max_phase1_iterations,
        );

        let first_layer_rev = reverse_equal_barycenters(&order.0, &order.1, edges);
        if are_barycenters_increasing(&order.1, &first_layer_rev, reversed_edges) {
            let second_layer_rev =
                reverse_equal_barycenters(&order.1, &first_layer_rev, reversed_edges);
            if are_barycenters_increasing(&first_layer_rev, &second_layer_rev, edges) {
                // If all barycenters are increasing, ordering according to barycenters won't cause changes
                return (first_layer_rev, second_layer_rev);
            } else {
                order = (first_layer_rev, second_layer_rev);
            }
        } else {
            order = (first_layer_rev, order.1);
        }
    }

    order
}

fn apply_barycenter_order(
    mut order: (Order, Order),
    edges: &EdgeMap,
    reversed_edges: &EdgeMap,
    max_iterations: usize,
) -> (Order, Order) {
    let mut crossings = count_crossings((&order.0, &order.1), edges);
    for _ in 0..max_iterations {
        let old_crossings = crossings;

        let new_bottom_order = apply_single_barycenter_order(&order.1, &order.0, reversed_edges);
        let new_crossings = count_crossings((&order.0, &new_bottom_order), reversed_edges);
        if new_crossings < crossings {
            crossings = new_crossings;
            order = (order.0, new_bottom_order);
        }

        let new_top_order = apply_single_barycenter_order(&order.0, &order.1, edges);
        let new_crossings = count_crossings((&new_top_order, &order.1), edges);
        if new_crossings < crossings {
            crossings = new_crossings;
            order = (new_top_order, order.1);
        }

        if old_crossings == crossings {
            break; // No change detected
        }
    }
    order
}

fn reverse_equal_barycenters(layer: &Order, other_layer: &Order, edges: &EdgeMap) -> Order {
    let seq = get_sequence(layer);
    let len = seq.len();

    let mut layer = layer.clone();
    if len == 0 {
        return layer;
    }

    let mut group_start = 0;
    let mut prev_barycenter = Some(get_barycenter(seq[0], other_layer, edges));
    for i in 1..=len {
        let barycenter = if i < len {
            Some(get_barycenter(seq[i], other_layer, edges))
        } else {
            None
        };
        if barycenter != prev_barycenter {
            let group_end = i;
            if group_end != group_start + 1 {
                for j in group_start..group_end {
                    let new_pos = group_end - (j - group_start) - 1;
                    layer.insert(seq[j], new_pos);
                }
            }

            group_start = i;
            prev_barycenter = barycenter;
        }
    }
    layer
}
fn are_barycenters_increasing(layer: &Order, other_layer: &Order, edges: &EdgeMap) -> bool {
    let seq = get_sequence(layer);
    let len = seq.len();

    let mut prev_barycenter = get_barycenter(seq[0], other_layer, edges);
    for i in 1..len {
        let barycenter = get_barycenter(seq[i], other_layer, edges);

        if barycenter <= prev_barycenter {
            return false;
        }
        prev_barycenter = barycenter
    }

    true
}
