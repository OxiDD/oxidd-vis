use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    marker::PhantomData,
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
        group_manager::EdgeData,
        grouped_graph_structure::{EdgeCountData, GroupedGraphStructure, SourceReader},
    },
    util::{logging::console, rectangle::Rectangle},
    wasm_interface::NodeGroupID,
};

use super::util::{
    compute_layers_layout::compute_layers_layout,
    layered::layer_orderer::{EdgeMap, Order},
    remove_redundant_bendpoints::remove_redundant_bendpoints,
};

/// The trait used to decide what ordering of nodes to use in the layout, including dummy nodes
pub trait LayerOrdering<T: Tag> {
    fn order_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T>,
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
        graph: &impl GroupedGraphStructure<T>,
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
        graph: &impl GroupedGraphStructure<T>,
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

pub struct LayeredLayout<
    T: Tag,
    O: LayerOrdering<T>,
    G: LayerGroupSorting<T>,
    P: NodePositioning<T>,
> {
    ordering: O,
    group_aligning: G,
    positioning: P,
    max_curve_offset: f32,
    tag: PhantomData<T>,
}

impl<T: Tag, O: LayerOrdering<T>, G: LayerGroupSorting<T>, P: NodePositioning<T>>
    LayeredLayout<T, O, G, P>
{
    pub fn new(
        ordering: O,
        group_aligning: G,
        positioning: P,
        max_curve_offset: f32,
    ) -> LayeredLayout<T, O, G, P> {
        LayeredLayout {
            ordering,
            group_aligning,
            positioning,
            max_curve_offset,
            tag: PhantomData,
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

impl<
        T: Tag,
        O: LayerOrdering<T>,
        S: LayerGroupSorting<T>,
        P: NodePositioning<T>,
        G: GroupedGraphStructure<T>,
    > LayoutRules<T, G> for LayeredLayout<T, O, S, P>
{
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T>,
        sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T> {
        // Setup the layers and edges, and a way of adding o them
        let mut layers: Vec<Order> = Vec::new();
        let mut edges: HashMap<NodeGroupID, HashSet<NodeGroupID>> = HashMap::new();

        let mut dummy_owners: HashMap<NodeGroupID, NodeGroupID> = HashMap::new();
        let mut next_free_id = 0; // uninitialized, will be initialized by add_groups

        let (dummy_group_start_id, group_layers) = add_groups_with_dummies(
            graph,
            &mut layers,
            &mut edges,
            &mut dummy_owners,
            &mut next_free_id,
        );
        let dummy_edge_start_id = next_free_id;

        let (edge_bend_nodes, edge_connection_nodes) = add_edges_with_dummies(
            graph,
            &mut layers,
            &mut edges,
            &mut dummy_owners,
            &group_layers,
            &mut next_free_id,
        );

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

        let (node_positions, layer_positions) = self.positioning.position_nodes(
            graph,
            &layers,
            &edges,
            dummy_group_start_id,
            dummy_edge_start_id,
            &dummy_owners,
        );

        format_layout(
            graph,
            self.max_curve_offset,
            node_positions,
            layer_positions,
            edge_bend_nodes,
            edge_connection_nodes,
            dummy_group_start_id,
        )
    }
}

fn add_to_layer(layers: &mut Vec<Order>, layer: usize, id: NodeGroupID) {
    while layer >= layers.len() {
        layers.push(HashMap::new());
    }
    let layer = layers.get_mut(layer).unwrap();
    layer.insert(id, layer.len());
}

fn add_to_edges(edges: &mut EdgeMap, from: NodeGroupID, to: NodeGroupID) {
    edges
        .entry(from)
        .or_insert_with(|| HashSet::new())
        .insert(to);
}

fn add_groups_with_dummies<T: Tag>(
    graph: &impl GroupedGraphStructure<T>,
    layers: &mut Vec<Order>,
    edges: &mut EdgeMap,
    dummy_owners: &mut HashMap<NodeGroupID, NodeGroupID>,
    next_free_id: &mut NodeGroupID,
) -> (NodeGroupID, HashMap<NodeGroupID, HashMap<u32, usize>>) {
    let mut group_layers: HashMap<NodeGroupID, HashMap<u32, usize>> = HashMap::new();
    for group in graph.get_all_groups() {
        let (start, end) = graph.get_level_range(group);
        add_to_layer(layers, start as usize, group);
        group_layers.insert(group, HashMap::from([(start, group)]));
        if group >= *next_free_id {
            *next_free_id = group + 1;
        }
    }
    let dummy_group_start_id = *next_free_id;

    for group in graph.get_all_groups() {
        let (start, end) = graph.get_level_range(group);
        dummy_owners.insert(group, group);
        let mut prev = group;
        for layer in start + 1..=end {
            let layer_group_id = *next_free_id;
            *next_free_id += 1;
            add_to_edges(edges, prev, layer_group_id);
            dummy_owners.insert(layer_group_id, group);
            add_to_layer(layers, layer as usize, layer_group_id);
            group_layers
                .entry(group)
                .or_default()
                .insert(layer, layer_group_id);
            prev = layer_group_id;
        }
    }

    (dummy_group_start_id, group_layers)
}

fn add_edges_with_dummies<T: Tag>(
    graph: &impl GroupedGraphStructure<T>,
    layers: &mut Vec<Order>,
    edges: &mut EdgeMap,
    dummy_owners: &mut HashMap<NodeGroupID, NodeGroupID>,
    group_layers: &HashMap<NodeGroupID, HashMap<u32, usize>>,
    next_free_id: &mut NodeGroupID,
) -> (
    HashMap<(NodeGroupID, EdgeData<T>), Vec<NodeGroupID>>,
    HashMap<(NodeGroupID, EdgeData<T>), (NodeGroupID, NodeGroupID)>,
) {
    let mut edge_bend_nodes: HashMap<(NodeGroupID, EdgeData<T>), Vec<NodeGroupID>> = HashMap::new();
    let mut edge_connection_nodes: HashMap<(NodeGroupID, EdgeData<T>), (NodeGroupID, NodeGroupID)> =
        HashMap::new();

    console::log!("---------------");
    for group in graph.get_all_groups() {
        // let (parent_start_level, parent_end_level) = graph.get_level_range(group);

        for EdgeCountData {
            to: to_group,
            from_level: edge_start_level,
            to_level: edge_end_level,
            edge_type,
            count: _,
        } in graph.get_children(group)
        {
            let edge_data = EdgeData::new(to_group, edge_start_level, edge_end_level, edge_type);

            let group_connection = group_layers
                .get(&group)
                .unwrap()
                .get(&edge_start_level)
                .unwrap();

            let mut prev = *group_connection;
            let mut bends = Vec::new();
            let first_bend_id = *next_free_id;

            for layer in (edge_start_level + 1)..edge_end_level {
                let id = *next_free_id;
                *next_free_id += 1;
                dummy_owners.insert(id, first_bend_id);
                bends.push(id);
                add_to_layer(layers, layer as usize, id);
                add_to_edges(edges, prev, id);
                prev = id;
            }
            edge_bend_nodes.insert((group, edge_data.clone()), bends);

            let to_group_connection = *group_layers
                .get(&to_group)
                .unwrap()
                .get(&edge_end_level)
                .unwrap();
            edge_connection_nodes
                .insert((group, edge_data), (*group_connection, to_group_connection));
            add_to_edges(edges, prev, to_group_connection);
        }
    }

    (edge_bend_nodes, edge_connection_nodes)
}

fn format_layout<T: Tag>(
    graph: &impl GroupedGraphStructure<T>,
    max_curve_offset: f32,
    node_positions: HashMap<usize, Point>,
    layer_positions: HashMap<LevelNo, f32>,
    edge_bend_nodes: HashMap<(NodeGroupID, EdgeData<T>), Vec<NodeGroupID>>,
    edge_connection_nodes: HashMap<(NodeGroupID, EdgeData<T>), (NodeGroupID, NodeGroupID)>,
    dummy_group_start_id: usize,
) -> DiagramLayout<T> {
    let node_size = 1.; // TODO: make configurable
    let node_size_shift = -0.5
        * Point {
            x: node_size,
            y: node_size,
        };
    let node_positions: HashMap<usize, Point> = node_positions
        .iter()
        .map(|(&group_id, &pos)| (group_id, pos + node_size_shift))
        .collect();
    let bottom_node_positions: HashMap<usize, Point> = node_positions
        .iter()
        .map(|(&group_id, pos)| {
            (
                group_id,
                (if group_id >= dummy_group_start_id {
                    *pos
                } else {
                    let (s, e) = graph.get_level_range(group_id);
                    Point {
                        x: pos.x,
                        y: pos.y
                            - (layer_positions.get(&s).unwrap_or(&0.)
                                - layer_positions.get(&e).unwrap_or(&0.)),
                    }
                }),
            )
        })
        .collect();

    // Map to a diagram layout
    DiagramLayout {
        layers: compute_layers_layout(
            graph,
            node_positions
                .iter()
                .filter(|(&group_id, _)| group_id < dummy_group_start_id)
                .map(|(&group_id, pos)| {
                    let (s, e) = graph.get_level_range(group_id);

                    let start_layer_y = layer_positions.get(&s).unwrap_or(&0.);
                    let prev_layer_y = (if s > 0 {
                        layer_positions.get(&(s - 1)).cloned()
                    } else {
                        None
                    })
                    .unwrap_or(start_layer_y + 2. * node_size);
                    let start_y = (start_layer_y + prev_layer_y) / 2.0;

                    let end_layer_y = *layer_positions.get(&e).unwrap_or(&0.);
                    let next_layer_y = layer_positions
                        .get(&(e + 1))
                        .cloned()
                        .unwrap_or(end_layer_y - 2. * node_size);
                    let end_y = (end_layer_y + next_layer_y) / 2.0;
                    (group_id, Rectangle::new(0., end_y, 0., start_y - end_y))
                }),
        ),
        groups: graph
            .get_all_groups()
            .iter()
            .map(|&group_id| {
                let (s, e) = graph.get_level_range(group_id);
                (
                    group_id,
                    NodeGroupLayout {
                        label: group_id.to_string(),
                        position: Transition::plain(*bottom_node_positions.get(&group_id).unwrap()),
                        size: Transition::plain(Point {
                            x: node_size,
                            y: node_size
                                + (layer_positions.get(&s).unwrap_or(&0.)
                                    - layer_positions.get(&e).unwrap_or(&0.))
                                    * node_size,
                        }),
                        exists: Transition::plain(1.),
                        edges: graph
                            .get_children(group_id)
                            .into_iter()
                            .group_by(
                                |&EdgeCountData {
                                     to,
                                     from_level,
                                     to_level,
                                     edge_type,
                                     count,
                                 }| (to, from_level, to_level),
                            )
                            .into_iter()
                            .flat_map(|(group, edge_datas)| {
                                let edge_datas = edge_datas.collect_vec();
                                let len = edge_datas.len();
                                edge_datas
                                    .iter()
                                    .enumerate()
                                    .map(|(index, edge_data)| {
                                        (
                                            edge_data.drop_count(),
                                            format_edge(
                                                &edge_data,
                                                if len > 1 {
                                                    ((index as f32 / (len - 1) as f32) - 0.5)
                                                        * 2.0
                                                        * max_curve_offset
                                                } else {
                                                    0.
                                                },
                                                group_id,
                                                &node_positions,
                                                &bottom_node_positions,
                                                &edge_bend_nodes,
                                                &edge_connection_nodes,
                                                node_size,
                                            ),
                                        )
                                    })
                                    .collect_vec()
                            })
                            .collect(),
                    },
                )
            })
            .collect(),
    }
}

fn format_edge<T: Tag>(
    edge: &EdgeCountData<T>,
    curve_offset: f32,
    group_id: NodeGroupID,
    node_positions: &HashMap<usize, Point>,
    bottom_node_positions: &HashMap<usize, Point>,
    edge_bend_nodes: &HashMap<(NodeGroupID, EdgeData<T>), Vec<NodeGroupID>>,
    edge_connection_nodes: &HashMap<(NodeGroupID, EdgeData<T>), (NodeGroupID, NodeGroupID)>,
    node_size: f32,
) -> EdgeLayout {
    let EdgeCountData {
        to,
        from_level,
        to_level,
        edge_type: _,
        count: _,
    } = edge;
    let edge_data = edge.drop_count();

    let (start_offset, end_offset) = edge_connection_nodes
        .get(&(group_id, edge_data.clone()))
        .map_or_else(
            || (Point { x: 0.0, y: 0.0 }, Point { x: 0.0, y: 0.0 }),
            |(start_id, end_id)| {
                (
                    node_positions.get(&start_id).map_or_else(
                        || Point { x: 0.0, y: 0.0 },
                        |start_point| {
                            bottom_node_positions.get(&group_id).map_or_else(
                                || Point { x: 0., y: 0. },
                                |center_point| *start_point - *center_point,
                            )
                        },
                    ),
                    node_positions.get(&end_id).map_or_else(
                        || Point { x: 0.0, y: 0.0 },
                        |end_point| {
                            bottom_node_positions.get(&to).map_or_else(
                                || Point { x: 0., y: 0. },
                                |center_point| *end_point - *center_point,
                            )
                        },
                    ),
                )
            },
        );

    let edge_offset = Point {
        x: node_size,
        y: node_size,
    } * 0.5;

    EdgeLayout {
        start_offset: Transition::plain(start_offset + edge_offset),
        end_offset: Transition::plain(end_offset + edge_offset),
        points: edge_bend_nodes.get(&(group_id, edge_data)).map_or_else(
            || Vec::new(),
            |nodes| {
                remove_redundant_bendpoints(
                    &nodes
                        .iter()
                        .map(|dummy_id| *node_positions.get(&dummy_id).unwrap() + edge_offset)
                        .collect(),
                )
                .iter()
                .map(|&point| EdgePoint {
                    point: Transition::plain(point),
                    exists: Transition::plain(1.),
                })
                .collect()
            },
        ),
        exists: Transition::plain(1.),
        curve_offset: Transition::plain(curve_offset),
    }
}
