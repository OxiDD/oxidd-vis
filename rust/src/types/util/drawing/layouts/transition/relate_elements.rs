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

pub struct ElementRelations<T: DrawTag> {
    /// A mapping some groups in the new layout, to the group in the old layout it originates from
    pub previous_groups: HashMap<NodeGroupID, TargetGroup>,
    /// A subset of the groups in the old layout which have now been deleted in the new layout, and optionally a related target group in the new layout that it turned into
    pub deleted_groups: HashMap<NodeGroupID, Option<TargetGroup>>,
    /// A mapping from edges in the new layout, to the edge in the old layout it originates from
    pub previous_edges: HashMap<(NodeGroupID, EdgeData<T>), TargetEdge<T>>,
    /// A subset of the edges in the old layout which have now been deleted in the new layout (which can be both in mapped and deleted nodes), and optionally a related target edge in the old layout that retains its shape.
    /// Note that all ids in the map are in terms of the new layout, including edge to data (even tho it might have originated from another old node, findable using the `previous_groups` map)
    pub deleted_edges: HashMap<NodeGroupID, Vec<DeletedEdge<T>>>,
}

pub struct TargetGroup {
    /// The ID of the group that is targeted
    pub id: NodeGroupID,
    /// The offset that the new group has in relation to this node (to compensate for layer shifts)
    pub offset: Point,
    /// Whether the group that we relate to this target represents this group
    pub represents: bool,
}

#[derive(Clone)]
pub struct DeletedEdge<T: DrawTag> {
    /// The edge data that was deleted
    pub edge_data: EdgeData<T>,
    /// Whether the deleted edge should simply morph to a straight edge and apply the given start and end offset, rather than fading out
    pub morph: Option<DeleteMorph>,
}

#[derive(Clone)]
pub struct DeleteMorph {
    /// An offset adjustment for the start of the edge
    pub start_offset: Point,
    /// An offset adjustment for the end of the edge
    pub end_offset: Point,
}

#[derive(Clone)]
pub struct TargetEdge<T: DrawTag> {
    /// The ID of the group that the edge originates from
    pub group_id: NodeGroupID,
    /// The edge of the group that's targetted
    pub edge_data: EdgeData<T>,
    /// An offset adjustment for the start of the edge
    pub start_offset: Point,
    /// An offset adjustment for the end of the edge
    pub end_offset: Point,
}

/// Relates new elements and old elements to one and another, to be used in making smooth transitions
pub fn relate_elements<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>>(
    graph: &G,
    old: &DiagramLayout<T>,
    new: &DiagramLayout<T>,
    sources: &G::Tracker,
    time: u32,
) -> ElementRelations<T> {
    let mut previous_nodes = map_groups::<T, GL, LL, G>(old, new, sources, time);
    select_node_representation(&mut previous_nodes, &new);
    let deleted_nodes =
        find_deleted_nodes::<T, GL, LL, G>(old, new, sources, &previous_nodes, time);
    let previous_edges = map_edges(graph, old, new, sources, &previous_nodes, time);
    let deleted_edges = find_deleted_edges::<T, GL, LL, G>(
        old,
        new,
        sources,
        &previous_nodes,
        &deleted_nodes,
        &previous_edges,
        time,
    );

    // TODO: set represents values in previous and deleted nodes
    ElementRelations {
        previous_groups: previous_nodes,
        deleted_groups: deleted_nodes,
        previous_edges,
        deleted_edges,
    }
}

pub fn map_groups<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>>(
    old: &DiagramLayout<T>,
    new: &DiagramLayout<T>,
    sources: &G::Tracker,
    time: u32,
) -> HashMap<NodeGroupID, TargetGroup> {
    new.groups
        .iter()
        .map(|(&group_id, data)| {
            let new_y_range = data.get_rect(Some(time)).y_range();

            (
                group_id,
                if let Some(old_group_layout) = old.groups.get(&group_id) {
                    let source_y_range = old_group_layout.get_rect(Some(time)).y_range();
                    let restricted_range = new_y_range.align_within(&source_y_range);
                    TargetGroup {
                        id: group_id,
                        offset: Point {
                            x: 0.,
                            y: restricted_range.start - source_y_range.start,
                        },
                        represents: false,
                    }
                } else {
                    let node_sources = sources.get_sources(group_id);
                    let closest = node_sources
                        .iter()
                        .filter_map(|source| {
                            old.groups.get(source).map(|source_data| {
                                let source_y_range = source_data.get_rect(Some(time)).y_range();
                                let restricted_range = new_y_range.align_within(&source_y_range);
                                let height_delta =
                                    (source_y_range.size() - new_y_range.size()).abs();
                                (
                                    source,
                                    restricted_range.start - source_y_range.start,
                                    height_delta,
                                )
                            })
                        })
                        .min_by(|(_, a_pos, a_height), (_, b_pos, b_height)| {
                            let height_comp = a_height.total_cmp(b_height);
                            if !height_comp.is_eq() {
                                height_comp
                            } else {
                                a_pos.total_cmp(&b_pos)
                            }
                        }); // TODO: choose an appropriate criteria for choosing the best source
                    if let Some((&source, shift, _)) = closest {
                        TargetGroup {
                            id: source,
                            offset: Point { x: 0., y: shift },
                            represents: false,
                        }
                    } else {
                        console::log!("Detect this no shift");
                        TargetGroup {
                            id: group_id,
                            offset: Point { x: 0., y: 0. },
                            represents: false,
                        }
                    }
                },
            )
        })
        .collect()
}

pub fn select_node_representation<T: DrawTag>(
    previous_nodes: &mut HashMap<NodeGroupID, TargetGroup>,
    new: &DiagramLayout<T>,
) {
    let mut reverse_node_mapping: HashMap<NodeGroupID, HashSet<NodeGroupID>> = HashMap::new();
    for (&node, source_target) in previous_nodes.iter() {
        reverse_node_mapping
            .entry(source_target.id)
            .or_insert_with(|| HashSet::new())
            .insert(node);
    }

    for (_, images) in reverse_node_mapping {
        let sizes = images
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
        previous_nodes.entry(*node).and_modify(|source_target| {
            source_target.offset = Point { x: 0., y: 0. };
            source_target.represents = true;
        });
    }
}

pub fn map_edges<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>>(
    graph: &G,
    old: &DiagramLayout<T>,
    new: &DiagramLayout<T>,
    sources: &G::Tracker,
    previous_nodes: &HashMap<NodeGroupID, TargetGroup>,
    time: u32,
) -> HashMap<(NodeGroupID, EdgeData<T>), TargetEdge<T>> {
    let mut edge_mapping = HashMap::<(NodeGroupID, EdgeData<T>), TargetEdge<T>>::new();
    for (&group_id, _group_layout) in new.groups.iter() {
        let group_sources = sources.get_sources(group_id);
        let group_sources = group_sources.iter().chain(once(&group_id));

        for edge in graph.get_children(group_id) {
            let edge = edge.drop_count();

            let to_group_sources = sources.get_sources(edge.to);
            let to_group_sources = to_group_sources.iter().chain(once(&edge.to));

            for (&source, &to_source) in group_sources.clone().cartesian_product(to_group_sources) {
                let mut to_level = edge.to_level;

                // Try to account for terminals that move between layers:
                if let Some(to_level_range) =
                    new.groups.get(&edge.to).map(|group| group.level_range)
                {
                    if to_level_range.0 == to_level_range.1 {
                        if let Some(to_source_level_range) =
                            old.groups.get(&to_source).map(|group| group.level_range)
                        {
                            if to_source_level_range.0 == to_source_level_range.1 {
                                to_level = to_level - to_level_range.0 + to_source_level_range.0
                            }
                        }
                    }
                };

                let old_edge = &EdgeData {
                    to: to_source,
                    to_level,
                    ..edge
                };

                let Some((old_edge, _old_edge_layout)) =
                    old.groups.get(&source).and_then(|group| {
                        group
                            .edges
                            .get(&old_edge)
                            .map(|edge_layout| (old_edge, edge_layout))
                    })
                else {
                    continue;
                };

                let start_offset = previous_nodes
                    .get(&group_id)
                    .map(|source_prev| {
                        old.groups
                            .get(&source_prev.id)
                            .zip(old.groups.get(&source))
                            .map(|(source_prev_layout, source_layout)| {
                                source_prev_layout.position.get(time)
                                    - source_layout.position.get(time)
                            })
                            .unwrap_or_default()
                            + source_prev.offset
                    })
                    .unwrap_or_default();

                let end_offset = previous_nodes
                    .get(&edge.to)
                    .map(|to_source_prev| {
                        old.groups
                            .get(&to_source_prev.id)
                            .zip(old.groups.get(&to_source))
                            .map(|(to_source_prev_layout, to_source_layout)| {
                                to_source_prev_layout.position.get(time)
                                    - to_source_layout.position.get(time)
                            })
                            .unwrap_or_default()
                            + to_source_prev.offset
                    })
                    .unwrap_or_default();

                edge_mapping.insert(
                    (group_id, edge.clone()),
                    TargetEdge {
                        group_id: source,
                        edge_data: old_edge.clone(),
                        start_offset,
                        end_offset,
                    },
                );

                break;
            }
        }
    }
    edge_mapping
}

pub fn find_deleted_nodes<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>>(
    old: &DiagramLayout<T>,
    new: &DiagramLayout<T>,
    sources: &G::Tracker,
    previous_nodes: &HashMap<NodeGroupID, TargetGroup>,
    time: u32,
) -> HashMap<NodeGroupID, Option<TargetGroup>> {
    let all_images = new
        .groups
        .keys()
        .flat_map(|&group_id| {
            sources
                .get_sources(group_id)
                .iter()
                .map(|&source| (source, group_id))
                .collect_vec()
        })
        .collect::<MultiMap<NodeGroupID, NodeGroupID>>();

    let used_old_groups = new
        .groups
        .iter()
        .filter_map(|(id, _)| previous_nodes.get(&id).map(|prev_group| prev_group.id))
        .collect::<HashSet<_>>();
    let deleted_groups = old.groups.iter().filter(|(id, group)| {
        let not_used = !used_old_groups.contains(*id);
        let still_exists = group.exists.get(time) > 0.;
        not_used && still_exists
    });
    deleted_groups
        .map(|(&group_id, group_layout)| {
            (
                group_id,
                all_images
                    .get_vec(&group_id)
                    .and_then(|group_images| {
                        group_images
                            .iter()
                            .filter_map(|&image_group_id| {
                                Some(image_group_id).zip(new.groups.get(&image_group_id).cloned())
                            })
                            // Choose one node it should disappear into
                            .next()
                    })
                    .map(|(target, target_layout)| {
                        let y_range = group_layout.get_rect(Some(time)).y_range();
                        let target_y_range = target_layout.get_rect(Some(time)).y_range();
                        let restricted_y_range = y_range.bounded_to(&target_y_range);
                        TargetGroup {
                            id: target,
                            offset: Point {
                                x: 0.,
                                y: restricted_y_range.start - target_y_range.start,
                            },
                            represents: false,
                        }
                    }),
            )
        })
        .collect()
}

pub fn find_deleted_edges<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>>(
    old: &DiagramLayout<T>,
    new: &DiagramLayout<T>,
    sources: &G::Tracker,
    previous_nodes: &HashMap<NodeGroupID, TargetGroup>,
    deleted_nodes: &HashMap<NodeGroupID, Option<TargetGroup>>,
    previous_edges: &HashMap<(NodeGroupID, EdgeData<T>), TargetEdge<T>>,
    time: u32,
) -> HashMap<NodeGroupID, Vec<DeletedEdge<T>>> {
    let mut reverse_node_mapping: HashMap<usize, usize> = HashMap::new();

    // let mut reverse_node_mapping: HashMap<usize, usize> = HashMap::new();
    let mut deleted_edges: HashSet<(NodeGroupID, &EdgeData<T>)> = old
        .groups
        .iter()
        .map(|(&group_id, group_layout)| {
            once(group_id).cartesian_product(group_layout.edges.keys())
        })
        .flatten()
        .collect();
    for (&group_id, prev_group) in previous_nodes {
        let insert =
            !reverse_node_mapping.contains_key(&prev_group.id) || group_id == prev_group.id;
        if insert {
            reverse_node_mapping.insert(prev_group.id, group_id);
        }

        for edge in new
            .groups
            .get(&group_id)
            .map(|group_layout| group_layout.edges.keys().collect_vec())
            .unwrap_or_default()
        {
            if let Some(edge_target) = previous_edges.get(&(group_id, edge.clone())) {
                deleted_edges.remove(&(edge_target.group_id, &edge_target.edge_data));
            }
        }
    }
    let deleted_edges = deleted_edges
        .into_iter()
        .map(|(source_group_id, old_edge)| {
            let new_to = reverse_node_mapping
                .get(&old_edge.to)
                .cloned()
                .unwrap_or(old_edge.to);

            let new_edge = EdgeData {
                to: new_to,
                ..old_edge.clone()
            };

            let new_from = reverse_node_mapping
                .get(&source_group_id)
                .cloned()
                .unwrap_or(source_group_id);

            let from_hide_group = deleted_nodes
                .get(&new_from)
                .and_then(|target| target.as_ref())
                .map(|target| target.id)
                .unwrap_or(new_from);
            let to_hide_group = deleted_nodes
                .get(&new_to)
                .and_then(|target| target.as_ref())
                .map(|target| target.id)
                .unwrap_or(new_to);
            if from_hide_group == to_hide_group {
                // We compute the movement delta to make edges that shift layers because of a delete transition remain on the same vertical position
                let from_delta_y = previous_nodes
                    .get(&new_from)
                    .and_then(|target| old.groups.get(&target.id).zip(new.groups.get(&new_from)))
                    .map(|(old_group, new_group)| {
                        old_group.position.get(time).y - new_group.position.new.y
                    })
                    .unwrap_or_default();
                let to_delta_y = previous_nodes
                    .get(&new_to)
                    .and_then(|target| old.groups.get(&target.id).zip(new.groups.get(&new_to)))
                    .map(|(old_group, new_group)| {
                        old_group.position.get(time).y - new_group.position.new.y
                    })
                    .unwrap_or_default();
                (
                    (new_from, new_edge),
                    Some(DeleteMorph {
                        start_offset: Point {
                            x: 0.,
                            y: from_delta_y,
                        },
                        end_offset: Point {
                            x: 0.,
                            y: to_delta_y,
                        },
                    }),
                )
            } else {
                ((new_from, new_edge), None)
            }
        })
        .collect::<HashMap<_, _>>();

    let mut out_delete_edges = HashMap::new();
    for ((group, edge_data), mapping) in deleted_edges {
        out_delete_edges
            .entry(group)
            .or_insert_with(Vec::new)
            .push(DeletedEdge {
                edge_data,
                morph: mapping,
            });
    }

    out_delete_edges
}
