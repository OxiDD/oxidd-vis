use std::{
    collections::{HashMap, HashSet, LinkedList},
    iter::FromIterator,
    marker::PhantomData,
};

use oxidd::{Edge, Function, InnerNode, LevelNo, Manager};
use oxidd_core::{DiagramRules, Tag};

use crate::{
    types::util::{
        drawing::{
            diagram_layout::{
                DiagramLayout, EdgeLayout, EdgePoint, NodeGroupLayout, Point, Transition,
            },
            layout_rules::LayoutRules,
        },
        edge_type::EdgeType,
        group_manager::{EdgeData, GroupManager},
        grouped_graph_structure::GroupedGraphStructure,
    },
    util::logging::console,
    wasm_interface::NodeGroupID,
};

///
/// A layout builder that takes another layout approach, and applies transitioning to it.
/// This will make layout changes smoothly transition from the previous state to the new state.
///
pub struct TransitionLayout<T: Tag, L: LayoutRules<T>> {
    layout: L,
    duration: u32,
    tag: PhantomData<T>,
}

impl<T: Tag, L: LayoutRules<T>> TransitionLayout<T, L> {
    pub fn new(layout: L) -> TransitionLayout<T, L> {
        TransitionLayout {
            layout,
            duration: 1000,
            tag: PhantomData,
        }
    }
}

impl<T: Tag, L: LayoutRules<T>> LayoutRules<T> for TransitionLayout<T, L> {
    fn layout(
        &mut self,
        graph: &GroupedGraphStructure<T>,
        old: &DiagramLayout<T>,
        time: u32,
    ) -> DiagramLayout<T> {
        let duration = self.duration;
        let old_time = time;
        let new = self.layout.layout(graph, old, time);

        fn get_per<T>(time: u32, val: Transition<T>) -> f32 {
            f32::max(
                0.0,
                f32::min(
                    (time as f32 - val.old_time as f32) / val.duration as f32,
                    1.0,
                ),
            )
        }
        let get_current_point = |val: Transition<Point>| {
            let per = get_per(time, val);
            Point {
                x: val.old.x * (1.0 - per) + val.new.x * per,
                y: val.old.y * (1.0 - per) + val.new.y * per,
            }
        };
        let get_current_float = |val: Transition<f32>| {
            let per = get_per(time, val);
            val.old * (1.0 - per) + val.new * per
        };

        let (node_mapping, edge_mapping) = relate_elements(graph, old, &new, &get_current_point);

        let map_edge = |from: NodeGroupID,
                        edge_data: &EdgeData<T>,
                        edge: &EdgeLayout,
                        old_group: &NodeGroupLayout<T>|
         -> EdgeLayout {
            let maybe_old_edge = edge_mapping.get(&(from, edge_data.clone())).and_then(
                |(old_from, old_edge_data)| {
                    old.groups.get(old_from).and_then(|group| {
                        group
                            .edges
                            .get(old_edge_data)
                            .map(|old_edge_layout| (old_edge_data, old_edge_layout))
                    })
                },
            );

            // let edge_type = edge_data.edge_type;
            if let Some((old_edge_data, old_edge_layout)) = maybe_old_edge {
                // Add all points needed for the new layout, and transition from any old points
                let mut new_points: Vec<EdgePoint> = edge
                    .points
                    .iter()
                    .enumerate()
                    .map(|(index, point)| {
                        let out_point = if index >= old_edge_layout.points.len() {
                            let to_pos = old.groups.get(&old_edge_data.to).unwrap().center_position;
                            Transition {
                                old: get_current_point(to_pos),
                                new: point.point.new,
                                duration,
                                old_time: time,
                            }
                        } else {
                            Transition {
                                old: get_current_point(
                                    old_edge_layout.points.get(index).unwrap().point,
                                ),
                                new: point.point.new,
                                duration,
                                old_time: time,
                            }
                        };

                        EdgePoint {
                            point: out_point,
                            exists: point.exists,
                        }
                    })
                    .collect();

                // Add any extra nodes needed to finish previous animation if needed
                new_points.extend(
                    old_edge_layout
                        .points
                        .iter()
                        .skip(edge.points.len())
                        .filter(|point| get_current_float(point.exists) > 0.0)
                        .map(|point| EdgePoint {
                            point: Transition {
                                duration,
                                old_time,
                                old: get_current_point(point.point),
                                new: new.groups.get(&edge_data.to).unwrap().center_position.new,
                            },
                            exists: Transition {
                                duration,
                                old_time,
                                old: get_current_float(point.exists),
                                new: 0.0,
                            },
                        }),
                );

                EdgeLayout {
                    start_offset: Transition {
                        duration,
                        old_time,
                        old: get_current_point(old_edge_layout.start_offset),
                        new: edge.start_offset.new,
                    },
                    end_offset: Transition {
                        duration,
                        old_time,
                        old: get_current_point(old_edge_layout.end_offset),
                        new: edge.end_offset.new,
                    },
                    points: new_points,
                    exists: edge.exists,
                    curve_offset: Transition {
                        duration,
                        old_time,
                        old: get_current_float(old_edge_layout.curve_offset),
                        new: edge.curve_offset.new,
                    },
                }
            } else {
                let points = edge
                    .points
                    .iter()
                    .map(|point| EdgePoint {
                        point: Transition {
                            duration,
                            old_time,
                            old: get_current_point(old_group.center_position),
                            new: point.point.new,
                        },
                        exists: point.exists,
                    })
                    .collect();
                EdgeLayout {
                    start_offset: edge.start_offset,
                    end_offset: edge.end_offset,
                    points: points,
                    exists: edge.exists,
                    curve_offset: edge.curve_offset,
                }
            }
        };

        DiagramLayout {
            groups: new
                .groups
                .iter()
                .map(|(id, group)| {
                    (
                        *id,
                        if let Some((offset, old_group)) =
                            node_mapping.get(&id).and_then(|(old_id, offset)| {
                                old.groups.get(old_id).map(|group| (offset, group))
                            })
                        {
                            NodeGroupLayout {
                                center_position: Transition {
                                    old_time,
                                    duration,
                                    old: get_current_point(old_group.center_position),
                                    new: group.center_position.new,
                                },
                                size: Transition {
                                    old_time,
                                    duration: duration,
                                    old: get_current_point(old_group.size),
                                    new: group.size.new,
                                },
                                label: group.label.clone(),
                                exists: Transition {
                                    old_time,
                                    duration,
                                    old: get_current_float(old_group.exists),
                                    new: group.exists.new,
                                },
                                edges: group
                                    .edges
                                    .iter()
                                    .map(|(edge_data, edge)| {
                                        (
                                            edge_data.clone(),
                                            map_edge(*id, edge_data, edge, old_group),
                                        )
                                    })
                                    .collect(),
                                nodes: group.nodes.clone(),
                            }
                        } else {
                            group.clone()
                        },
                    )
                })
                .collect(),
            layers: new.layers,
        }
    }
}

fn relate_elements<T: Tag>(
    graph: &dyn GroupedGraphStructure<T>,
    old: &DiagramLayout<T>,
    new: &DiagramLayout<T>,
    get_current_point: &impl Fn(Transition<Point>) -> Point,
) -> (
    /* A mapping from a node to an old node + offset */
    HashMap<NodeGroupID, (NodeGroupID, Point)>,
    /* A mapping from an edge (including source node) to another edge */
    HashMap<(NodeGroupID, EdgeData<T>), (NodeGroupID, EdgeData<T>)>,
) {
    let old_to_edges = get_edge_lookup(old, false);
    let old_reverse_edges = get_edge_lookup(old, true);

    let mut edge_mapping: HashMap<(NodeGroupID, EdgeData<T>), (NodeGroupID, EdgeData<T>)> =
        HashMap::new();

    // Perform trivial mapping
    let mut node_mapping: HashMap<NodeGroupID, (NodeGroupID, Point)> = new
        .groups
        .iter()
        .filter_map(|(&group_id, data)| {
            if old.groups.contains_key(&group_id) {
                Some((group_id, (group_id, Point { x: 0., y: 0. })))
            } else {
                None
            }
        })
        .collect();

    // Perform search from this mapping
    let mut queue = LinkedList::from_iter(node_mapping.iter().map(|(&old, &(new, _))| (old, new)));
    while let Some((old_node, new_node)) = queue.pop_front() {
        // Follow and link child edges
        for edge in graph.get_children(new_node) {
            if let Some(&(old_to, _)) = node_mapping.get(&edge.to) {
                edge_mapping.insert(
                    (new_node, edge.drop_count()),
                    (
                        old_node,
                        EdgeData {
                            to: old_to,
                            from_level: edge.from_level,
                            to_level: edge.to_level,
                            edge_type: edge.edge_type,
                        },
                    ),
                );
                continue;
            }

            if let Some(to_old) =
                old_to_edges.get(&(old_node, edge.from_level, edge.edge_type, edge.to_level))
            {
                if !can_come_from(old, new, *to_old, edge.to) {
                    continue;
                }

                let old_edge_data = EdgeData {
                    to: *to_old,
                    from_level: edge.from_level,
                    to_level: edge.to_level,
                    edge_type: edge.edge_type,
                };
                let offset = old
                    .groups
                    .get(&old_node)
                    .and_then(|group| {
                        group
                            .edges
                            .get(&old_edge_data)
                            .map(|layout| get_current_point(layout.end_offset))
                    })
                    .unwrap_or_else(|| Point { x: 0., y: 0. });
                node_mapping.insert(edge.to, (*to_old, offset));
                edge_mapping.insert((new_node, edge.drop_count()), (old_node, old_edge_data));
                queue.push_back((*to_old, edge.to));
            }
        }

        // Follow and link parent edges
        for reverse_edge in graph.get_parents(new_node) {
            let old_edge_data = EdgeData {
                to: old_node,
                from_level: reverse_edge.to_level,
                to_level: reverse_edge.from_level,
                edge_type: reverse_edge.edge_type,
            };
            let new_edge_data = EdgeData {
                to: new_node,
                from_level: reverse_edge.to_level,
                to_level: reverse_edge.from_level,
                edge_type: reverse_edge.edge_type,
            };
            if let Some(&(old_from, _)) = node_mapping.get(&reverse_edge.to) {
                edge_mapping.insert((reverse_edge.to, new_edge_data), (old_from, old_edge_data));
                continue;
            }

            if let Some(from_old) = old_reverse_edges.get(&(
                old_node,
                reverse_edge.from_level,
                reverse_edge.edge_type,
                reverse_edge.to_level,
            )) {
                if !can_come_from(old, new, *from_old, reverse_edge.to) {
                    continue;
                }

                let offset = old
                    .groups
                    .get(&from_old)
                    .and_then(|group| {
                        group
                            .edges
                            .get(&old_edge_data)
                            .map(|layout| get_current_point(layout.start_offset))
                    })
                    .unwrap_or_else(|| Point { x: 0., y: 0. });
                node_mapping.insert(reverse_edge.to, (*from_old, offset));
                edge_mapping.insert((reverse_edge.to, new_edge_data), (*from_old, old_edge_data));
                queue.push_back((*from_old, reverse_edge.to));
            }
        }
    }

    (node_mapping, edge_mapping)
}

fn can_come_from<T: Tag>(
    old: &DiagramLayout<T>,
    new: &DiagramLayout<T>,
    old_group: NodeGroupID,
    new_group: NodeGroupID,
) -> bool {
    let Some(old_nodes) = old.groups.get(&old_group).map(|group| &group.nodes) else {
        return false;
    };
    let Some(new_nodes) = new.groups.get(&new_group).map(|group| &group.nodes) else {
        return false;
    };

    // TODO: add other conditions here later to handle animations related to
    return new_nodes.is_subset(&old_nodes) || new_nodes.is_superset(&old_nodes);
}

fn get_edge_lookup<T: Tag>(
    layout: &DiagramLayout<T>,
    reverse: bool,
) -> HashMap<(NodeGroupID, LevelNo, EdgeType<T>, LevelNo), NodeGroupID> {
    layout
        .groups
        .iter()
        .flat_map(|(&from_group_id, group)| {
            group.edges.keys().map(move |edge| {
                if reverse {
                    (
                        (edge.to, edge.to_level, edge.edge_type, edge.from_level),
                        (from_group_id),
                    )
                } else {
                    (
                        (
                            from_group_id,
                            edge.from_level,
                            edge.edge_type,
                            edge.to_level,
                        ),
                        (edge.to),
                    )
                }
            })
        })
        .collect()
}
