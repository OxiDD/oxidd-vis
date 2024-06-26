use std::{
    collections::{HashMap, HashSet, LinkedList},
    iter::FromIterator,
    marker::PhantomData,
    ops::Add,
};

use itertools::{EitherOrBoth, Itertools};
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
        edge_type::EdgeType,
        group_manager::{EdgeData, GroupManager},
        grouped_graph_structure::{GroupedGraphStructure, SourceReader},
    },
    util::logging::console,
    wasm_interface::NodeGroupID,
};

///
/// A layout builder that takes another layout approach, and applies transitioning to it.
/// This will make layout changes smoothly transition from the previous state to the new state.
///
pub struct TransitionLayout<T: Tag, G: GroupedGraphStructure<T>, L: LayoutRules<T, G>> {
    layout: L,
    duration: u32,
    // TODO: see if these generics and  phantom data is even needed
    tag: PhantomData<T>,
    graph: PhantomData<G>,
}

impl<T: Tag, G: GroupedGraphStructure<T>, L: LayoutRules<T, G>> TransitionLayout<T, G, L> {
    pub fn new(layout: L) -> TransitionLayout<T, G, L> {
        TransitionLayout {
            layout,
            duration: 1000,
            tag: PhantomData,
            graph: PhantomData,
        }
    }
}

impl<T: Tag, G: GroupedGraphStructure<T>, L: LayoutRules<T, G>> LayoutRules<T, G>
    for TransitionLayout<T, G, L>
{
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T>,
        sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T> {
        let duration = self.duration;
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

        let (node_mapping, edge_mapping) =
            relate_elements(graph, old, &new, sources, &get_current_point);

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
                let start_node_offset = node_mapping
                    .get(&from)
                    .map(|(_, offset, _)| *offset)
                    .unwrap_or_default();
                let start_offset =
                    get_current_point(old_edge_layout.start_offset) - start_node_offset;
                let end_node_offset = node_mapping
                    .get(&edge_data.to)
                    .map(|(_, offset, _)| *offset)
                    .unwrap_or_default();
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
                                new: new.groups.get(&edge_data.to).unwrap().position.new,
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
                        if let Some((&offset, old_group, &copy_size)) = node_mapping
                            .get(&id)
                            .and_then(|(old_id, offset, copy_size)| {
                                old.groups
                                    .get(old_id)
                                    .map(|group| (offset, group, copy_size))
                            })
                        {
                            let cur_size = get_current_point(old_group.size);
                            let cur_center = get_current_point(old_group.position);
                            let start_size = if copy_size {
                                cur_size
                            } else {
                                // TODO: Perform better inside bounding box guarantee that uses offset
                                Point {
                                    x: f32::min(cur_size.x, group.size.new.x),
                                    y: f32::min(cur_size.y, group.size.new.y),
                                }
                            };
                            NodeGroupLayout {
                                position: Transition {
                                    old_time,
                                    duration,
                                    old: cur_center + offset,
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
                            }
                        } else {
                            group.clone()
                        },
                    )
                })
                .collect(),
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

fn relate_elements<T: Tag, G: GroupedGraphStructure<T>>(
    graph: &G,
    old: &DiagramLayout<T>,
    new: &DiagramLayout<T>,
    sources: &G::Tracker,
    get_current_point: &impl Fn(Transition<Point>) -> Point,
) -> (
    /* A mapping from a node to (old node, offset, whether to use source size)*/
    HashMap<NodeGroupID, (NodeGroupID, Point, bool)>,
    /* A mapping from an edge (including source node) to another edge */
    HashMap<(NodeGroupID, EdgeData<T>), (NodeGroupID, EdgeData<T>)>,
) {
    let mut edge_mapping: HashMap<(NodeGroupID, EdgeData<T>), (NodeGroupID, EdgeData<T>)> =
        HashMap::new();

    // Perform initial node mapping without offsets
    let mut node_mapping: HashMap<NodeGroupID, (NodeGroupID, Point, bool)> = new
        .groups
        .iter()
        .map(|(&group_id, data)| {
            (
                group_id,
                (sources.get_source(group_id), Point { x: 0., y: 0. }, false),
            )
        })
        .collect();

    // Decide which node should represent the old source, and hence copy the size
    let mut source_dests: HashMap<NodeGroupID, HashSet<NodeGroupID>> = HashMap::new();
    for (&node, &(source, _, _)) in &node_mapping {
        source_dests
            .entry(source)
            .or_insert_with(|| HashSet::new())
            .insert(node);
    }
    for (_, dests) in source_dests {
        let sizes = dests
            .iter()
            .filter_map(|dest| new.groups.get(dest).map(|group| (dest, group.size.old)));
        let Some((node, _)) = sizes.reduce(|(node1, size1), (node2, size2)| {
            if size1.length() > size2.length() {
                (node1, size1)
            } else {
                (node2, size2)
            }
        }) else {
            continue;
        };
        node_mapping
            .entry(*node)
            .and_modify(|(_, _, copy_size)| *copy_size = true);
    }

    // Perform edge mapping + derive offsets
    for (&node, &(source, _, _)) in &node_mapping.clone() {
        for edge in graph.get_children(node) {
            let edge = edge.drop_count();
            let Some(&(to_source, _, _)) = node_mapping.get(&edge.to) else {
                continue;
            };

            let old_edge = EdgeData {
                to: to_source,
                ..edge
            };

            let Some(old_edge_layout) = old
                .groups
                .get(&source)
                .and_then(|group| group.edges.get(&old_edge))
            else {
                continue;
            };

            edge_mapping.insert((node, edge.clone()), (source, old_edge));

            let Some(new_edge_layout) = new
                .groups
                .get(&node)
                .and_then(|group| group.edges.get(&edge))
            else {
                continue;
            };

            if let Some((_, offset, false)) = node_mapping.get_mut(&edge.to) {
                *offset =
                    get_current_point(old_edge_layout.end_offset) - new_edge_layout.end_offset.old;
            }
            if let Some((_, offset, false)) = node_mapping.get_mut(&node) {
                *offset = get_current_point(old_edge_layout.start_offset)
                    - new_edge_layout.start_offset.old;
            }
        }
    }

    (node_mapping, edge_mapping)
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

    console::log!(
        "layers: {}",
        out.iter()
            .map(|layer| format!(
                "([{}], [{}], [{}])",
                layer.label.replace("\n", ""),
                layer.exists,
                layer.index
            ))
            .join(",\n ")
    );

    out
}
// fn transition_layers(
//     old: &Vec<LayerLayout>,
//     new: &Vec<LayerLayout>,
//     duration: u32,
//     old_time: u32,
//     get_current_float: &impl Fn(Transition<f32>) -> f32,
// ) -> Vec<LayerLayout> {
//     let prev_bottom = old
//         .iter()
//         .last()
//         .map(|last_old| get_current_float(last_old.bottom));

//     let boundaries = old
//         .iter()
//         .flat_map(|layer| [layer.start_layer, layer.end_layer + 1])
//         .chain(
//             new.iter()
//                 .flat_map(|layer| [layer.start_layer, layer.end_layer + 1]),
//         )
//         .sorted()
//         .dedup();

//     let mut old_iter = old.iter().peekable();
//     let mut new_iter = old.iter().peekable();
//     for (start_layer_no, end_layer_no) in boundaries.clone().zip(boundaries.skip(1)) {
//         // Progress to the right layer
//         while let Some(old) = old_iter.peek() {
//             if old.end_layer >= start_layer_no {
//                 break;
//             }
//             old_iter.next();
//         }
//         while let Some(new) = new_iter.peek() {
//             if new.end_layer >= start_layer_no {
//                 break;
//             }
//             new_iter.next();
//         }

//         // Match new case
//         match (old_iter.peek(), new_iter.peek()) {
//             (Some(old), Some(new)) => {
//                 let old_longer
//             }
//             _ => {}
//         }
//     }

//     let mut out = Vec::new();

//     let mut old_layers = old.iter().peekable();
//     for new_layer in new {
//         if let Some(old_layer) = old_layers.peek() {
//             let in_old = new_layer.end_layer <= old_layer.end_layer;
//             let in_new = old_layer.end_layer <= new_layer.end_layer;
//             match (in_old, in_new) {
//                 (true, true) => {}
//                 (_, _) => {}
//             }
//         } else {
//             out.push(LayerLayout {
//                 top: Transition {
//                     old_time,
//                     duration,
//                     old: prev_bottom.unwrap_or(new_layer.top.new),
//                     new: new_layer.top.new,
//                 },
//                 bottom: Transition {
//                     old_time,
//                     duration,
//                     old: prev_bottom.unwrap_or(new_layer.bottom.new),
//                     new: new_layer.bottom.new,
//                 },
//                 exists: Transition {
//                     old_time,
//                     duration,
//                     old: 0.,
//                     new: new_layer.exists.new,
//                 },
//                 ..new_layer.clone()
//             });
//         }
//     }
//     out
// }

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
    f32::max(
        0.0,
        f32::min(
            (time as f32 - val.old_time as f32) / val.duration as f32,
            1.0,
        ),
    )
}
