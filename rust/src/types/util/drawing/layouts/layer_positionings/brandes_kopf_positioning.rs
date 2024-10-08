use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use oxidd::LevelNo;
use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::{
            diagram_layout::Point,
            layouts::{
                layered_layout::{is_edge_dummy, is_group_dummy},
                layered_layout_traits::{NodePositioning, WidthLabel},
                util::layered::layer_orderer::{
                    get_edge_index_sequence, get_ordered_edge_map, get_sequence, swap_edges,
                    EdgeMap, Order, OrderedEdgeMap,
                },
            },
        },
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
    },
    util::logging::console,
    wasm_interface::NodeGroupID,
};

pub struct BrandesKopfPositioning;

impl<T: DrawTag, GL: WidthLabel, LL> NodePositioning<T, GL, LL> for BrandesKopfPositioning {
    fn position_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        dummy_group_start_id: NodeGroupID,
        dummy_edge_start_id: NodeGroupID,
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> (HashMap<NodeGroupID, Point>, HashMap<LevelNo, f32>) {
        let spacing = 2.0;

        // Remove edges between dummy group nodes and nodes from other groups, such that node positioning will align these items
        let edges = remove_internal_group_to_other_edges(
            edges,
            &owners,
            dummy_group_start_id,
            dummy_edge_start_id,
        );

        let x_coords = balanced_layout(
            layers,
            &edges,
            dummy_group_start_id,
            dummy_edge_start_id,
            owners,
            spacing,
        );
        (
            layers
                .iter()
                .enumerate()
                .flat_map(|(index, layer)| {
                    let y_coord = (layers.len() - index) as f32 * spacing;
                    let x_coords = &x_coords; // create a new ref that can be moved
                    layer.keys().map(move |node| {
                        (
                            *node,
                            Point {
                                x: x_coords[node],
                                y: y_coord,
                            },
                        )
                    })
                })
                .collect(),
            layers
                .iter()
                .enumerate()
                .map(|(level, _)| (level as u32, (layers.len() - level) as f32 * spacing))
                .collect(),
        )
    }
}

fn remove_internal_group_to_other_edges(
    edges: &EdgeMap,
    dummy_owners: &HashMap<NodeGroupID, NodeGroupID>,
    dummy_group_start_id: NodeGroupID,
    dummy_edge_start_id: NodeGroupID,
) -> EdgeMap {
    edges
        .iter()
        .map(|(&node, edges)| {
            // Is any node representing a group, except the last node representing said group
            let is_non_end_group_node = dummy_owners.get(&node).is_some()
                && edges
                    .iter()
                    .any(|to| dummy_owners.get(&node) == dummy_owners.get(to));

            (
                node,
                if is_non_end_group_node {
                    edges
                        .iter()
                        .filter(|&to| dummy_owners.get(&node) == dummy_owners.get(to))
                        // .filter(|&to| false)
                        .cloned()
                        .collect()
                } else {
                    edges
                        .iter()
                        .filter(|&&to| {
                            !is_group_dummy(to, dummy_group_start_id, dummy_edge_start_id)
                        })
                        .cloned()
                        .collect()
                },
            )
        })
        .collect()
}

fn balanced_layout(
    layers: &Vec<Order>,
    edges: &EdgeMap,
    dummy_group_start_id: NodeGroupID,
    dummy_edge_start_id: NodeGroupID,
    owners: &HashMap<NodeGroupID, NodeGroupID>,
    spacing: f32,
) -> HashMap<NodeGroupID, f32> {
    let up_edges = swap_edges(edges);
    let mut up_layers = layers.clone();
    up_layers.reverse();

    let right_layers = &get_reverse_layers(layers); // right to left layer
    let right_up_layers = &get_reverse_layers(&up_layers); // right to left + bottom to top layer

    let left_up_edges = get_ordered_edge_map(&up_edges, &up_layers);
    let left_down_edges = get_ordered_edge_map(&edges, &layers);
    let right_up_edges = get_ordered_edge_map(&up_edges, &right_up_layers);
    let right_down_edges = get_ordered_edge_map(&edges, &right_layers);

    let left_down_layout = shift_layout(&compact_horizontally(
        layers,
        &left_up_edges,
        dummy_group_start_id,
        dummy_edge_start_id,
        owners,
        spacing,
    ));
    let left_up_layout = shift_layout(&compact_horizontally(
        &up_layers,
        &left_down_edges,
        dummy_group_start_id,
        dummy_edge_start_id,
        owners,
        spacing,
    ));

    let right_down_layout = shift_layout(&compact_horizontally(
        &right_layers,
        &right_up_edges,
        dummy_group_start_id,
        dummy_edge_start_id,
        owners,
        spacing,
    ))
    .iter()
    .map(|(&node, &x)| (node, -x))
    .collect::<HashMap<NodeGroupID, f32>>();
    let right_up_layout = shift_layout(&compact_horizontally(
        &right_up_layers,
        &right_down_edges,
        dummy_group_start_id,
        dummy_edge_start_id,
        owners,
        spacing,
    ))
    .iter()
    .map(|(&node, x)| (node, -x))
    .collect::<HashMap<NodeGroupID, f32>>();

    left_down_layout
        .keys()
        .map(|node| {
            let mut values = vec![
                left_down_layout.get(node).unwrap(),
                left_up_layout.get(node).unwrap(),
                right_down_layout.get(node).unwrap(),
                right_up_layout.get(node).unwrap(),
            ];
            values.sort_by(|a, b| f32::total_cmp(a, b));
            (*node, (values[1] + values[2]) / 2.0)
        })
        .collect()
}

fn shift_layout(layout: &HashMap<NodeGroupID, f32>) -> HashMap<NodeGroupID, f32> {
    // This makes sure the layout is 0 aligned to the left
    let Some(min) = layout.values().map(|&v| v).reduce(|a, b| f32::min(a, b)) else {
        return layout.clone();
    };
    layout
        .iter()
        .map(|(&node, val)| (node, val - min))
        .collect()
}

fn get_reverse_layers(layers: &Vec<Order>) -> Vec<Order> {
    layers
        .iter()
        .map(|layer| {
            let len = layer.len();
            layer
                .iter()
                .map(|(&node, index)| (node, len - 1 - index))
                .collect()
        })
        .collect()
}

fn compact_horizontally(
    layers: &Vec<Order>,
    reverse_edges: &OrderedEdgeMap,
    dummy_group_start_id: NodeGroupID,
    dummy_edge_start_id: NodeGroupID,
    owners: &HashMap<NodeGroupID, NodeGroupID>,
    spacing: f32,
) -> HashMap<NodeGroupID, f32> {
    let layer_seqs = layers
        .iter()
        .map(|layer| get_sequence(layer))
        .collect::<Vec<Vec<NodeGroupID>>>();
    let alignment = align_vertical(
        layers,
        &layer_seqs,
        reverse_edges,
        dummy_group_start_id,
        dummy_edge_start_id,
        owners,
    );

    let all_nodes = layers.iter().flat_map(|layer| layer.keys());
    let pred = HashMap::from_iter(layer_seqs.iter().flat_map(|layer| {
        let node_seq = layer.iter().map(|node| *node);
        node_seq.clone().skip(1).zip(node_seq)
    }));
    let node_identity = all_nodes.clone().map(|node| (*node, *node));
    let mut sink = HashMap::from_iter(node_identity);
    let mut shift = HashMap::new();
    let mut x = HashMap::new();

    for &node in all_nodes.clone() {
        if alignment.root[&node] == node {
            place_block(
                node, &mut sink, &mut shift, &mut x, &pred, &alignment, spacing,
            );
        }
    }

    for node in all_nodes.clone() {
        let root = &alignment.root[node];
        x.insert(*node, x[root]);
    }
    for node in all_nodes {
        let sink = &sink[&alignment.root[node]];
        if shift.contains_key(sink) {
            x.insert(*node, x[node] + shift[sink]);
        }
    }

    x
}

fn place_block(
    root_node: NodeGroupID,
    sink: &mut HashMap<NodeGroupID, NodeGroupID>,
    shift: &mut HashMap<NodeGroupID, f32>,
    x: &mut HashMap<NodeGroupID, f32>,
    pred: &HashMap<NodeGroupID, NodeGroupID>,
    alignment: &VerticalAlignment,
    spacing: f32,
) {
    if !x.contains_key(&root_node) {
        x.insert(root_node, 0.0);
        let mut node = root_node; // node is iterated over from the root down
        loop {
            if pred.contains_key(&node) {
                let pred_root = alignment.root[&pred[&node]];
                place_block(pred_root, sink, shift, x, pred, alignment, spacing);
                let pred_sink = sink[&pred_root];
                if sink[&root_node] == root_node {
                    sink.insert(root_node, pred_sink);
                }
                if sink[&root_node] != pred_sink {
                    let delta = x[&root_node] - x[&pred_root] - spacing;
                    shift.insert(
                        pred_sink,
                        if shift.contains_key(&pred_sink) {
                            f32::min(shift[&pred_sink], delta)
                        } else {
                            delta
                        },
                    );
                } else {
                    x.insert(root_node, f32::max(x[&root_node], x[&pred_root] + spacing));
                }
            }
            node = alignment.align[&node];
            if node == root_node {
                break;
            }
        }
    }
}

struct VerticalAlignment {
    root: HashMap<NodeGroupID, NodeGroupID>,
    align: HashMap<NodeGroupID, NodeGroupID>,
}
fn align_vertical(
    layers: &Vec<Order>,
    layer_seqs: &Vec<Vec<NodeGroupID>>,
    reverse_edges: &OrderedEdgeMap,
    dummy_group_start_id: NodeGroupID,
    dummy_edge_start_id: NodeGroupID,
    owners: &HashMap<NodeGroupID, NodeGroupID>,
) -> VerticalAlignment {
    let conflicts = get_type1_conflicts(
        layers,
        &layer_seqs,
        reverse_edges,
        dummy_group_start_id,
        dummy_edge_start_id,
        owners,
    );
    let mut root = HashMap::from_iter(
        layers
            .iter()
            .flat_map(|layer| layer.keys().map(|node| (*node, *node))),
    );
    let mut align = HashMap::from(root.clone());
    for i in 1..layers.len() {
        let mut r: isize = -1;

        let layer_seq = &layer_seqs[i];
        let prev_layer = &layers[i - 1];
        for k in 0..layer_seq.len() {
            let node = layer_seq[k];
            let Some(upper_neighbors) = reverse_edges.get(&node) else {
                continue;
            };
            let len = upper_neighbors.len();
            if len == 0 {
                continue;
            }

            let average = ((len - 1) as f32) / 2.0;
            let lower_median = f32::floor(average) as usize;
            let upper_median = f32::ceil(average) as usize;
            for m in lower_median..=upper_median {
                if align.get(&node) == Some(&node) {
                    let upper_node = upper_neighbors[m];
                    let pos = *prev_layer.get(&upper_node).unwrap() as isize;
                    if !conflicts.contains(&(upper_node, node)) && r < pos {
                        align.insert(upper_node, node);
                        root.insert(node, root[&upper_node]);
                        align.insert(node, root[&node]);
                        r = pos;
                    }
                }
            }
        }
    }

    VerticalAlignment { root, align }
}

fn get_type1_conflicts(
    layers: &Vec<Order>,
    layer_seqs: &Vec<Vec<NodeGroupID>>,
    reverse_edges: &OrderedEdgeMap,
    dummy_group_start_id: NodeGroupID,
    dummy_edge_start_id: NodeGroupID,
    owners: &HashMap<NodeGroupID, NodeGroupID>,
) -> HashSet<(NodeGroupID, NodeGroupID)> {
    let mut conflicts = HashSet::new();
    for i in 1..usize::max(layer_seqs.len(), 2) - 2 {
        let mut k0 = 0; // Previous inner segment (start) at layer i
        let mut l = 0; // Previous considered node at layer i+1

        let prev_layer_seq = &layer_seqs[i];
        let prev_layer = &layers[i];
        let layer_seq = &layer_seqs[i + 1];
        let layer_len = layer_seq.len();
        for l1 in 0..layer_len {
            // Currently considered node at layer i+1
            let node = layer_seq[l1];

            let incident_inner_segment = is_edge_dummy(node, dummy_edge_start_id)
                && reverse_edges.get(&node).map_or(false, |tos| {
                    tos.iter()
                        .find(|&&to| is_edge_dummy(to, dummy_edge_start_id))
                        .is_some()
                });
            let incident_group_segment = owners.contains_key(&node)
                && reverse_edges.get(&node).map_or(false, |tos| {
                    tos.iter()
                        .find(|&to| owners.get(&node) == owners.get(to))
                        .is_some()
                });
            let last = l1 == layer_len - 1;
            if last || incident_inner_segment || incident_group_segment {
                let inner_edge_node = node;
                if prev_layer_seq.len() == 0 {
                    continue;
                }
                let mut k1 = prev_layer_seq.len() - 1; // Currently considered inner segment at layer i (or the default last node place holder to correctly finish processing the previous inner segment)
                if incident_inner_segment || incident_group_segment {
                    let upper_neighbors = reverse_edges.get(&inner_edge_node).unwrap();
                    assert!(upper_neighbors.len() == 1); // Otherwise it's not an inner edge
                    k1 = *prev_layer
                        .get(upper_neighbors.iter().last().unwrap())
                        .unwrap();
                }

                while l <= l1 {
                    // Consider all not yet considered nodes left of the current inner segment at layer i+1
                    if let Some(node_upper_neighbors) = reverse_edges.get(&layer_seq[l]) {
                        for upper_neighbor in node_upper_neighbors {
                            let k = *prev_layer.get(upper_neighbor).unwrap();
                            // k < k0: crosses the previous inner segment from left to right (going from layer i to i+1)
                            // k1 < k: crosses the current inner segment from right to left (going from layer i to i+1)
                            if k < k0 || k1 < k {
                                conflicts.insert((*upper_neighbor, layer_seq[l]));
                            }
                        }
                    }
                    l += 1;
                }
                k0 = k1;
            }
        }
    }
    conflicts
}
