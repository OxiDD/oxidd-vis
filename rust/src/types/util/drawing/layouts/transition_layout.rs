use std::collections::HashMap;

use oxidd::{Edge, Function, InnerNode, Manager};
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
        group_manager::GroupManager,
    },
    util::logging::console,
    wasm_interface::NodeGroupID,
};

///
/// A layout builder that takes another layout approach, and applies transitioning to it.
/// This will make layout changes smoothly transition from the previous state to the new state.
///
pub struct TransitionLayout<
    ET: Tag,
    T,
    E: Edge<Tag = ET>,
    N: InnerNode<E>,
    R: DiagramRules<E, N, T>,
    F: Function,
> where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    layout: Box<dyn LayoutRules<ET, F>>,
    duration: u32,
}

impl<ET: Tag, T, E: Edge<Tag = ET>, N: InnerNode<E>, R: DiagramRules<E, N, T>, F: Function>
    TransitionLayout<ET, T, E, N, R, F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    pub fn new(layout: Box<dyn LayoutRules<ET, F>>) -> TransitionLayout<ET, T, E, N, R, F> {
        TransitionLayout {
            layout,
            duration: 1000,
        }
    }
}

impl<ET: Tag, T, E: Edge<Tag = ET>, N: InnerNode<E>, R: DiagramRules<E, N, T>, F: Function>
    LayoutRules<ET, F> for TransitionLayout<ET, T, E, N, R, F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn layout(
        &mut self,
        groups: &GroupManager<ET, F>,
        old: &DiagramLayout<ET>,
        time: u32,
    ) -> DiagramLayout<ET> {
        let duration = self.duration;
        let old_time = time;
        let new = self.layout.layout(groups, old, time);

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

        let map_edges = |to: &NodeGroupID,
                         edges: &HashMap<EdgeType<ET>, EdgeLayout>,
                         old_group: &NodeGroupLayout<ET>|
         -> HashMap<EdgeType<ET>, EdgeLayout> {
            edges
                .iter()
                .map(|(edge_type, edge)| {
                    if let Some(old_to_edges) = old_group.edges.get(to) {
                        if let Some(old_edge) = old_to_edges.get(edge_type) {
                            // Add all points needed for the new layout, and transition from any old points
                            let mut new_points: Vec<EdgePoint> = edge
                                .points
                                .iter()
                                .enumerate()
                                .map(|(index, point)| {
                                    let out_point = if index >= old_edge.points.len() {
                                        let to_pos = old.groups.get(to).unwrap().center_position;
                                        Transition {
                                            old: get_current_point(to_pos),
                                            new: point.point.new,
                                            duration,
                                            old_time: time,
                                        }
                                    } else {
                                        Transition {
                                            old: get_current_point(
                                                old_edge.points.get(index).unwrap().point,
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
                                old_edge
                                    .points
                                    .iter()
                                    .skip(edge.points.len())
                                    .filter(|point| get_current_float(point.exists) > 0.0)
                                    .map(|point| EdgePoint {
                                        point: Transition {
                                            duration,
                                            old_time,
                                            old: get_current_point(point.point),
                                            new: new.groups.get(to).unwrap().center_position.new,
                                        },
                                        exists: Transition {
                                            duration,
                                            old_time,
                                            old: get_current_float(point.exists),
                                            new: 0.0,
                                        },
                                    }),
                            );

                            return (
                                *edge_type,
                                EdgeLayout {
                                    points: new_points,
                                    exists: edge.exists,
                                },
                            );
                        }
                    }

                    (*edge_type, {
                        EdgeLayout {
                            points: edge
                                .points
                                .iter()
                                .filter(|_| false)
                                .map(|point| EdgePoint {
                                    point: Transition {
                                        duration,
                                        old_time,
                                        old: get_current_point(old_group.center_position),
                                        new: point.point.new,
                                    },
                                    exists: point.exists,
                                })
                                .collect(),
                            exists: edge.exists,
                        }
                    })
                })
                .collect()
        };

        DiagramLayout {
            groups: new
                .groups
                .iter()
                .map(|(id, group)| {
                    (
                        *id,
                        if let Some(old_group) = old.groups.get(id) {
                            NodeGroupLayout {
                                center_position: Transition {
                                    old_time,
                                    duration,
                                    old: get_current_point(old_group.center_position),
                                    new: group.center_position.new,
                                },
                                size: Transition {
                                    old_time,
                                    duration: duration * 2,
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
                                    .map(|(to, edges)| (*to, map_edges(to, edges, old_group)))
                                    .collect(),
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
