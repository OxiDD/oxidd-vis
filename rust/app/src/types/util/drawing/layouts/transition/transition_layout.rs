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
                DiagramLayout, EdgeLayout, EdgePoint, LayerLayout, LayerStyle, NodeGroupLayout,
                NodeStyle,
            },
            layout_rules::LayoutRules,
        },
        graph_structure::{
            graph_structure::DrawTag,
            grouped_graph_structure::{EdgeData, GroupedGraphStructure, SourceReader},
        },
    },
    util::{logging::console, point::Point, transition::Transition},
    wasm_interface::NodeGroupID,
};

use super::{
    relate_elements::{relate_elements, ElementRelations, TargetEdge, TargetGroup},
    transition_layers::transition_layers,
};

///
/// A layout builder that takes another layout approach, and applies transitioning to it.
/// This will make layout changes smoothly transition from the previous state to the new state.
///
pub struct TransitionLayout<L: LayoutRules> {
    layout: L,
    durations: TransitionDurations,
}

impl<L: LayoutRules> TransitionLayout<L> {
    pub fn new(layout: L) -> Self {
        let speed_modifier = 1; // for testing
                                // TODO: add parameters
        TransitionLayout {
            layout,
            // TODO: make this more finely grained configurable
            durations: TransitionDurations {
                insert_duration: 900 * speed_modifier,
                transition_duration: 600 * speed_modifier,
                delete_duration: 300 * speed_modifier,
            },
        }
    }
    pub fn get_layout_rules(&mut self) -> &mut L {
        &mut self.layout
    }
}

#[derive(Clone)]
pub struct TransitionDurations {
    transition_duration: u32,
    delete_duration: u32,
    insert_duration: u32,
}

impl<L: LayoutRules> LayoutRules for TransitionLayout<L> {
    type T = L::T;
    type NS = L::NS;
    type LS = L::LS;
    type Tracker = L::Tracker;
    type G = L::G;

    fn layout(
        &mut self,
        graph: &Self::G,
        old: &DiagramLayout<Self::T, Self::NS, Self::LS>,
        sources: &Self::Tracker,
        time: u32,
    ) -> DiagramLayout<Self::T, Self::NS, Self::LS> {
        let duration = self.durations.transition_duration;
        let old_time = time;
        let new = self.layout.layout(graph, old, sources, time);

        let relations = relate_elements(graph, old, &new, sources, time);
        let ElementRelations {
            previous_groups,
            deleted_groups,
            previous_edges: _,
            deleted_edges: _,
        } = &relations;

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
            .map(|(&id, group, (old_group, old_target_group))| {
                (
                    id,
                    layout_updated_group(
                        id,
                        group,
                        old_group,
                        old_target_group,
                        &old,
                        &new,
                        &self.durations,
                        &relations,
                        time,
                    ),
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
            .map(|(&id, group, _)| {
                (
                    id,
                    layout_added_group::<Self::T, Self::NS, Self::LS>(
                        id,
                        group,
                        &some_updated_parents,
                        &updated_groups,
                        &self.durations,
                        time,
                    ),
                )
            })
            .collect_vec();

        let old_groups = old.groups.iter().filter_map(|(&id, group)| {
            let target_group = deleted_groups.get(&id)?;
            Some((
                id,
                layout_removed_group(
                    id,
                    group,
                    target_group,
                    &new,
                    &self.durations,
                    &relations,
                    time,
                ),
            ))
        });

        let groups = new_groups
            .into_iter()
            .chain(updated_groups.into_iter())
            .chain(old_groups.clone())
            .collect::<HashMap<_, _>>();
        DiagramLayout {
            groups,
            layers: transition_layers(&old.layers, &new.layers, duration, old_time, time),
        }
    }
}

fn layout_updated_group<T: DrawTag, S: NodeStyle, LS: LayerStyle>(
    id: NodeGroupID,
    group: &NodeGroupLayout<T, S>,
    old_group: &NodeGroupLayout<T, S>,
    target_data: &TargetGroup,
    old: &DiagramLayout<T, S, LS>,
    new: &DiagramLayout<T, S, LS>,
    durations: &TransitionDurations,
    relations: &ElementRelations<T>,
    time: u32,
) -> NodeGroupLayout<T, S> {
    let old_time = time;
    let duration = durations.transition_duration;

    let cur_size = old_group.size.get(time);
    let cur_old_group_position = old_group.position.get(time);
    let old_position = cur_old_group_position + target_data.offset;
    let start_size = if target_data.represents {
        cur_size
    } else {
        group.size.new
    };

    let deleted_edges_layout = layout_deleted_edges(
        id,
        old_group,
        group.position.new,
        durations,
        &relations,
        time,
    );

    NodeGroupLayout {
        position: Transition {
            old_time,
            duration,
            old: old_position,
            new: group.position.new,
        },
        size: Transition {
            old_time,
            duration: duration,
            old: start_size,
            new: group.size.new,
        },
        exists: Transition {
            old_time,
            duration,
            old: old_group.exists.get(time),
            new: group.exists.new,
        },
        level_range: group.level_range.clone(),
        edges: group
            .edges
            .iter()
            .map(|(edge_data, edge)| {
                (
                    edge_data.clone(),
                    layout_current_edge(
                        id,
                        edge_data,
                        edge,
                        old_position,
                        &old,
                        &new,
                        &durations,
                        &relations,
                        time,
                    ),
                )
            })
            .chain(deleted_edges_layout.into_iter())
            .collect(),
        style: Transition {
            old_time,
            duration,
            old: old_group.style.get(time),
            new: group.style.new.clone(),
        },
    }
}

fn layout_added_group<T: DrawTag, S: NodeStyle, LS: LayerStyle>(
    id: NodeGroupID,
    group: &NodeGroupLayout<T, S>,
    // Per node, potentially a parent edge that is a node in both the old and new layout
    some_updated_parents: &HashMap<usize, usize>,
    updated_groups: &HashMap<usize, NodeGroupLayout<T, S>>,
    durations: &TransitionDurations,
    time: u32,
) -> NodeGroupLayout<T, S> {
    let old_time = time;
    let duration = durations.transition_duration;

    if let Some(parent_id) = some_updated_parents.get(&id) {
        let parent = updated_groups.get(parent_id).unwrap();
        NodeGroupLayout {
            position: Transition {
                old_time,
                duration,
                old: parent.position.get(time),
                new: group.position.new,
            },

            style: Transition {
                old_time,
                duration,
                old: parent.style.get(time),
                new: group.style.new.clone(),
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
        }
    } else {
        NodeGroupLayout {
            exists: Transition {
                old_time,
                duration: durations.insert_duration,
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
                                duration: durations.insert_duration,
                                old: 0.,
                                new: edge.exists.new,
                            },
                            ..edge.clone()
                        },
                    )
                })
                .collect(),
            ..group.clone()
        }
    }
}

fn layout_removed_group<T: DrawTag, S: NodeStyle, LS: LayerStyle>(
    id: NodeGroupID,
    group: &NodeGroupLayout<T, S>,
    target_data: &Option<TargetGroup>,
    new: &DiagramLayout<T, S, LS>,
    durations: &TransitionDurations,
    relations: &ElementRelations<T>,
    time: u32,
) -> NodeGroupLayout<T, S> {
    let old_time = time;
    let duration = durations.transition_duration;

    let target = target_data.as_ref().and_then(|target_group| {
        new.groups
            .get(&target_group.id)
            .zip(Some(target_group.offset))
    });

    let deleted_edges_layout = layout_deleted_edges(
        id,
        &group,
        if let Some((target, offset)) = target {
            target.position.new + offset.clone()
        } else {
            group.position.new
        },
        &durations,
        &relations,
        time,
    );

    match target {
        Some((target, offset)) => {
            let cur_pos = group.position.get(time);
            NodeGroupLayout {
                position: Transition {
                    old_time,
                    duration,
                    old: cur_pos,
                    new: target.position.new + offset.clone(),
                },
                style: Transition {
                    old_time,
                    duration,
                    old: group.style.get(time),
                    new: target.style.new.clone(),
                },
                exists: Transition {
                    old_time: old_time + duration,
                    duration: durations.delete_duration,
                    old: group.exists.get(time),
                    new: 0.,
                },
                size: Transition {
                    old_time,
                    duration,
                    old: group.size.get(time),
                    new: Point {
                        x: group.size.new.x.min(target.size.new.x),
                        y: group.size.new.y.min(target.size.new.y),
                    },
                },
                edges: deleted_edges_layout,
                ..group.clone()
            }
        }
        _ => NodeGroupLayout {
            exists: Transition {
                old_time,
                duration: durations.delete_duration,
                old: group.exists.get(time),
                new: 0.,
            },
            edges: deleted_edges_layout,
            ..group.clone()
        },
    }
}

fn layout_current_edge<T: DrawTag, S: NodeStyle, LS: LayerStyle>(
    from: NodeGroupID,
    edge: &EdgeData<T>,
    edge_layout: &EdgeLayout,
    old_group_position: Point,
    old: &DiagramLayout<T, S, LS>,
    new: &DiagramLayout<T, S, LS>,
    durations: &TransitionDurations,
    relations: &ElementRelations<T>,
    time: u32,
) -> EdgeLayout {
    let old_time = time;
    let duration = durations.transition_duration;

    let maybe_old_edge = relations
        .previous_edges
        .get(&(from, edge.clone()))
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

    if let Some((old_edge_data, old_edge_layout, start_node_offset, end_node_offset)) =
        maybe_old_edge
    {
        let start_offset = old_edge_layout.start_offset.get(time) - start_node_offset;
        let end_offset = old_edge_layout.end_offset.get(time) - end_node_offset;

        // Add all points needed for the new layout, and transition from any old points
        let mut new_points: Vec<EdgePoint> = edge_layout
            .points
            .iter()
            .enumerate()
            .map(|(index, point)| {
                let out_point = if index >= old_edge_layout.points.len() {
                    let to_pos = old.groups.get(&old_edge_data.to).unwrap().position;
                    Transition {
                        old_time,
                        duration,
                        old: to_pos.get(time) + end_offset,
                        new: point.point.new,
                    }
                } else {
                    Transition {
                        old_time,
                        duration,
                        old: old_edge_layout.points.get(index).unwrap().point.get(time),
                        new: point.point.new,
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
                .skip(edge_layout.points.len())
                .filter(|point| point.exists.get(time) > 0.0)
                .map(|point| EdgePoint {
                    point: Transition {
                        old_time,
                        duration,
                        old: point.point.get(time),
                        new: new.groups.get(&edge.to).unwrap().position.new
                            + edge_layout.end_offset.new,
                    },
                    exists: Transition {
                        old_time,
                        duration,
                        old: point.exists.get(time),
                        new: 0.0,
                    },
                }),
        );

        EdgeLayout {
            start_offset: Transition {
                duration,
                old_time,
                old: start_offset,
                new: edge_layout.start_offset.new,
            },
            end_offset: Transition {
                duration,
                old_time,
                old: end_offset,
                new: edge_layout.end_offset.new,
            },
            points: new_points,
            exists: Transition {
                duration,
                old_time,
                old: old_edge_layout.exists.get(time),
                new: edge_layout.exists.new,
            },
            curve_offset: Transition {
                duration,
                old_time,
                old: old_edge_layout.curve_offset.get(time),
                new: edge_layout.curve_offset.new,
            },
        }
    } else {
        // There is no edge to morph, so create a new one
        let was_hidden = relations
            .previous_groups
            .get(&from)
            .zip(relations.previous_groups.get(&edge.to))
            .map(|(from_source, to_source)| from_source.id == to_source.id)
            .unwrap_or(false);
        let start_offset = edge_layout.start_offset.get(time);
        let points = edge_layout
            .points
            .iter()
            .map(|point| EdgePoint {
                point: Transition {
                    duration,
                    old_time,
                    old: old_group_position + start_offset,
                    new: point.point.new,
                },
                exists: point.exists,
            })
            .collect();

        EdgeLayout {
            start_offset: edge_layout.start_offset,
            end_offset: edge_layout.end_offset,
            points: points,
            exists: if was_hidden {
                // If the group was hidden in another group, the edge does not have to fade in
                edge_layout.exists
            } else {
                Transition {
                    old_time,
                    duration: durations.insert_duration,
                    old: 0.,
                    new: edge_layout.exists.new,
                }
            },
            curve_offset: Transition {
                old_time,
                duration: durations.insert_duration,
                old: 0.,
                new: edge_layout.curve_offset.new,
            },
        }
    }
}

fn layout_deleted_edges<T: DrawTag, S: NodeStyle>(
    from: NodeGroupID,
    group: &NodeGroupLayout<T, S>,
    point_pos: Point,
    durations: &TransitionDurations,
    relations: &ElementRelations<T>,
    time: u32,
) -> HashMap<EdgeData<T>, EdgeLayout> {
    let old_time = time;
    let duration = durations.transition_duration;

    relations
        .deleted_edges
        .get(&from)
        .map(|node_deleted_edges| {
            node_deleted_edges.iter().filter_map(|target_edge| {
                let new_edge = &target_edge.edge_data;
                let old_to = relations
                    .previous_groups
                    .get(&new_edge.to)
                    .map(|group_target| group_target.id)
                    .unwrap_or(new_edge.to);
                let old_edge = EdgeData {
                    to: old_to,
                    ..new_edge.clone()
                };
                let old_edge_layout = group.edges.get(&old_edge)?;

                let exists = old_edge_layout.exists.get(time);
                if exists <= 0. {
                    return None;
                }

                Some((
                    new_edge.clone(),
                    EdgeLayout {
                        exists: if target_edge.morph.is_some() {
                            Transition {
                                old_time: old_time + duration,
                                duration: 1,
                                old: exists,
                                new: 0.,
                            }
                        } else {
                            Transition {
                                old_time,
                                duration: durations.delete_duration,
                                old: exists,
                                new: 0.,
                            }
                        },
                        points: old_edge_layout
                            .points
                            .iter()
                            .map(|point| EdgePoint {
                                exists: Transition {
                                    old_time,
                                    duration: durations.delete_duration,
                                    old: point.exists.get(time),
                                    new: 0.,
                                },
                                point: Transition {
                                    old_time,
                                    duration,
                                    old: point.point.get(time),
                                    new: point_pos + old_edge_layout.start_offset.new,
                                },
                            })
                            .collect(),
                        start_offset: if let Some(morph) = &target_edge.morph {
                            Transition {
                                old_time,
                                duration,
                                old: old_edge_layout.start_offset.get(time),
                                new: old_edge_layout.start_offset.new + morph.start_offset,
                            }
                        } else {
                            old_edge_layout.start_offset
                        },
                        end_offset: if let Some(morph) = &target_edge.morph {
                            Transition {
                                old_time,
                                duration,
                                old: old_edge_layout.end_offset.get(time),
                                new: old_edge_layout.end_offset.new + morph.end_offset,
                            }
                        } else {
                            old_edge_layout.end_offset
                        },
                        curve_offset: Transition {
                            old_time,
                            duration: durations.insert_duration,
                            old: old_edge_layout.curve_offset.get(time),
                            new: 0.,
                        },
                    },
                ))
            })
        })
        .into_iter()
        .flatten()
        .collect()
}
