use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use oxidd::NodeID;
use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::{
            diagram_layout::{
                DiagramLayout, EdgeLayout, EdgePoint, NodeGroupLayout, Point, Transition,
            },
            layout_rules::LayoutRules,
        },
        edge_type::EdgeType,
        grouped_graph_structure::GroupedGraphStructure,
    },
    util::logging::console,
    wasm_interface::NodeGroupID,
};

use super::util::layered::layer_orderer::{EdgeMap, Order};

/// The trait used to decide what ordering of nodes to use in the layout, including dummy nodes
pub trait LayerOrdering<T: Tag> {
    fn order_nodes(
        &self,
        graph: &dyn GroupedGraphStructure<T>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        // The ID such that any greater ID represents a dummy vertex
        dummy_start_id: NodeGroupID,
    ) -> Vec<Order>;
}

/// The trait used to decide what positioning of nodes to use in the layout for the given node orders, including dummy nodes
pub trait NodePositioning<T: Tag> {
    fn position_nodes(
        &self,
        graph: &dyn GroupedGraphStructure<T>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        // The ID such that any greater ID represents a dummy vertex
        dummy_start_id: NodeGroupID,
    ) -> HashMap<NodeGroupID, Point>;
}

pub struct LayeredLayout<T: Tag> {
    ordering: Box<dyn LayerOrdering<T>>,
    positioning: Box<dyn NodePositioning<T>>,
}

impl<T: Tag> LayeredLayout<T> {
    pub fn new(
        ordering: Box<dyn LayerOrdering<T>>,
        positioning: Box<dyn NodePositioning<T>>,
    ) -> LayeredLayout<T> {
        LayeredLayout {
            ordering,
            positioning,
        }
    }
}

impl<T: Tag> LayoutRules<T> for LayeredLayout<T> {
    fn layout(
        &mut self,
        graph: &GroupedGraphStructure<T>,
        old: &DiagramLayout<T>,
        time: u32,
    ) -> DiagramLayout<T> {
        let mut layers: Vec<Order> = Vec::new();
        let mut add_to_layer = |layer: usize, id: NodeGroupID| {
            while layer >= layers.len() {
                layers.push(HashMap::new());
            }
            let layer = layers.get_mut(layer).unwrap();
            layer.insert(id, layer.len());
        };

        // Add nodes
        let mut next_free_id = 0;
        for group in graph.get_all_groups() {
            let (start, end) = graph.get_level_range(group);
            for layer in start..=end {
                add_to_layer(layer as usize, group);
            }
            if group >= next_free_id {
                next_free_id = group + 1;
            }
        }
        let dummy_start_id = next_free_id;

        // Add dummy vertices and edges
        let mut edges: HashMap<NodeGroupID, HashSet<NodeGroupID>> = HashMap::new();
        let mut add_to_edges = |from: NodeGroupID, to: NodeGroupID| {
            edges
                .entry(from)
                .or_insert_with(|| HashSet::new())
                .insert(to);
        };
        let mut edge_bend_nodes: HashMap<
            (NodeGroupID, EdgeType<T>, NodeGroupID),
            Vec<NodeGroupID>,
        > = HashMap::new();

        for group in graph.get_all_groups() {
            let (start_, start) = graph.get_level_range(group);
            for (edge_type, to_group, _) in graph.get_children(group) {
                let mut prev = group;
                let (end, end_) = graph.get_level_range(to_group);
                let mut bends = Vec::new();

                // let delta = (start + 1) as i32 - end as i32;
                // if delta != 0 {
                //     console::log!(
                //         "({} {}) ({} {}), {} {}",
                //         start_,
                //         start,
                //         end,
                //         end_,
                //         graph
                //             .get_nodes_of_group(group)
                //             .collect::<Vec<NodeID>>()
                //             .get(0)
                //             .unwrap(),
                //         graph
                //             .get_nodes_of_group(to_group)
                //             .collect::<Vec<NodeID>>()
                //             .get(0)
                //             .unwrap()
                //     );
                // }
                for layer in (start + 1)..end {
                    let id = next_free_id;
                    next_free_id += 1;

                    bends.push(id);
                    add_to_layer(layer as usize, id);
                    add_to_edges(prev, id);
                    prev = id;
                }
                edge_bend_nodes.insert((group, edge_type, to_group), bends);
                add_to_edges(prev, to_group);
            }
        }

        // Perform node positioning
        let layers = self
            .ordering
            .order_nodes(graph, &layers, &edges, dummy_start_id);
        let node_positions =
            self.positioning
                .position_nodes(graph, &layers, &edges, dummy_start_id);

        // TODO: remove straight edge bendpoints

        // Map to a diagram layout
        DiagramLayout {
            // TODO:
            layers: HashMap::new(),
            // TODO: cleanup waterfalls
            groups: graph
                .get_all_groups()
                .iter()
                .map(|&group_id| {
                    (
                        group_id,
                        NodeGroupLayout {
                            label: group_id.to_string(),
                            center_position: {
                                let (s, e) = graph.get_level_range(group_id);
                                console::log!(
                                    "{}, [{}], ({}, {})",
                                    group_id,
                                    graph
                                        .get_nodes_of_group(group_id)
                                        .map(|id| id.to_string())
                                        .join(", "),
                                    s,
                                    e
                                );
                                Transition::plain(*node_positions.get(&group_id).unwrap())
                            },
                            size: Transition::plain(Point { x: 1., y: 1. }),
                            exists: Transition::plain(1.),
                            edges: graph
                                .get_children(group_id)
                                .group_by(|(_, to, _)| *to)
                                .into_iter()
                                .map(|(to, edges)| {
                                    (
                                        to,
                                        edges
                                            .map(|(edge_type, _, _)| {
                                                (
                                                    edge_type,
                                                    EdgeLayout {
                                                        points: edge_bend_nodes
                                                            .get(&(group_id, edge_type, to))
                                                            .map_or_else(
                                                                || Vec::new(),
                                                                |nodes| {
                                                                    nodes
                                                                        .iter()
                                                                        .map(|dummy_id| EdgePoint {
                                                                            point: Transition::plain(
                                                                                *node_positions
                                                                                    .get(&dummy_id)
                                                                                    .unwrap(),
                                                                            ),
                                                                            exists: Transition::plain(
                                                                                1.,
                                                                            ),
                                                                        })
                                                                        .collect()
                                                                },
                                                            ),
                                                        exists: Transition::plain(1.),
                                                    },
                                                )
                                            })
                                            .collect(),
                                    )
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
            }
    }
}
