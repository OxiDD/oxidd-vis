use std::collections::HashMap;

use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::layouts::{
            layered_layout_traits::LayerOrdering,
            util::layered::{
                barycenter_ordering::BarycenterOrdering,
                layer_orderer::{count_crossings, swap_edges, EdgeMap, LayerOrderer, Order},
            },
        },
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
    },
    util::logging::console,
    wasm_interface::{NodeGroupID, NodeID},
};

pub struct SugiyamaOrdering {
    layer_order: Box<dyn LayerOrderer>,
    max_phase1_iterations: usize,
    max_phase2_iterations: usize,
}

impl SugiyamaOrdering {
    pub fn new(max_phase1_iterations: usize, max_phase2_iterations: usize) -> SugiyamaOrdering {
        SugiyamaOrdering {
            layer_order: Box::new(BarycenterOrdering),
            max_phase1_iterations,
            max_phase2_iterations,
        }
    }
    pub fn new_custom(
        layer_order: Box<dyn LayerOrderer>,
        max_phase1_iterations: usize,
        max_phase2_iterations: usize,
    ) -> SugiyamaOrdering {
        SugiyamaOrdering {
            layer_order,
            max_phase1_iterations,
            max_phase2_iterations,
        }
    }
}

impl<T: DrawTag, GL, LL> LayerOrdering<T, GL, LL> for SugiyamaOrdering {
    fn order_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        dummy_group_start_id: NodeGroupID,
        dummy_edge_start_id: NodeGroupID,
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order> {
        hierarchical_barycenter_order(
            layers,
            edges,
            &*self.layer_order,
            self.max_phase1_iterations,
            self.max_phase2_iterations,
        )
    }
}

fn hierarchical_barycenter_order(
    orders: &Vec<Order>,
    edges: &EdgeMap,
    ordering: &LayerOrderer,
    max_phase1_iterations: usize,
    max_phase2_iterations: usize,
) -> Vec<Order> {
    let reversed_edges = &swap_edges(edges);
    let mut orders = orders.clone();
    for i in 0..max_phase2_iterations {
        orders = apply_barycenter_iterative(
            &orders,
            edges,
            reversed_edges,
            ordering,
            max_phase1_iterations,
        );

        if i == max_phase2_iterations - 1 {
            break;
        }

        if let Some(new_orders) = reverse_equal_nodes(&orders, edges, reversed_edges, ordering) {
            orders = new_orders;
        } else {
            console::log!("used {} phase 2 iters", i + 1);
            break;
        }
    }

    // Apply a final downwards sweep, since the resulting layout seems more intuitive for top-down reading of BDDs
    let old_crossings = count_graph_crossings(&orders, edges);
    let new_orders = apply_barycenters_oneway(&orders, reversed_edges, edges, ordering);
    let new_crossings = count_graph_crossings(&new_orders, edges);
    if new_crossings <= old_crossings {
        orders = new_orders;
    }
    console::log!("crossings old: {}, new: {}", old_crossings, new_crossings);

    orders
}

fn apply_barycenter_iterative(
    orders: &Vec<Order>,
    edges: &EdgeMap,
    reversed_edges: &EdgeMap,
    ordering: &LayerOrderer,
    max_iterations: usize,
) -> Vec<Order> {
    let mut orders = orders.clone();
    let mut crossings = count_graph_crossings(&orders, edges);
    for i in 0..max_iterations {
        let old_crossings = crossings;

        let new_down_orders = apply_barycenters_oneway(&orders, reversed_edges, edges, ordering);
        let new_crossings = count_graph_crossings(&new_down_orders, edges);
        if new_crossings <= crossings {
            crossings = new_crossings;
            orders = new_down_orders;
        }

        orders.reverse();
        let mut new_up_orders = apply_barycenters_oneway(&orders, edges, reversed_edges, ordering);
        new_up_orders.reverse();
        let new_crossings = count_graph_crossings(&new_up_orders, edges);
        if new_crossings <= crossings {
            crossings = new_crossings;
            orders = new_up_orders;
        } else {
            orders.reverse();
        }

        if old_crossings == crossings {
            console::log!("used {} phase 1 iters", i + 1);
            break; // No change detected
        }
    }
    orders
}

fn reverse_equal_nodes(
    orders: &Vec<Order>,
    edges: &EdgeMap,
    reversed_edges: &EdgeMap,
    ordering: &dyn LayerOrderer,
) -> Option<Vec<Order>> {
    let Some(first_layer) = orders.get(0) else {
        return None;
    };
    let mut out = orders.clone();
    let mut prev_layer = first_layer;

    let mut i = 0;
    for layer in orders.iter().skip(1) {
        let equal_groups = ordering.get_equal_groups(prev_layer, layer, edges);
        let new_prev_layer = reverse_groups(equal_groups, prev_layer);
        out[i] = new_prev_layer;

        if !ordering.is_sorted(layer, &out[i], reversed_edges) {
            return Some(out);
        }

        prev_layer = layer;
        i += 1;
    }

    let mut i = orders.len() - 1;
    let mut prev_layer = &orders[i];
    for layer in orders.iter().rev().skip(1) {
        let equal_groups = ordering.get_equal_groups(prev_layer, layer, reversed_edges);
        let new_prev_layer = reverse_groups(equal_groups, prev_layer);
        out[i] = new_prev_layer;

        if !ordering.is_sorted(layer, &out[i], edges) {
            return Some(out);
        }

        prev_layer = layer;
        i -= 1;
    }

    None
}

fn apply_barycenters_oneway(
    orders: &Vec<Order>,
    reverse_edges: &EdgeMap,
    edges: &EdgeMap,
    ordering: &dyn LayerOrderer,
) -> Vec<Order> {
    let Some(mut prev_layer) = orders.get(0) else {
        return orders.clone();
    };
    let mut out = vec![prev_layer.clone()];

    for layer in orders.iter().skip(1) {
        let new_layer = ordering.order(layer, &prev_layer, reverse_edges);
        if false {
            let old_crossings = count_crossings((prev_layer, layer), edges);
            let new_crossings = count_crossings((prev_layer, &new_layer), edges);
            out.push(if old_crossings >= new_crossings {
                new_layer
            } else {
                layer.clone()
            });
        } else {
            out.push(new_layer);
        }
        prev_layer = out.last().unwrap(); // == new_layer/layer
    }
    out
}

fn reverse_groups(equal_groups: Vec<Vec<NodeID>>, order: &Order) -> Order {
    let mut out = order.clone();
    for group in equal_groups {
        let len = group.len();
        for i in 0..len / 2 {
            let first_node = &group[i];
            let last_node = &group[len - i - 1];
            if let Some(first_pos) = order.get(first_node) {
                out.insert(*last_node, *first_pos);
            }
            if let Some(last_pos) = order.get(last_node) {
                out.insert(*first_node, *last_pos);
            }
        }
    }
    out
}

fn count_graph_crossings(orders: &Vec<Order>, edges: &EdgeMap) -> usize {
    orders
        .iter()
        .zip(orders.iter().skip(1))
        .map(|order| count_crossings(order, edges))
        .fold(0, |val, prev| val + prev)
}
