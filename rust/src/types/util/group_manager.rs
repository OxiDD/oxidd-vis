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
        graph_structure::{Change, DrawTag, EdgeType, GraphStructure},
        grouped_graph_structure::{EdgeCountData, EdgeData, GroupedGraphStructure},
    },
    source_tracker_manager::{SourceReader, SourceTrackerManager},
};

pub struct GroupManager<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> {
    // This wrapper is required for the listener to have access to the internal functions
    inner: MutRcRefCell<InnerGroupManager<T, NL, LL, G>>,
    listener_handle: Option<usize>,
}

impl<
        T: DrawTag + 'static,
        NL: Clone + 'static,
        LL: Clone + 'static,
        G: GraphStructure<T, NL, LL> + 'static,
    > GroupManager<T, NL, LL, G>
{
    pub fn new(graph: G) -> GroupManager<T, NL, LL, G> {
        let mut group_manager = GroupManager {
            inner: MutRcRefCell::new(InnerGroupManager::new(graph)),
            listener_handle: None,
        };
        group_manager.setup_listener();
        group_manager
    }

    pub fn set_group(
        &mut self,
        from: Vec<crate::wasm_interface::TargetID>,
        to: crate::wasm_interface::NodeGroupID,
    ) -> bool {
        self.inner.get().set_group(from, to)
    }

    pub fn create_group(
        &mut self,
        from: Vec<crate::wasm_interface::TargetID>,
    ) -> crate::wasm_interface::NodeGroupID {
        self.inner.get().create_group(from)
    }

    pub fn split_edges(&mut self, group_id: NodeGroupID, fully: bool) {
        self.inner.get().split_edges(group_id, fully)
    }

    pub fn setup_listener(&mut self) {
        let inner = self.inner.clone();
        self.listener_handle = Some(self.inner.get().graph.on_change(Box::new(move |events| {
            let mut removed_from = HashMap::new();
            let mut used_sources = HashSet::new();
            for event in events {
                let mut inner = inner.get();
                match event {
                    Change::NodeLabelChange { node } => {}
                    Change::LevelChange { node } => {}
                    Change::LevelLabelChange { level } => {}
                    Change::NodeConnectionsChange { node } => {
                        // TODO: handle connection changes, node removals, and node insertions
                        let group = inner.remove_node_from_group(*node);
                        inner.add_node_to_group(*node, group);
                    }
                    Change::NodeRemoval { node } => {
                        let group = inner.remove_node_from_group(*node);
                        removed_from.insert(node, group);
                    }
                    Change::NodeInsertion { node, source } => {
                        let add_group = source.and_then(|source| {
                            // A source may have been removed and replaced by something (1 thing) else, in which case we want to place this thing int he original group
                            if !used_sources.contains(&source) {
                                used_sources.insert(source);
                                return removed_from.get(&source).cloned();
                            }

                            // Otherwise, a source may have been in a group together with all its parents, in which case we don't want to separate the new node
                            let source_group = inner.get_group(source);
                            let all_parents_in_group = inner
                                .graph
                                .get_known_parents(source)
                                .iter()
                                .all(|(_, parent)| inner.get_group(*parent) == source_group);
                            if all_parents_in_group {
                                return Some(source_group);
                            }

                            // Otherwise a new group may be created
                            None
                        });
                        if let Some(group) = add_group {
                            // Don't add it to the group in which it's already implicitly present
                            if group == 0 {
                                return;
                            }
                            inner.add_node_to_group(*node, group);
                        } else {
                            inner.create_group(vec![TargetID(TargetIDType::NodeID, *node)]);
                        }
                    }
                }
            }

            for group_id in removed_from.values().cloned() {
                inner.get().remove_group_if_empty(group_id);
            }
        })));
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> Drop
    for GroupManager<T, NL, LL, G>
{
    fn drop(&mut self) {
        if let Some(listener_handle) = self.listener_handle {
            self.inner.get().graph.off_change(listener_handle);
        }
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>>
    GroupedGraphStructure<T, String, LL> for GroupManager<T, NL, LL, G>
{
    type Tracker = SourceReader;

    fn get_root(&self) -> NodeGroupID {
        self.inner.read().get_root()
    }

    fn get_all_groups(&self) -> Vec<NodeGroupID> {
        self.inner.read().get_all_groups()
    }

    fn get_hidden(&self) -> Option<NodeGroupID> {
        self.inner.read().get_hidden()
    }

    fn get_group(&self, node: NodeID) -> NodeGroupID {
        self.inner.read().get_group(node)
    }

    fn get_group_label(&self, node: NodeID) -> String {
        self.inner.read().get_group_label(node)
    }

    fn get_parents(&self, group: NodeGroupID) -> IntoIter<EdgeCountData<T>> {
        self.inner.read().get_parents(group)
    }

    fn get_children(&self, group: NodeGroupID) -> IntoIter<EdgeCountData<T>> {
        self.inner.read().get_children(group)
    }

    fn get_nodes_of_group(&self, group: NodeGroupID) -> IntoIter<NodeID> {
        self.inner.read().get_nodes_of_group(group)
    }

    fn get_level_range(&self, group: NodeGroupID) -> (LevelNo, LevelNo) {
        self.inner.read().get_level_range(group)
    }

    fn get_level_label(&self, level: LevelNo) -> LL {
        self.inner.read().get_level_label(level)
    }

    fn get_source_reader(&mut self) -> Self::Tracker {
        self.inner.get().get_source_reader()
    }
}

pub struct InnerGroupManager<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> {
    node_label: PhantomData<NL>,
    level_label: PhantomData<LL>,
    graph: G,
    /// Nodes are implicitly in group 0 by default, I.e either:
    /// - group_by_id[group_id_by_node[node]].nodes.contains(node)
    /// - or !group_id_by_node.contains(node) && !exists g. group_by_id[g].nodes.contains(node)
    group_id_by_node: HashMap<NodeID, NodeGroupID>,
    group_by_id: HashMap<NodeGroupID, NodeGroup<T>>,
    free_ids: FreeIdManager<usize>,
    returned_ids: HashSet<usize>,
    /// Source trackers to manage sources obtained from the groupedGraphStructure
    sources: SourceTrackerManager,
}

type EdgeCounts<T: DrawTag> = HashMap<EdgeData<T>, usize>;

#[derive(Clone)]
struct ConnectionData<T: DrawTag> {
    parents: HashSet<(EdgeType<T>, NodeID)>,
    children: HashSet<(EdgeType<T>, NodeID)>,
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
impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>>
    InnerGroupManager<T, NL, LL, G>
{
    fn get_node_group_mut(&mut self, group_id: NodeGroupID) -> &mut NodeGroup<T> {
        self.group_by_id.get_mut(&group_id).unwrap()
    }

    fn get_node_group(&self, group_id: NodeGroupID) -> &NodeGroup<T> {
        self.group_by_id.get(&group_id).unwrap()
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

        // We are not allowed to reuse an ID, until it's no longer a source
        if self.sources.is_tracked_source(id) {
            self.returned_ids.insert(id);
        } else {
            self.free_ids.make_available(id);
        }

        // Perform some cleanup of earlier returned ids
        for returned in self.returned_ids.clone() {
            if !self.sources.is_tracked_source(returned) {
                self.returned_ids.remove(&returned);
                self.free_ids.make_available(returned);
            }
        }
    }

    fn remove_edges_to_set(edges: &mut EdgeCounts<T>, edge_data: EdgeData<T>, count: usize) {
        if let Some(cur_count) = edges.get_mut(&edge_data) {
            *cur_count -= count;
            if (*cur_count <= 0) {
                edges.remove(&edge_data);
            }
        }
    }

    fn remove_edges(
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

        let from_group = self.get_node_group_mut(from);
        InnerGroupManager::<T, NL, LL, G>::remove_edges_to_set(
            &mut from_group.out_edges,
            EdgeData::new(to, from_level, to_level, edge_type),
            count,
        );
        from_group.nodes.entry(from_source).and_modify(|cd| {
            cd.children.remove(&(edge_type, to_source));
        });

        let to_group = self.get_node_group_mut(to);
        InnerGroupManager::<T, NL, LL, G>::remove_edges_to_set(
            &mut to_group.in_edges,
            EdgeData::new(from, to_level, from_level, edge_type),
            count,
        );
        to_group.nodes.entry(to_source).and_modify(|cd| {
            cd.parents.remove(&(edge_type, to_source));
        });
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

        let from_group = self.get_node_group_mut(from);
        InnerGroupManager::<T, NL, LL, G>::add_edges_to_set(
            &mut from_group.out_edges,
            EdgeData::new(to, from_level, to_level, edge_type),
            count,
        );
        from_group
            .nodes
            .entry(from_source)
            .or_insert_with(|| ConnectionData::new())
            .children
            .insert((edge_type, to_source));

        let to_group = self.get_node_group_mut(to);
        InnerGroupManager::<T, NL, LL, G>::add_edges_to_set(
            &mut to_group.in_edges,
            EdgeData::new(from, to_level, from_level, edge_type),
            count,
        );
        to_group
            .nodes
            .entry(to_source)
            .or_insert_with(|| ConnectionData::new())
            .parents
            .insert((edge_type, from_source));
    }

    fn remove_node_from_group(&mut self, node: NodeID) -> NodeGroupID {
        let cur_group_id = self.get_node_group_id(node);

        // Check if the node is explicitly contained (instead of implicitly, which can happen for the 0 group)
        let cur_group = self.get_node_group(cur_group_id);
        let contained = cur_group.nodes.contains_key(&node);

        // Remove old edges
        let cur_group = self.get_node_group_mut(cur_group_id);
        if let Some(connections) = cur_group.nodes.get(&node) {
            let connections = connections.clone();
            for (edge_type, child_id) in connections.children {
                let child_group_id = self.get_node_group_id(child_id);
                if contained && cur_group_id != child_group_id {
                    self.remove_edges(cur_group_id, node, child_group_id, child_id, edge_type, 1);
                }
            }

            for (edge_type, parent_id) in connections.parents {
                let parent_group_id = self.get_node_group_id(parent_id);
                if contained && parent_group_id != cur_group_id {
                    self.remove_edges(parent_group_id, parent_id, cur_group_id, node, edge_type, 1);
                }
            }
        }

        // Remove from group
        let cur_group = self.get_node_group_mut(cur_group_id);
        cur_group.nodes.remove(&node);
        cur_group.layer_min.remove(&node);
        cur_group.layer_max.remove(&node);

        cur_group_id
    }

    fn remove_group_if_empty(&mut self, group_id: NodeGroupID) {
        let cur_group = self.get_node_group_mut(group_id);
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
        let new_group = self.get_node_group_mut(group_id);
        new_group.nodes.insert(node, ConnectionData::new());
        new_group.layer_min.push(node, Reverse(from_level));
        new_group.layer_max.push(node, from_level);

        // Add new connections
        for (edge_type, child_id) in self.graph.get_children(node) {
            let child_group_id = self.get_node_group_id(child_id);
            let child_level = self.graph.get_level(child_id);

            if group_id != child_group_id {
                self.add_edges(group_id, node, child_group_id, child_id, edge_type, 1);
            }

            // Ensure the child id is in there, which may not have been the case initially for the initial group 0
            if child_group_id == 0 {
                let child_group = self.get_node_group_mut(child_group_id);
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
impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>>
    InnerGroupManager<T, NL, LL, G>
{
    pub fn new(mut graph: G) -> InnerGroupManager<T, NL, LL, G> {
        let root_id = graph.get_root();
        let root_level = graph.get_level(root_id);
        InnerGroupManager {
            level_label: PhantomData,
            node_label: PhantomData,
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
            free_ids: FreeIdManager::new(1),
            returned_ids: HashSet::new(),
            sources: SourceTrackerManager::new(),
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
                let init_nodes = self.get_node_group_mut(from_id).nodes.keys();
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
            } else if let Some(_) = self.group_by_id.get(&from_id) {
                let from_group = &self.get_node_group(from_id);
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

        let new_id = self.free_ids.get_next();
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
            self.sources.add_source(new_id, source);
        }
        new_id
    }

    pub fn split_edges(&mut self, group_id: NodeGroupID, fully: bool) {
        // TODO: rethink this entire approach, one nodeID can end up in multiple splits atm
        let group_nodes = &self
            .get_node_group(group_id)
            .nodes
            .keys()
            .cloned()
            .collect_vec();
        let mut splits: HashMap<(EdgeType<T>, NodeGroupID), HashSet<NodeID>> = HashMap::new();
        for &node in group_nodes {
            let children = &self.graph.get_children(node);
            for (edge_type, to) in children {
                let to_group = self.get_group(*to);
                splits
                    .entry((*edge_type, to_group))
                    .or_insert_with(|| HashSet::new())
                    .insert(*to);
            }
        }

        for ((_, group_id), nodes) in splits {
            let already_group = self.get_nodes_of_group(group_id).eq(nodes.clone());
            if already_group {
                continue;
            }

            if fully {
                for node in nodes {
                    self.create_group(vec![TargetID(TargetIDType::NodeID, node)]);
                }
            } else {
                console::log!("create-group: {}", nodes.iter().join(", "));
                self.create_group(
                    nodes
                        .iter()
                        .map(|&node| TargetID(TargetIDType::NodeID, node))
                        .collect(),
                );
            }
        }
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>>
    GroupedGraphStructure<T, String, LL> for InnerGroupManager<T, NL, LL, G>
{
    type Tracker = SourceReader;
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

    fn get_source_reader(&mut self) -> SourceReader {
        self.sources.get_reader()
    }

    fn get_level_label(&self, level: LevelNo) -> LL {
        self.graph.get_level_label(level)
    }

    fn get_group_label(&self, node: NodeID) -> String {
        todo!()
    }
}
