use std::{
    collections::{HashMap, HashSet, LinkedList},
    iter::{once, FromIterator},
    marker::PhantomData,
    ops::Add,
};

use itertools::{EitherOrBoth, Itertools};
use multimap::MultiMap;
use oxidd::{Edge, Function, InnerNode, LevelNo, Manager};
use oxidd_core::{DiagramRules, Tag};

use crate::{
    types::util::{
        drawing::{
            diagram_layout::{
                DiagramLayout, EdgeLayout, EdgePoint, LayerLayout, NodeGroupLayout, Point,
                Transition,
            },
            layout_rules::LayoutRules,
        },
        graph_structure::{
            graph_structure::DrawTag,
            grouped_graph_structure::{EdgeData, GroupedGraphStructure, SourceReader},
        },
    },
    util::logging::console,
    wasm_interface::NodeGroupID,
};

use super::relate_elements::{relate_elements, ElementRelations};

///
/// A layout builder that takes another layout approach, and applies transitioning to it.
/// This will make layout changes smoothly transition from the previous state to the new state.
///
pub struct TransitionLayout<
    T: DrawTag,
    GL,
    LL,
    G: GroupedGraphStructure<T, GL, LL>,
    L: LayoutRules<T, GL, LL, G>,
> {
    layout: L,
    transition_duration: u32,
    delete_duration: u32,
    insert_duration: u32,
    group_label: PhantomData<GL>,
    level_label: PhantomData<LL>,
    // TODO: see if these generics and  phantom data is even needed
    tag: PhantomData<T>,
    graph: PhantomData<G>,
}

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, L: LayoutRules<T, GL, LL, G>>
    TransitionLayout<T, GL, LL, G, L>
{
    pub fn new(layout: L) -> TransitionLayout<T, GL, LL, G, L> {
        let speed_modifier = 3; // for testing
                                // TODO: add parameters
        TransitionLayout {
            layout,
            insert_duration: 900 * speed_modifier,
            transition_duration: 600 * speed_modifier,
            delete_duration: 300 * speed_modifier,
            tag: PhantomData,
            graph: PhantomData,
            group_label: PhantomData,
            level_label: PhantomData,
        }
    }
}

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>, L: LayoutRules<T, GL, LL, G>>
    LayoutRules<T, GL, LL, G> for TransitionLayout<T, GL, LL, G, L>
{
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T>,
        sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T> {
        let duration = self.transition_duration;
        let insert_duration = self.insert_duration;
        let delete_duration = self.delete_duration;
        let old_time = time;
        let new = self.layout.layout(graph, old, sources, time);

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
        let get_current_color = |val: Transition<(f32, f32, f32)>| {
            let per = get_per(time, val);
            let r = (val.old.0 * val.old.0 * (1.0 - per) + val.new.0 * val.new.0 * per).sqrt();
            let g = (val.old.1 * val.old.1 * (1.0 - per) + val.new.1 * val.new.1 * per).sqrt();
            let b = (val.old.2 * val.old.2 * (1.0 - per) + val.new.2 * val.new.2 * per).sqrt();
            (r, g, b)
        };

        let ElementRelations {
            previous_groups,
            deleted_groups,
            previous_edges,
            deleted_edges,
        } = relate_elements(graph, old, &new, sources, time);

        let map_edge = |from: NodeGroupID,
                        edge_data: &EdgeData<T>,
                        edge: &EdgeLayout,
                        old_group: &NodeGroupLayout<T>|
         -> EdgeLayout {
            let maybe_old_edge =
                previous_edges
                    .get(&(from, edge_data.clone()))
                    .and_then(|old_edge_target| {
                        old.groups.get(&old_edge_target.group_id).and_then(|group| {
                            group
                                .edges
                                .get(&old_edge_target.edge_data)
                                .map(|old_edge_layout| {
                                    (
                                        &old_edge_target.edge_data,
                                        old_edge_layout,
                                        old_edge_target.start_offset,
                                        old_edge_target.end_offset,
                                    )
                                })
                        })
                    });

            // let edge_type = edge_data.edge_type;
            if let Some((old_edge_data, old_edge_layout, start_node_offset, end_node_offset)) =
                maybe_old_edge
            {
                let start_offset =
                    get_current_point(old_edge_layout.start_offset) - start_node_offset;
                let end_offset = get_current_point(old_edge_layout.end_offset) - end_node_offset;

                // Add all points needed for the new layout, and transition from any old points
                let mut new_points: Vec<EdgePoint> = edge
                    .points
                    .iter()
                    .enumerate()
                    .map(|(index, point)| {
                        let out_point = if index >= old_edge_layout.points.len() {
                            let to_pos = old.groups.get(&old_edge_data.to).unwrap().position;
                            Transition {
                                old: get_current_point(to_pos) + end_offset,
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
                                new: new.groups.get(&edge_data.to).unwrap().position.new
                                    + edge.end_offset.new,
                            },
                            exists: Transition {
                                duration: delete_duration,
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
                        old: start_offset,
                        new: edge.start_offset.new,
                    },
                    end_offset: Transition {
                        duration,
                        old_time,
                        old: end_offset,
                        new: edge.end_offset.new,
                    },
                    points: new_points,
                    exists: Transition {
                        duration,
                        old_time,
                        old: get_current_float(old_edge_layout.exists),
                        new: edge.exists.new,
                    },
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
                            // TODO: could also transition in from the new node's edge start position
                            old: get_current_point(old_group.position)
                                + get_current_point(edge.end_offset),
                            new: point.point.new,
                        },
                        exists: point.exists,
                    })
                    .collect();
                EdgeLayout {
                    start_offset: edge.start_offset,
                    end_offset: edge.end_offset,
                    points: points,
                    exists: Transition {
                        old_time,
                        duration: insert_duration,
                        old: 0.,
                        new: edge.exists.new,
                    },
                    curve_offset: edge.curve_offset,
                }
            }
        };

        let groups = new.groups.iter().map(|(id, group)| {
            (
                id,
                group,
                previous_groups.get(&id).and_then(|old_target_group| {
                    old.groups
                        .get(&old_target_group.id)
                        .map(|group| (group, old_target_group))
                }),
            )
        });

        let updated_groups = groups
            .clone()
            .filter_map(|(id, group, old_group_data)| {
                old_group_data.map(|old_group_data| (id, group, old_group_data))
            })
            .map(|(id, group, (old_group, old_target_group))| {
                let cur_size = get_current_point(old_group.size);
                let cur_position = get_current_point(old_group.position);
                let start_size = if old_target_group.represents {
                    cur_size
                } else {
                    // TODO: Perform better inside bounding box guarantee that uses offset
                    // Point {
                    //     x: f32::min(cur_size.x, group.size.new.x),
                    //     y: f32::min(cur_size.y, group.size.new.y),
                    // }
                    group.size.new
                };

                let deleted_edges_layout = deleted_edges.get(id).map(|node_deleted_edges| {
                    node_deleted_edges.iter().filter_map(|target_edge| {
                        let new_edge = &target_edge.edge_data;
                        let old_to = previous_groups
                            .get(&new_edge.to)
                            .map(|group_target| group_target.id)
                            .unwrap_or(new_edge.to);
                        let old_edge = EdgeData {
                            to: old_to,
                            ..new_edge.clone()
                        };
                        let old_edge_layout = old_group.edges.get(&old_edge)?;

                        let exists = get_current_float(old_edge_layout.exists);
                        if exists <= 0. {
                            return None;
                        }

                        let moving_out = deleted_groups
                            .get(&new_edge.to)
                            .map(|target| target.is_some())
                            .unwrap_or(false);
                        Some((
                            new_edge.clone(),
                            EdgeLayout {
                                exists: if moving_out {
                                    Transition {
                                        old_time: old_time + duration,
                                        duration: 1,
                                        old: exists,
                                        new: 0.,
                                    }
                                } else {
                                    Transition {
                                        old_time,
                                        duration: delete_duration,
                                        old: exists,
                                        new: 0.,
                                    }
                                },
                                points: old_edge_layout
                                    .points
                                    .iter()
                                    .map(|(point)| EdgePoint {
                                        exists: Transition {
                                            old_time,
                                            duration: delete_duration,
                                            old: get_current_float(point.exists),
                                            new: 0.,
                                        },
                                        point: Transition {
                                            old_time,
                                            duration,
                                            old: get_current_point(point.point),
                                            new: group.position.new
                                                + old_edge_layout.start_offset.new,
                                        },
                                    })
                                    .collect(),
                                ..old_edge_layout.clone()
                            },
                        ))
                    })
                });
                // console::log!(
                //     "Deleted edge count: {}",
                //     deleted_edges
                //         .clone()
                //         .into_iter()
                //         .flatten()
                //         .collect_vec()
                //         .len()
                // );
                (
                    *id,
                    NodeGroupLayout {
                        position: Transition {
                            old_time,
                            duration,
                            old: cur_position + old_target_group.offset,
                            new: group.position.new,
                        },
                        size: Transition {
                            old_time,
                            duration: duration,
                            old: start_size,
                            new: group.size.new,
                        },
                        label: group.label.clone(),
                        exists: Transition {
                            old_time,
                            duration,
                            old: get_current_float(old_group.exists),
                            new: group.exists.new,
                        },
                        level_range: group.level_range.clone(),
                        edges: group
                            .edges
                            .iter()
                            .map(|(edge_data, edge)| {
                                (edge_data.clone(), map_edge(*id, edge_data, edge, old_group))
                            })
                            .chain(deleted_edges_layout.into_iter().flatten())
                            .collect(),
                        color: Transition {
                            old_time,
                            duration,
                            old: get_current_color(old_group.color),
                            new: group.color.new,
                        },
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        let some_updated_parents = groups
            .clone()
            .filter(|(_id, _group, old_group_data)| old_group_data.is_some())
            .flat_map(|(&id, group, _)| {
                group
                    .edges
                    .iter()
                    .map(move |(edge_data, _)| (edge_data.to, id))
            })
            .collect::<HashMap<_, _>>();

        let new_groups = groups
            .filter(|(_id, _group, old_group_data)| old_group_data.is_none())
            .map(|(id, group, _)| {
                if let Some(parent_id) = some_updated_parents.get(id) {
                    let parent = updated_groups.get(parent_id).unwrap();
                    (
                        *id,
                        NodeGroupLayout {
                            position: Transition {
                                old_time,
                                duration,
                                old: get_current_point(parent.position),
                                new: group.position.new,
                            },
                            color: Transition {
                                old_time,
                                duration,
                                old: get_current_color(parent.color),
                                new: group.color.new,
                            },
                            edges: group
                                .edges
                                .iter()
                                .map(|(edge_data, edge)| {
                                    (
                                        edge_data.clone(),
                                        EdgeLayout {
                                            points: edge
                                                .points
                                                .iter()
                                                .map(|point| EdgePoint {
                                                    point: Transition {
                                                        old_time,
                                                        duration,
                                                        old: parent.position.old,
                                                        new: point.point.new.clone(),
                                                    },
                                                    ..point.clone()
                                                })
                                                .collect(),
                                            ..edge.clone()
                                        },
                                    )
                                })
                                .collect(),
                            ..group.clone()
                        },
                    )
                } else {
                    (
                        *id,
                        NodeGroupLayout {
                            exists: Transition {
                                old_time,
                                duration: insert_duration,
                                old: 0.,
                                new: group.exists.new,
                            },
                            edges: group
                                .edges
                                .iter()
                                .map(|(edge_data, edge)| {
                                    (
                                        edge_data.clone(),
                                        EdgeLayout {
                                            exists: Transition {
                                                old_time,
                                                duration: insert_duration,
                                                old: 0.,
                                                new: edge.exists.new,
                                            },
                                            ..edge.clone()
                                        },
                                    )
                                })
                                .collect(),
                            ..group.clone()
                        },
                    )
                }
            })
            .collect_vec();

        let old_groups = old.groups.iter().filter_map(|(&id, group)| {
            let Some(target_group) = deleted_groups.get(&id) else {
                return None;
            };
            let target = target_group.as_ref().and_then(|target_group| {
                new.groups
                    .get(&target_group.id)
                    .zip(Some(target_group.offset))
            });

            let deleted_edges_layout = deleted_edges.get(&id).map(|node_deleted_edges| {
                node_deleted_edges.iter().filter_map(|target_edge| {
                    let new_edge = &target_edge.edge_data;
                    let old_to = previous_groups
                        .get(&new_edge.to)
                        .map(|group_target| group_target.id)
                        .unwrap_or(new_edge.to);
                    let old_edge = EdgeData {
                        to: old_to,
                        ..new_edge.clone()
                    };
                    let old_edge_layout = group.edges.get(&old_edge)?;

                    let exists = get_current_float(old_edge_layout.exists);
                    if exists <= 0. {
                        return None;
                    }

                    let moving_out = target.is_some()
                        && deleted_groups
                            .get(&new_edge.to)
                            .map(|target| target.is_some())
                            .unwrap_or(false);
                    Some((
                        new_edge.clone(),
                        EdgeLayout {
                            exists: if moving_out {
                                Transition {
                                    old_time: old_time + duration,
                                    duration: 1,
                                    old: exists,
                                    new: 0.,
                                }
                            } else {
                                Transition {
                                    old_time,
                                    duration: delete_duration,
                                    old: exists,
                                    new: 0.,
                                }
                            },
                            points: old_edge_layout
                                .points
                                .iter()
                                .map(|(point)| EdgePoint {
                                    exists: Transition {
                                        old_time,
                                        duration: delete_duration,
                                        old: get_current_float(point.exists),
                                        new: 0.,
                                    },
                                    point: Transition {
                                        old_time,
                                        duration,
                                        old: get_current_point(point.point),
                                        new: if let Some((target, offset)) = target {
                                            target.position.new + offset.clone()
                                        } else {
                                            group.position.new
                                        } + old_edge_layout.start_offset.new,
                                    },
                                })
                                .collect(),
                            ..old_edge_layout.clone()
                        },
                    ))
                })
            });

            Some((
                id,
                match target {
                    Some((target, offset)) => {
                        let cur_pos = get_current_point(group.position);
                        NodeGroupLayout {
                            position: Transition {
                                old_time,
                                duration,
                                old: cur_pos,
                                new: target.position.new + offset.clone(),
                            },
                            color: Transition {
                                old_time,
                                duration,
                                old: get_current_color(group.color),
                                new: target.color.new,
                            },
                            exists: Transition {
                                old_time: old_time + duration,
                                duration: 1,
                                old: get_current_float(group.exists),
                                new: 0.,
                            },
                            edges: deleted_edges_layout.into_iter().flatten().collect(),
                            ..group.clone()
                        }
                    }
                    _ => NodeGroupLayout {
                        exists: Transition {
                            old_time,
                            duration: delete_duration,
                            old: get_current_float(group.exists),
                            new: 0.,
                        },
                        edges: deleted_edges_layout.into_iter().flatten().collect(),
                        ..group.clone()
                    },
                },
            ))
        });

        let groups = new_groups
            .into_iter()
            .chain(updated_groups.into_iter())
            .chain(old_groups.clone())
            .collect::<HashMap<_, _>>();
        DiagramLayout {
            groups,
            layers: transition_layers(
                &old.layers,
                &new.layers,
                duration,
                old_time,
                &get_current_float,
            ),
        }
    }
}

fn transition_layers(
    old: &Vec<LayerLayout>,
    new: &Vec<LayerLayout>,
    duration: u32,
    old_time: u32,
    get_current_float: &impl Fn(Transition<f32>) -> f32,
) -> Vec<LayerLayout> {
    let mut out = Vec::new();

    let transition_out = |old_layer: &LayerLayout, out: &mut Vec<LayerLayout>| {
        let exists = get_current_float(old_layer.exists);
        if exists > 0. {
            out.push(LayerLayout {
                exists: Transition {
                    old_time,
                    duration,
                    old: exists,
                    new: 0.,
                },
                ..old_layer.clone()
            });
        }
    };

    let mut old_iter = old.iter().peekable();
    for new_layer in new {
        // Progress to the right old layer
        while let Some(&old_layer) = old_iter.peek() {
            if old_layer.exists.new >= 1. && old_layer.start_layer >= new_layer.start_layer {
                break;
            }
            old_iter.next();
            transition_out(&old_layer, &mut out);
        }

        // Try to transition from old to new
        if let Some(&old_layer) = old_iter.peek() {
            if old_layer.start_layer == new_layer.start_layer
                && old_layer.end_layer == new_layer.end_layer
            {
                old_iter.next();
                out.push(LayerLayout {
                    bottom: Transition {
                        old_time,
                        duration,
                        old: get_current_float(old_layer.bottom),
                        new: new_layer.bottom.new,
                    },
                    top: Transition {
                        old_time,
                        duration,
                        old: get_current_float(old_layer.top),
                        new: new_layer.top.new,
                    },
                    exists: Transition {
                        old_time,
                        duration,
                        old: get_current_float(old_layer.exists),
                        new: new_layer.exists.new,
                    },
                    index: Transition {
                        old_time,
                        duration,
                        old: get_current_float(old_layer.index),
                        new: new_layer.index.new,
                    },
                    ..new_layer.clone()
                });
                continue;
            }
        }

        // Otherwise insert new
        if old.len() == 0 {
            out.push(new_layer.clone()); // Don't transition in when there is no old
        } else {
            let center = (new_layer.bottom.old + new_layer.top.old) / 2.;
            out.push(LayerLayout {
                top: Transition {
                    old_time,
                    duration,
                    old: center,
                    new: new_layer.top.new,
                },
                bottom: Transition {
                    old_time,
                    duration,
                    old: center,
                    new: new_layer.bottom.new,
                },
                exists: Transition {
                    old_time,
                    duration,
                    old: 0.,
                    new: new_layer.exists.new,
                },
                ..new_layer.clone()
            });
        }
    }

    // Transition out any other old layers
    for old_layer in old_iter {
        transition_out(&old_layer, &mut out);
    }

    out
}

fn transition_layers_shift(
    old: &Vec<LayerLayout>,
    new: &Vec<LayerLayout>,
    duration: u32,
    old_time: u32,
    get_current_float: &impl Fn(Transition<f32>) -> f32,
) -> Vec<LayerLayout> {
    let prev_bottom = old
        .iter()
        .last()
        .map(|last_old| get_current_float(last_old.bottom));

    new.iter()
        .zip_longest(old.iter())
        .filter_map(|p| match p {
            EitherOrBoth::Both(new_layer, old_layer) => Some(LayerLayout {
                bottom: Transition {
                    old_time,
                    duration,
                    old: get_current_float(old_layer.bottom),
                    new: new_layer.bottom.new,
                },
                top: Transition {
                    old_time,
                    duration,
                    old: get_current_float(old_layer.top),
                    new: new_layer.top.new,
                },
                exists: Transition {
                    old_time,
                    duration,
                    old: get_current_float(old_layer.exists),
                    new: new_layer.exists.new,
                },
                ..new_layer.clone()
            }),
            EitherOrBoth::Left(new_layer) => Some(LayerLayout {
                top: Transition {
                    old_time,
                    duration,
                    old: prev_bottom.unwrap_or(new_layer.top.new),
                    new: new_layer.top.new,
                },
                bottom: Transition {
                    old_time,
                    duration,
                    old: prev_bottom.unwrap_or(new_layer.bottom.new),
                    new: new_layer.bottom.new,
                },
                exists: Transition {
                    old_time,
                    duration,
                    old: 0.,
                    new: new_layer.exists.new,
                },
                ..new_layer.clone()
            }),
            EitherOrBoth::Right(old_layer) => {
                let exists = get_current_float(old_layer.exists);
                if exists > 0.0 {
                    Some(old_layer.clone())
                } else {
                    None
                }
            }
        })
        .collect()
}

fn get_per<T>(time: u32, val: Transition<T>) -> f32 {
    let delta = time as f32 - val.old_time as f32;
    if val.duration == 0 {
        if delta > 0. {
            1.
        } else {
            0.
        }
    } else {
        f32::max(0.0, f32::min(delta / val.duration as f32, 1.0))
    }
}
