use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use itertools::Itertools;
use num_rational::Ratio;
use oxidd::{LevelNo, NodeID};
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
        // The ID such that any ID in the range [dummy_group_start_id, dummy_edge_start_id) represents a dummy node of a group
        dummy_group_start_id: NodeGroupID,
        // The ID such that any ID greater or equal represents a dummy node of an edge
        dummy_edge_start_id: NodeGroupID,
        // The owner of a given dummy node, such that multiple nodes derived from the same data can be considered as a group
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order>;
}

/// The trait used to decide what positioning of nodes to use in the layout for the given node orders, including dummy nodes
pub trait NodePositioning<T: Tag> {
    fn position_nodes(
        &self,
        graph: &dyn GroupedGraphStructure<T>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        // The ID such that any ID in the range [dummy_group_start_id, dummy_edge_start_id) represents a dummy node of a group
        dummy_group_start_id: NodeGroupID,
        // The ID such that any ID greater or equal represents a dummy node of an edge
        dummy_edge_start_id: NodeGroupID,
        // The owner of a given dummy node, such that multiple nodes derived from the same data can be considered as a group
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> (HashMap<NodeGroupID, Point>, HashMap<LevelNo, f32>);
}

/// The trait used to decide what positioning of nodes to use in the layout for the given node orders, including dummy nodes
pub trait LayerGroupSorting<T: Tag> {
    fn align_cross_layer_nodes(
        &self,
        graph: &dyn GroupedGraphStructure<T>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        // The ID such that any ID in the range [dummy_group_start_id, dummy_edge_start_id) represents a dummy node of a group
        dummy_group_start_id: NodeGroupID,
        // The ID such that any ID greater or equal represents a dummy node of an edge
        dummy_edge_start_id: NodeGroupID,
        // The owner of a given dummy node, such that multiple nodes derived from the same data can be considered as a group
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order>;
}

pub struct LayeredLayout<T: Tag> {
    ordering: Box<dyn LayerOrdering<T>>,
    group_aligning: Box<dyn LayerGroupSorting<T>>,
    positioning: Box<dyn NodePositioning<T>>,
}

impl<T: Tag> LayeredLayout<T> {
    pub fn new(
        ordering: Box<dyn LayerOrdering<T>>,
        group_aligning: Box<dyn LayerGroupSorting<T>>,
        positioning: Box<dyn NodePositioning<T>>,
    ) -> LayeredLayout<T> {
        LayeredLayout {
            ordering,
            group_aligning,
            positioning,
        }
    }
}

pub fn is_group_dummy(
    node: NodeGroupID,
    dummy_group_start_id: NodeGroupID,
    dummy_edge_start_id: NodeGroupID,
) -> bool {
    node >= dummy_group_start_id && node < dummy_edge_start_id
}
pub fn is_edge_dummy(node: NodeGroupID, dummy_edge_start_id: NodeGroupID) -> bool {
    node >= dummy_edge_start_id
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
        let mut edges: HashMap<NodeGroupID, HashSet<NodeGroupID>> = HashMap::new();
        let mut add_to_edges = |from: NodeGroupID, to: NodeGroupID| {
            edges
                .entry(from)
                .or_insert_with(|| HashSet::new())
                .insert(to);
        };

        // Add nodes
        let mut next_free_id = 0;
        let mut group_layers: HashMap<NodeGroupID, HashMap<u32, usize>> = HashMap::new();
        for group in graph.get_all_groups() {
            let (start, end) = graph.get_level_range(group);
            add_to_layer(start as usize, group);
            group_layers.insert(group, HashMap::from([(start, group)]));
            // for layer in start..=end {
            //     add_to_layer(layer as usize, group);
            // }
            if group >= next_free_id {
                next_free_id = group + 1;
            }
        }
        let dummy_group_start_id = next_free_id;

        let mut dummy_owners: HashMap<NodeGroupID, NodeGroupID> = HashMap::new();
        let mut non_end_group_nodes: HashSet<NodeGroupID> = HashSet::new(); // The group nodes (including dummy ones) that are not the end of the group
        for group in graph.get_all_groups() {
            let (start, end) = graph.get_level_range(group);
            dummy_owners.insert(group, group);
            let mut prev = group;
            for layer in start + 1..=end {
                non_end_group_nodes.insert(prev);
                let layer_group_id = next_free_id;
                next_free_id += 1;
                add_to_edges(prev, layer_group_id);
                dummy_owners.insert(layer_group_id, group);
                add_to_layer(layer as usize, layer_group_id);
                group_layers
                    .entry(group)
                    .or_default()
                    .insert(layer, layer_group_id);
                prev = layer_group_id;
            }
        }

        let dummy_edge_start_id = next_free_id;

        // Add dummy vertices and edges
        let mut edge_bend_nodes: HashMap<
            (NodeGroupID, EdgeType<T>, NodeGroupID),
            Vec<NodeGroupID>,
        > = HashMap::new();
        let mut edge_connection_nodes: HashMap<
            (NodeGroupID, EdgeType<T>, NodeGroupID),
            (NodeGroupID, NodeGroupID),
        > = HashMap::new();

        console::log!("---------------");
        for group in graph.get_all_groups() {
            let (parent_start_level, parent_end_level) = graph.get_level_range(group);

            for (edge_type, to_group, _) in graph.get_children(group) {
                let (start_level, end_level) = graph.get_level_range(to_group);

                // Stylistic choice for how edges should span between two groups that cross multiple layers
                // TODO: make these layers based on the actual node layer that the edge is coming from
                let (edge_start_level, edge_end_level) = if parent_end_level < start_level {
                    console::log!("> 0 {} {}", parent_end_level, start_level);
                    (parent_end_level, start_level)
                } else if parent_start_level < start_level {
                    console::log!("> 1 {} {}", start_level - 1, start_level);
                    (start_level - 1, start_level)
                } else if parent_end_level < end_level {
                    console::log!("> 2 {} {}", parent_end_level, parent_end_level + 1);
                    (parent_end_level, parent_end_level + 1)
                } else if parent_start_level < end_level {
                    console::log!("> 3 {} {}", parent_start_level, parent_start_level + 1);
                    (parent_start_level, parent_start_level + 1)
                } else {
                    panic!("The child group was somehow fully present above the parent\n parent: ({}, {}); child: ({}, {})", parent_start_level, parent_end_level, start_level, end_level);
                };

                let group_connection = group_layers
                    .get(&group)
                    .unwrap()
                    .get(&edge_start_level)
                    .unwrap();

                let mut prev = *group_connection;
                let mut bends = Vec::new();
                let first_bend_id = next_free_id;
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
                for layer in (edge_start_level + 1)..edge_end_level {
                    let id = next_free_id;
                    next_free_id += 1;
                    dummy_owners.insert(id, first_bend_id);
                    bends.push(id);
                    add_to_layer(layer as usize, id);
                    add_to_edges(prev, id);
                    prev = id;
                }
                edge_bend_nodes.insert((group, edge_type, to_group), bends);

                let to_group_connection = *group_layers
                    .get(&to_group)
                    .unwrap()
                    .get(&edge_end_level)
                    .unwrap();
                edge_connection_nodes.insert(
                    (group, edge_type, to_group),
                    (*group_connection, to_group_connection),
                );
                add_to_edges(prev, to_group_connection);
            }
        }

        // Perform node positioning
        let layers = self.ordering.order_nodes(
            graph,
            &layers,
            &edges,
            dummy_group_start_id,
            dummy_edge_start_id,
            &dummy_owners,
        );

        let layers = self.group_aligning.align_cross_layer_nodes(
            graph,
            &layers,
            &edges,
            dummy_group_start_id,
            dummy_edge_start_id,
            &dummy_owners,
        );
        edges = edges
            .iter()
            .map(|(&node, edges)| {
                (
                    node,
                    if non_end_group_nodes.contains(&node) {
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
            .collect();
        console::log!("g, e; {} {}", dummy_group_start_id, dummy_edge_start_id);
        let (node_positions, layer_positions) = self.positioning.position_nodes(
            graph,
            &layers,
            &edges,
            dummy_group_start_id,
            dummy_edge_start_id,
            &dummy_owners,
        );

        let centered_node_positions: HashMap<usize, Point> = node_positions
            .iter()
            .map(|(&group_id, pos)| {
                (
                    group_id,
                    if group_id >= dummy_group_start_id {
                        *pos
                    } else {
                        let (s, e) = graph.get_level_range(group_id);
                        Point {
                            x: pos.x,
                            y: pos.y
                                - (layer_positions.get(&e).unwrap_or(&0.)
                                    - layer_positions.get(&s).unwrap_or(&0.))
                                    / 2.,
                        }
                    },
                )
            })
            .collect();
        // TODO: remove straight edge bendpoints

        let map_edges = |edges: &mut dyn Iterator<Item = (EdgeType<T>, usize, i32)>,
                         group_id: NodeGroupID,
                         to: NodeGroupID| {
            edges
                .map(|(edge_type, _, _)| {
                    let (start_offset, end_offset) = edge_connection_nodes
                        .get(&(group_id, edge_type, to))
                        .map_or_else(
                            || (Point { x: 0.0, y: 0.0 }, Point { x: 0.0, y: 0.0 }),
                            |(start_id, end_id)| {
                                (
                                    node_positions.get(&start_id).map_or_else(
                                        || Point { x: 0.0, y: 0.0 },
                                        |start_point| {
                                            centered_node_positions.get(&group_id).map_or_else(
                                                || Point { x: 0., y: 0. },
                                                |center_point| *start_point - *center_point,
                                            )
                                        },
                                    ),
                                    node_positions.get(&end_id).map_or_else(
                                        || Point { x: 0.0, y: 0.0 },
                                        |end_point| {
                                            centered_node_positions.get(&to).map_or_else(
                                                || Point { x: 0., y: 0. },
                                                |center_point| *end_point - *center_point,
                                            )
                                        },
                                    ),
                                )
                            },
                        );

                    (
                        edge_type,
                        EdgeLayout {
                            start_offset: Transition::plain(start_offset),
                            end_offset: Transition::plain(end_offset),
                            points: edge_bend_nodes.get(&(group_id, edge_type, to)).map_or_else(
                                || Vec::new(),
                                |nodes| {
                                    nodes
                                        .iter()
                                        .map(|dummy_id| EdgePoint {
                                            point: Transition::plain(
                                                *node_positions.get(&dummy_id).unwrap(),
                                            ),
                                            exists: Transition::plain(1.),
                                        })
                                        .collect()
                                },
                            ),
                            exists: Transition::plain(1.),
                        },
                    )
                })
                .collect()
        };

        // Map to a diagram layout
        DiagramLayout {
            // TODO:
            layers: HashMap::new(),
            // TODO: cleanup waterfalls
            groups: graph
                .get_all_groups()
                .iter()
                .map(|&group_id| {
                    let (s, e) = graph.get_level_range(group_id);
                    (
                        group_id,
                        NodeGroupLayout {
                            label: group_id.to_string(),
                            center_position: {
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
                                Transition::plain(*centered_node_positions.get(&group_id).unwrap())
                            },
                            size: Transition::plain(Point {
                                x: 1.,
                                y: 1.
                                    + (layer_positions.get(&e).unwrap_or(&0.)
                                        - layer_positions.get(&s).unwrap_or(&0.)),
                            }),
                            exists: Transition::plain(1.),
                            edges: graph
                                .get_children(group_id)
                                .group_by(|(_, to, _)| *to)
                                .into_iter()
                                .map(|(to, mut edges)| (to, map_edges(&mut edges, group_id, to)))
                                .collect(),
                        },
                    )
                })
                .collect(),
        }
    }
}
