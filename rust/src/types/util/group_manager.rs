use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{HashMap, HashSet, LinkedList},
    fmt::Display,
    hash::Hash,
    iter::FromIterator,
    marker::PhantomData,
    rc::Rc,
    vec::IntoIter,
};

use itertools::Itertools;
use oxidd::{Edge, Function, InnerNode, LevelNo, Manager};
use oxidd_core::{DiagramRules, Node, Tag};
use priority_queue::PriorityQueue;

use crate::{
    util::{free_id_manager::FreeIdManager, logging::console, rc_refcell::MutRcRefCell},
    wasm_interface::{NodeGroupID, NodeID, TargetID, TargetIDType},
};

use super::{
    graph_structure::{
        graph_structure::{Change, DrawTag, EdgeType, GraphEventsReader, GraphStructure},
        grouped_graph_structure::{EdgeCountData, EdgeData, GroupedGraphStructure},
        oxidd_graph_structure::NodeLabel,
    },
    node_tracker_manager::{NodeTrackerM, NodeTrackerManager},
};

pub struct GroupManager<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> {
    node_label: PhantomData<NL>,
    level_label: PhantomData<LL>,
    graph: G,
    graph_events: GraphEventsReader,

    /// Nodes are implicitly in group 0 by default, I.e either:
    /// - group_by_id[group_id_by_node[node]].nodes.contains(node)
    /// - or !group_id_by_node.contains(node) && !exists g. group_by_id[g].nodes.contains(node)
    group_id_by_node: HashMap<NodeID, NodeGroupID>,
    group_by_id: HashMap<NodeGroupID, NodeGroup<T>>,
    // free_ids: FreeIdManager<usize>,
    // returned_ids: HashSet<usize>,
    /// Source trackers to manage sources obtained from the groupedGraphStructure
    group_ids: NodeTrackerManager,
}

type EdgeCounts<T: DrawTag> = HashMap<EdgeData<T>, usize>;

#[derive(Clone)]
struct ConnectionData<T: DrawTag> {
    parents: HashSet<(LevelNo, LevelNo, EdgeType<T>, NodeID)>,
    children: HashSet<(LevelNo, LevelNo, EdgeType<T>, NodeID)>,
}
impl<T: DrawTag> ConnectionData<T> {
    pub fn new() -> ConnectionData<T> {
        ConnectionData {
            parents: HashSet::new(),
            children: HashSet::new(),
        }
    }
}

pub struct NodeGroup<T: DrawTag> {
    nodes: HashMap<NodeID, ConnectionData<T>>,
    out_edges: EdgeCounts<T>,
    in_edges: EdgeCounts<T>,
    layer_min: PriorityQueue<NodeID, Reverse<LevelNo>>,
    layer_max: PriorityQueue<NodeID, LevelNo>,
}
impl<T: DrawTag> Display for NodeGroup<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let min = self
            .layer_min
            .peek()
            .map_or(0, |(node, Reverse(layer))| *layer);
        let max = self.layer_max.peek().map_or(0, |(node, layer)| *layer);
        write!(
            f,
            "group(nodes: [{}], levels: ({}, {})",
            self.nodes.keys().join(", "),
            min,
            max
        )
    }
}

// Helper methods
impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> GroupManager<T, NL, LL, G> {
    fn process_graph_events(&mut self) {
        let events = self.graph.consume_events(&self.graph_events);

        let mut removed_from = HashMap::<NodeID, NodeGroupID>::new();
        let mut used_sources = HashSet::<NodeID>::new();
        let mut refresh_data = HashSet::<NodeID>::new();
        for event in events {
            match event {
                Change::NodeLabelChange { node } => {
                    refresh_data.insert(node);
                }
                Change::LevelChange { node } => {
                    refresh_data.insert(node);
                }
                Change::NodeConnectionsChange { node } => {
                    refresh_data.insert(node);
                }
                Change::NodeRemoval { node } => {
                    let group = self.remove_node_from_group(node);
                    let source_group_exists = self.group_by_id.contains_key(&group); // If the node wasn't in the diagram to begin the group may not exist
                    if source_group_exists {
                        removed_from.insert(node, group);
                    }
                }
                Change::NodeInsertion { node, source } => {
                    let add_group = source.and_then(|source| {
                        // TODO: better way of deciding where to put a node

                        // Choose the most common parent group, if it has more than 1 item
                        let parents = self.graph.get_known_parents(node);
                        let parent_groups =
                            parents.iter().map(|&(_, parent)| self.get_group(parent));
                        let group_counts = parent_groups.fold(HashMap::new(), |mut acc, parent| {
                            *acc.entry(parent).or_insert(0) += 1;
                            acc
                        });
                        let max_group = group_counts.iter().max_by_key(|&(_, count)| count);
                        if let Some((&group, &count)) = max_group {
                            // if count > 1 {
                            //     return Some(group);
                            // }
                            if self.get_nodes_of_group(group).len() > 1 {
                                return Some(group);
                            } else {
                                return None;
                            }
                        }

                        // A source may have been removed and replaced by something (1 thing) else, in which case we want to place this thing int he original group
                        if !used_sources.contains(&source) {
                            used_sources.insert(source);
                            if let Some(&group_id) = removed_from.get(&source) {
                                return Some(group_id);
                            }
                        }

                        // // Otherwise, a source may have been in a group together with all its parents, in which case we don't want to separate the new node
                        // let source_group = removed_from
                        //     .get(&source)
                        //     .cloned()
                        //     .unwrap_or_else(|| self.get_group(source));
                        // let source_group_exists = self.group_by_id.contains_key(&source_group);
                        // let all_parents_in_group = source_group_exists
                        //     && self
                        //         .graph
                        //         .get_known_parents(node)
                        //         .iter()
                        //         .all(|(_, parent)| self.get_group(*parent) == source_group);
                        // if all_parents_in_group {
                        //     return Some(source_group);
                        // }

                        // Otherwise a new group may be created
                        None
                    });
                    if let Some(group) = add_group {
                        // // Don't add it to the group in which it's already implicitly present
                        // if group == 0 {
                        //     return;
                        // }
                        self.add_node_to_group(node, group);
                    } else {
                        let group_id =
                            self.create_group(vec![TargetID(TargetIDType::NodeID, node)]);

                        // Add a source for the new group
                        if let Some(source) = source {
                            if let Some(&source_group_id) = removed_from
                                .get(&source)
                                .or_else(|| self.group_id_by_node.get(&source))
                            {
                                self.group_ids.add_source(group_id, source_group_id);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // We batch connection changes at the end, since the graph may already refer to newly inserted nodes (that were inserted after the connection change)
        for node in refresh_data {
            let group = self.remove_node_from_group(node);
            let group_exists = self.group_by_id.contains_key(&group);
            if group_exists {
                self.add_node_to_group(node, group);
            }
        }
        console::log!("after events");

        for group_id in removed_from.values().cloned() {
            self.remove_group_if_empty(group_id);
        }
    }

    fn get_node_group_id(&self, node: NodeID) -> NodeGroupID {
        if let Some(group_id) = self.group_id_by_node.get(&node) {
            *group_id
        } else {
            0
        }
    }

    fn remove_group(&mut self, id: NodeGroupID) {
        self.group_by_id.remove(&id);
        self.group_ids.make_available(id);
    }

    fn remove_edges_to_set(edges: &mut EdgeCounts<T>, edge_data: EdgeData<T>, count: usize) {
        if let Some(cur_count) = edges.get_mut(&edge_data) {
            *cur_count -= count;
            if *cur_count <= 0 {
                edges.remove(&edge_data);
            }
        }
    }

    fn remove_edges(
        &mut self,
        from: NodeGroupID,
        from_source: NodeID,
        from_level: LevelNo,
        to: NodeGroupID,
        to_source: NodeID,
        to_level: LevelNo,
        edge_type: EdgeType<T>,
        count: usize,
    ) {
        if let Some(from_group) = self.group_by_id.get_mut(&from) {
            GroupManager::<T, NL, LL, G>::remove_edges_to_set(
                &mut from_group.out_edges,
                EdgeData::new(to, from_level, to_level, edge_type),
                count,
            );
            from_group.nodes.entry(from_source).and_modify(|cd| {
                cd.children
                    .remove(&(from_level, to_level, edge_type, to_source));
            });
        }

        if let Some(to_group) = self.group_by_id.get_mut(&to) {
            GroupManager::<T, NL, LL, G>::remove_edges_to_set(
                &mut to_group.in_edges,
                EdgeData::new(from, to_level, from_level, edge_type),
                count,
            );
            to_group.nodes.entry(to_source).and_modify(|cd| {
                cd.parents
                    .remove(&(from_level, to_level, edge_type, from_source));
            });
        }
    }

    fn add_edges_to_set(edges: &mut EdgeCounts<T>, edge_data: EdgeData<T>, count: usize) {
        let cur_count = edges.entry(edge_data).or_insert(0);
        *cur_count += count;
    }

    fn add_edges(
        &mut self,
        from: NodeGroupID,
        from_source: NodeID,
        to: NodeGroupID,
        to_source: NodeID,
        edge_type: EdgeType<T>,
        count: usize,
    ) {
        let from_level = self.graph.get_level(from_source);
        let to_level = self.graph.get_level(to_source);

        if let Some(from_group) = self.group_by_id.get_mut(&from) {
            GroupManager::<T, NL, LL, G>::add_edges_to_set(
                &mut from_group.out_edges,
                EdgeData::new(to, from_level, to_level, edge_type),
                count,
            );
            from_group
                .nodes
                .entry(from_source)
                .or_insert_with(|| ConnectionData::new())
                .children
                .insert((from_level, to_level, edge_type, to_source));
        }

        if let Some(to_group) = self.group_by_id.get_mut(&to) {
            GroupManager::<T, NL, LL, G>::add_edges_to_set(
                &mut to_group.in_edges,
                EdgeData::new(from, to_level, from_level, edge_type),
                count,
            );
            to_group
                .nodes
                .entry(to_source)
                .or_insert_with(|| ConnectionData::new())
                .parents
                .insert((from_level, to_level, edge_type, from_source));
        }
    }

    fn remove_node_from_group(&mut self, node: NodeID) -> NodeGroupID {
        let cur_group_id = self.get_node_group_id(node);
        if cur_group_id == 0 && !self.group_by_id.contains_key(&0) {
            return 0;
        }

        // Check if the node is explicitly contained (instead of implicitly, which can happen for the 0 group)
        let Some(cur_group) = self.group_by_id.get(&cur_group_id) else {
            console::log!(
                "cur group {}, for node {}, does not exist",
                cur_group_id,
                node
            );

            return cur_group_id;
        };
        let contained = cur_group.nodes.contains_key(&node);

        // Remove old edges
        let Some(cur_group) = self.group_by_id.get_mut(&cur_group_id) else {
            return cur_group_id;
        };
        if let Some(connections) = cur_group.nodes.get(&node) {
            let connections = connections.clone();
            for (from_level, to_level, edge_type, child_id) in connections.children {
                let child_group_id = self.get_node_group_id(child_id);
                if contained && cur_group_id != child_group_id {
                    self.remove_edges(
                        cur_group_id,
                        node,
                        from_level,
                        child_group_id,
                        child_id,
                        to_level,
                        edge_type,
                        1,
                    );
                }
            }

            for (from_level, to_level, edge_type, parent_id) in connections.parents {
                let parent_group_id = self.get_node_group_id(parent_id);
                if contained && parent_group_id != cur_group_id {
                    self.remove_edges(
                        parent_group_id,
                        parent_id,
                        from_level,
                        cur_group_id,
                        node,
                        to_level,
                        edge_type,
                        1,
                    );
                }
            }
        }

        // Remove from group
        let Some(cur_group) = self.group_by_id.get_mut(&cur_group_id) else {
            return cur_group_id;
        };
        cur_group.nodes.remove(&node);
        cur_group.layer_min.remove(&node);
        cur_group.layer_max.remove(&node);

        // Default node to be in group 0
        self.group_id_by_node.insert(node, 0);

        cur_group_id
    }

    fn remove_group_if_empty(&mut self, group_id: NodeGroupID) {
        let Some(cur_group) = self.group_by_id.get_mut(&group_id) else {
            return;
        };
        let from_empty = cur_group.nodes.is_empty();
        if from_empty {
            self.remove_group(group_id);
            console::log!("removed {}", group_id);
        }
    }

    fn add_node_to_group(&mut self, node: NodeID, group_id: NodeGroupID) {
        let from_level = self.graph.get_level(node);

        // Add to new
        self.group_id_by_node.insert(node, group_id);
        if let Some(new_group) = self.group_by_id.get_mut(&group_id) {
            new_group.nodes.insert(node, ConnectionData::new());
            new_group.layer_min.push(node, Reverse(from_level));
            new_group.layer_max.push(node, from_level);
        }

        // Add new connections
        for (edge_type, child_id) in self.graph.get_children(node) {
            let child_group_id = self.get_node_group_id(child_id);
            let child_level = self.graph.get_level(child_id);

            if group_id != child_group_id {
                self.add_edges(group_id, node, child_group_id, child_id, edge_type, 1);
            }

            // Ensure the child id is in there, which may not have been the case initially for the initial group 0
            if child_group_id == 0 {
                let Some(child_group) = self.group_by_id.get_mut(&child_group_id) else {
                    continue;
                };
                if !child_group.nodes.contains_key(&child_id) {
                    child_group.nodes.insert(child_id, ConnectionData::new());
                    child_group.layer_min.push(child_id, Reverse(child_level));
                    child_group.layer_max.push(child_id, child_level);
                }
            }
        }

        for (edge_type, parent_id) in self.graph.get_known_parents(node) {
            let parent_group_id = self.get_node_group_id(parent_id);

            if parent_group_id != group_id {
                self.add_edges(parent_group_id, parent_id, group_id, node, edge_type, 1);
            }
        }
    }
}

// Main methods
impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> GroupManager<T, NL, LL, G> {
    pub fn new(mut graph: G) -> GroupManager<T, NL, LL, G> {
        let root_id = graph.get_root();
        let root_level = graph.get_level(root_id);
        let mut group_ids = NodeTrackerManager::new(1);
        group_ids.add_group_id(0);
        GroupManager {
            level_label: PhantomData,
            node_label: PhantomData,
            graph_events: graph.create_event_reader(),
            graph,
            group_id_by_node: HashMap::new(),
            group_by_id: HashMap::from([(
                0,
                NodeGroup {
                    nodes: HashMap::from([(root_id, ConnectionData::new())]),
                    out_edges: HashMap::new(),
                    in_edges: HashMap::new(),
                    layer_min: {
                        let mut q = PriorityQueue::new();
                        q.push(root_id, Reverse(root_level));
                        q
                    },
                    layer_max: {
                        let mut q = PriorityQueue::new();
                        q.push(root_id, root_level);
                        q
                    },
                },
            )]),
            group_ids,
        }
    }

    pub fn get_groups(&self) -> &HashMap<NodeGroupID, NodeGroup<T>> {
        &self.group_by_id
    }

    pub fn set_group(
        &mut self,
        from: Vec<crate::wasm_interface::TargetID>,
        to: crate::wasm_interface::NodeGroupID,
    ) -> bool {
        if let None = self.group_by_id.get_mut(&to) {
            return false;
        }

        for item in from {
            let from_id_type = item.0;
            let from_id = item.1;
            if from_id_type == TargetIDType::NodeID {
                let old_group_id = self.remove_node_from_group(from_id);
                self.add_node_to_group(from_id, to);
                self.remove_group_if_empty(old_group_id);
            } else if from_id == to {
                continue;
            } else if from_id == 0 {
                let Some(group) = self.group_by_id.get_mut(&from_id) else {
                    continue;
                };
                let init_nodes = group.nodes.keys();
                let mut found: HashSet<NodeID> = init_nodes.clone().cloned().collect();
                let mut queue: LinkedList<NodeID> = init_nodes.cloned().collect();

                while !queue.is_empty() {
                    let node_id = queue.pop_front().unwrap();
                    for (_, child_id) in self.graph.get_children(node_id) {
                        if found.contains(&child_id) {
                            continue;
                        }

                        found.insert(child_id);
                        queue.push_back(child_id);
                    }
                }

                self.set_group(
                    found
                        .into_iter()
                        .map(|id| TargetID(TargetIDType::NodeID, id))
                        .collect(),
                    to,
                );
            } else if let Some(from_group) = self.group_by_id.get(&from_id) {
                self.set_group(
                    from_group
                        .nodes
                        .keys()
                        .map(|id| TargetID(TargetIDType::NodeID, *id))
                        .collect(),
                    to,
                );

                // let from_group = &self.get_node_group(from_id);
                // let out_edges = from_group.out_edges.clone();
                // let in_edges = from_group.in_edges.clone();
                // let from_nodes = from_group.nodes.clone();
                // let layer_min: Vec<(NodeID, Reverse<LevelNo>)> =
                //     from_group.layer_min.iter().map(|(&n, &i)| (n, i)).collect();
                // let layer_max: Vec<(NodeID, LevelNo)> =
                //     from_group.layer_max.iter().map(|(&n, &i)| (n, i)).collect();

                // for edge_data in out_edges.keys() {
                //     let count = out_edges[edge_data];
                //     let EdgeData {
                //         to: edge_to,
                //         from_level,
                //         to_level,
                //         edge_type,
                //     } = *edge_data;

                //     self.remove_edges(from_id, from_level, edge_to, to_level, edge_type, count);
                //     self.add_edges(to, from_level, edge_to, to_level, edge_type, count);
                // }
                // for edge_data in in_edges.keys() {
                //     let count = in_edges[edge_data];
                //     let EdgeData {
                //         to: edge_to,
                //         from_level,
                //         to_level,
                //         edge_type,
                //     } = *edge_data;

                //     self.remove_edges(edge_to, to_level, from_id, from_level, edge_type, count);
                //     self.add_edges(edge_to, to_level, to, from_level, edge_type, count);
                // }

                // for from_node in from_nodes.keys() {
                //     self.group_id_by_node.insert(*from_node, to);
                // }
                // let to_group = self.get_node_group_mut(to);
                // to_group.nodes.extend(from_nodes);
                // to_group.layer_min.extend(layer_min);
                // to_group.layer_max.extend(layer_max);

                // self.remove_group(from_id);
            } else {
                return false;
            }
        }

        return true;
    }

    pub fn create_group(
        &mut self,
        from: Vec<crate::wasm_interface::TargetID>,
    ) -> crate::wasm_interface::NodeGroupID {
        let maybe_source = from
            .get(0)
            .map(|&TargetID(target_type, id)| match target_type {
                TargetIDType::NodeID => self.get_node_group_id(id),
                _ => id,
            });

        let new_id = self.group_ids.get_free_group_id_and_add();
        self.group_by_id.insert(
            new_id,
            NodeGroup {
                nodes: HashMap::new(),
                in_edges: HashMap::new(),
                out_edges: HashMap::new(),
                layer_min: PriorityQueue::new(),
                layer_max: PriorityQueue::new(),
            },
        );
        self.set_group(from, new_id);
        if let Some(source) = maybe_source {
            let source_exists = self.group_by_id.contains_key(&source);
            if source_exists {
                self.group_ids.add_source(new_id, source);
            }
        }
        new_id
    }

    pub fn split_edges(&mut self, node_ids: &[NodeID], fully: bool) {
        // TODO: come up with a better splitting approach that considers nodes together
        let mut split = HashSet::new();
        for &node_id in node_ids {
            let children = &self.graph.get_children(node_id);
            for &(_, child) in children {
                if split.contains(&child) {
                    continue;
                }

                split.insert(child);
                if self.get_nodes_of_group(self.get_group(child)).len() == 1 {
                    continue;
                }
                self.create_group(vec![TargetID(TargetIDType::NodeID, child)]);
            }
        }
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>>
    GroupedGraphStructure<T, Vec<NL>, LL> for GroupManager<T, NL, LL, G>
{
    type Tracker = NodeTrackerM;
    fn get_root(&self) -> NodeGroupID {
        let root_node = &self.graph.get_root();
        self.get_node_group_id(*root_node)
    }

    fn get_all_groups(&self) -> Vec<NodeGroupID> {
        self.group_by_id.keys().into_iter().map(|&id| id).collect()
    }

    fn get_hidden(&self) -> Option<NodeGroupID> {
        if self.group_by_id.contains_key(&0) {
            Some(0)
        } else {
            None
        }
    }

    fn get_parents(&self, group: NodeGroupID) -> IntoIter<EdgeCountData<T>> {
        self.group_by_id.get(&group).map_or_else(
            || Vec::default().into_iter(),
            |group| {
                group
                    .in_edges
                    .iter()
                    .map(
                        |(
                            &EdgeData {
                                to,
                                from_level,
                                to_level,
                                edge_type,
                            },
                            &count,
                        )| {
                            EdgeCountData::new(to, from_level, to_level, edge_type, count)
                        },
                    )
                    .collect::<Vec<EdgeCountData<T>>>()
                    .into_iter()
            },
        )
    }

    fn get_children(&self, group: NodeGroupID) -> IntoIter<EdgeCountData<T>> {
        self.group_by_id.get(&group).map_or_else(
            || Vec::default().into_iter(),
            |group| {
                group
                    .out_edges
                    .iter()
                    .map(
                        |(
                            &EdgeData {
                                to,
                                from_level,
                                to_level,
                                edge_type,
                            },
                            &count,
                        )| {
                            EdgeCountData::new(to, from_level, to_level, edge_type, count)
                        },
                    )
                    .collect::<Vec<EdgeCountData<T>>>()
                    .into_iter()
            },
        )
    }

    fn get_level_range(&self, group_id: NodeGroupID) -> (oxidd::LevelNo, oxidd::LevelNo) {
        self.group_by_id.get(&group_id).map_or_else(
            || (0, 0),
            |group| {
                let min = group
                    .layer_min
                    .peek()
                    .map_or(0, |(node, Reverse(layer))| *layer);
                let max = group.layer_max.peek().map_or(0, |(node, layer)| *layer);
                (min, max)
            },
        )
    }

    fn get_nodes_of_group(&self, group: NodeGroupID) -> IntoIter<NodeID> {
        self.group_by_id
            .get(&group)
            .map_or_else(
                || Vec::default().into_iter(),
                |group| group.nodes.keys().cloned().collect_vec().into_iter(),
            )
            .sorted()
    }

    fn get_group(&self, node: NodeID) -> NodeGroupID {
        self.get_node_group_id(node)
    }

    fn create_node_tracker(&mut self) -> Self::Tracker {
        self.group_ids.create_reader()
    }

    fn get_level_label(&self, level: LevelNo) -> LL {
        self.graph.get_level_label(level)
    }

    fn get_group_label(&self, group_id: NodeID) -> Vec<NL> {
        self.group_by_id.get(&group_id).map_or_else(
            || Vec::default(),
            |group| {
                group
                    .nodes
                    .keys()
                    .map(|&node_id| self.graph.get_node_label(node_id))
                    .collect_vec()
            },
        )
    }

    fn refresh(&mut self) {
        self.process_graph_events();
    }
}
