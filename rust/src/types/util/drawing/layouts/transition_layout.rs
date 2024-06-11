use std::{collections::HashMap, marker::PhantomData};

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

        let map_edge = |edge_data: &EdgeData<T>,
                        edge: &EdgeLayout,
                        old_group: &NodeGroupLayout<T>|
         -> EdgeLayout {
            let to = &edge_data.to;
            // let edge_type = edge_data.edge_type;
            if let Some(old_edge) = old_group.edges.get(edge_data) {
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
                                old: get_current_point(old_edge.points.get(index).unwrap().point),
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

                EdgeLayout {
                    start_offset: Transition {
                        duration,
                        old_time,
                        old: get_current_point(old_edge.start_offset),
                        new: edge.start_offset.new,
                    },
                    end_offset: Transition {
                        duration,
                        old_time,
                        old: get_current_point(old_edge.end_offset),
                        new: edge.end_offset.new,
                    },
                    points: new_points,
                    exists: edge.exists,
                }
            } else {
                let points = edge
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
                    .collect();
                EdgeLayout {
                    start_offset: edge.start_offset,
                    end_offset: edge.end_offset,
                    points: points,
                    exists: edge.exists,
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
                                        (edge_data.clone(), map_edge(edge_data, edge, old_group))
                                    })
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
