use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet, LinkedList},
    fmt::Display,
    hash::Hash,
    iter::FromIterator,
    rc::Rc,
    vec::IntoIter,
};

use itertools::Itertools;
use oxidd::{Edge, Function, InnerNode, LevelNo, Manager};
use oxidd_core::{DiagramRules, Node, Tag};
use priority_queue::PriorityQueue;

use crate::{
    types::util::edge_type::EdgeType,
    util::{free_id_manager::FreeIdManager, logging::console},
    wasm_interface::{NodeGroupID, NodeID, TargetID, TargetIDType},
};

use super::{graph_structure::GraphStructure, grouped_graph_structure::GroupedGraphStructure};

pub struct GroupManager<T: Tag, G: GraphStructure<T>> {
    /// root: Rc<F>,
    /// node_by_id: HashMap<NodeID, F>,
    graph: G,
    /// Nodes are implicitly in group 0 by default, I.e either:
    /// - group_by_id[group_id_by_node[node]].nodes.contains(node)
    /// - or !group_id_by_node.contains(node) && !exists g. group_by_id[g].nodes.contains(node)
    group_id_by_node: HashMap<NodeID, NodeGroupID>,
    group_by_id: HashMap<NodeGroupID, NodeGroup<T>>,
    free_ids: FreeIdManager<usize>,
    /// The highest level found so far
    max_level: LevelNo,
}

type EdgeSet<T: Tag> = HashMap<NodeGroupID, HashMap<EdgeType<T>, i32>>;
pub struct NodeGroup<T: Tag> {
    nodes: HashSet<NodeID>,
    out_edges: EdgeSet<T>,
    in_edges: EdgeSet<T>,
    layer_min: PriorityQueue<NodeID, Reverse<LevelNo>>,
    layer_max: PriorityQueue<NodeID, LevelNo>,
}
impl<T: Tag> Display for NodeGroup<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let min = self
            .layer_min
            .peek()
            .map_or(0, |(node, Reverse(layer))| *layer);
        let max = self.layer_max.peek().map_or(0, |(node, layer)| *layer);
        write!(
            f,
            "group(nodes: [{}], levels: ({}, {})",
            self.nodes.iter().join(", "),
            min,
            max
        )
    }
}

// Helper methods
impl<T: Tag, G: GraphStructure<T>> GroupManager<T, G> {
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
        self.free_ids.make_available(id);
    }

    fn remove_edges_to_set(
        edges: &mut EdgeSet<T>,
        edge_type: EdgeType<T>,
        target: NodeGroupID,
        count: i32,
    ) {
        if let Some(target_edges) = edges.get_mut(&target) {
            if let Some(cur_count) = target_edges.get_mut(&edge_type) {
                *cur_count -= count;
                if (*cur_count <= 0) {
                    target_edges.remove(&edge_type);
                }
            }
            if target_edges.is_empty() {
                edges.remove(&target);
            }
        }
    }

    fn remove_edges(
        &mut self,
        from: NodeGroupID,
        to: NodeGroupID,
        edge_type: EdgeType<T>,
        count: i32,
    ) {
        let from_group = self.get_node_group_mut(from);
        GroupManager::<T, G>::remove_edges_to_set(&mut from_group.out_edges, edge_type, to, count);

        let to_group = self.get_node_group_mut(to);
        GroupManager::<T, G>::remove_edges_to_set(&mut to_group.in_edges, edge_type, from, count);
    }

    fn add_edges_to_set(
        edges: &mut EdgeSet<T>,
        edge_type: EdgeType<T>,
        target: NodeGroupID,
        count: i32,
    ) {
        let target_edges = edges.entry(target).or_insert_with(|| HashMap::new());
        let cur_count = target_edges.entry(edge_type).or_insert(0);
        *cur_count += count;
    }

    fn add_edges(
        &mut self,
        from: NodeGroupID,
        to: NodeGroupID,
        edge_type: EdgeType<T>,
        count: i32,
    ) {
        let from_group = self.get_node_group_mut(from);
        GroupManager::<T, G>::add_edges_to_set(&mut from_group.out_edges, edge_type, to, count);

        let to_group = self.get_node_group_mut(to);
        GroupManager::<T, G>::add_edges_to_set(&mut to_group.in_edges, edge_type, from, count);
    }
}

// Main methods
impl<T: Tag, G: GraphStructure<T>> GroupManager<T, G> {
    pub fn new(mut graph: G) -> GroupManager<T, G> {
        let root_id = graph.get_root();
        let root_level = graph.get_level(root_id);
        GroupManager {
            graph,
            group_id_by_node: HashMap::new(),
            group_by_id: HashMap::from([(
                0,
                NodeGroup {
                    nodes: HashSet::from([root_id]),
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
            max_level: 0,
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
                let cur_group_id = self.get_node_group_id(from_id);
                let from_level = self.graph.get_level(from_id);

                // Remove from old
                let cur_group = self.get_node_group_mut(cur_group_id);
                let contained = cur_group.nodes.remove(&from_id);
                cur_group.layer_min.remove(&from_id);
                cur_group.layer_max.remove(&from_id);

                // Add to new
                self.group_id_by_node.insert(from_id, to);
                let new_group = self.get_node_group_mut(to);
                new_group.nodes.insert(from_id);
                new_group.layer_min.push(from_id, Reverse(from_level));
                new_group.layer_max.push(from_id, from_level);

                // Update edges
                for (edge_type, child_id) in self.graph.get_children(from_id) {
                    let child_group_id = self.get_node_group_id(child_id);
                    if contained && cur_group_id != child_group_id {
                        self.remove_edges(cur_group_id, child_group_id, edge_type, 1);
                    }
                    if to != child_group_id {
                        self.add_edges(to, child_group_id, edge_type, 1);
                    }

                    // Ensure the child id is in there
                    if child_group_id == 0 {
                        let child_level = self.graph.get_level(child_id);
                        let child_group = self.get_node_group_mut(child_group_id);
                        if child_group.nodes.insert(child_id) {
                            child_group.layer_min.push(child_id, Reverse(child_level));
                            child_group.layer_max.push(child_id, child_level);
                        }
                    }
                }

                for (edge_type, edge_from) in self.graph.get_known_parents(from_id) {
                    let from_group = self.get_node_group_id(edge_from);

                    if contained && from_group != cur_group_id {
                        self.remove_edges(from_group, cur_group_id, edge_type, 1);
                    }
                    if from_group != to {
                        self.add_edges(from_group, to, edge_type, 1);
                    }
                }

                // Check if old group became empty
                let cur_group = self.get_node_group_mut(cur_group_id);
                let from_empty = cur_group.nodes.is_empty();
                if from_empty {
                    self.remove_group(cur_group_id);
                    console::log!("removed");
                }
            } else if from_id == to {
                continue;
            } else if from_id == 0 {
                let init_nodes = self.get_node_group_mut(from_id).nodes.clone();
                let mut found: HashSet<NodeID> = init_nodes.clone().into_iter().collect();
                let mut queue: LinkedList<NodeID> = init_nodes.into_iter().collect();

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
                let out_edges = from_group.out_edges.clone();
                let in_edges = from_group.in_edges.clone();
                let from_nodes = from_group.nodes.clone();
                let layer_min: Vec<(NodeID, Reverse<LevelNo>)> =
                    from_group.layer_min.iter().map(|(&n, &i)| (n, i)).collect();
                let layer_max: Vec<(NodeID, LevelNo)> =
                    from_group.layer_max.iter().map(|(&n, &i)| (n, i)).collect();

                for out_edge_target in out_edges.keys() {
                    let out_types = &out_edges[out_edge_target];
                    for out_type in out_types.keys() {
                        let count = out_types[out_type];

                        self.remove_edges(from_id, *out_edge_target, *out_type, count);
                        self.add_edges(to, *out_edge_target, *out_type, count);
                    }
                }
                for in_edge_target in in_edges.keys() {
                    let in_types = &in_edges[in_edge_target];
                    for in_type in in_types.keys() {
                        let count = in_types[in_type];

                        self.remove_edges(*in_edge_target, from_id, *in_type, count);
                        self.add_edges(*in_edge_target, to, *in_type, count);
                    }
                }

                for from_node in &from_nodes {
                    self.group_id_by_node.insert(*from_node, to);
                }
                let to_group = self.get_node_group_mut(to);
                to_group.nodes.extend(&from_nodes);
                to_group.layer_min.extend(layer_min);
                to_group.layer_max.extend(layer_max);

                self.remove_group(from_id);
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
        let new_id = self.free_ids.get_next();
        self.group_by_id.insert(
            new_id,
            NodeGroup {
                nodes: HashSet::new(),
                in_edges: HashMap::new(),
                out_edges: HashMap::new(),
                layer_min: PriorityQueue::new(),
                layer_max: PriorityQueue::new(),
            },
        );
        self.set_group(from, new_id);
        new_id
    }

    pub fn split_edges(&mut self, group_id: NodeGroupID, fully: bool) {
        // TODO: rethink this enture approach, one nodeID can end up in multiple splits atm
        let group_nodes = &self.get_node_group(group_id).nodes.clone();
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
                console::log!("{}", nodes.iter().join(", "));
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

impl<T: Tag, G: GraphStructure<T>> GroupedGraphStructure<T> for GroupManager<T, G> {
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

    fn get_parents(&self, group: NodeGroupID) -> IntoIter<(EdgeType<T>, NodeGroupID, i32)> {
        if let Some(group) = self.group_by_id.get(&group) {
            return group
                .in_edges
                .iter()
                .flat_map(|(to, edges)| {
                    edges
                        .iter()
                        .map(move |(edge_type, count)| (*edge_type, *to, *count))
                })
                .collect::<Vec<(EdgeType<T>, NodeGroupID, i32)>>()
                .into_iter();
        }
        Vec::new().into_iter()
    }

    fn get_children(&self, group: NodeGroupID) -> IntoIter<(EdgeType<T>, NodeGroupID, i32)> {
        if let Some(group) = self.group_by_id.get(&group) {
            return group
                .out_edges
                .iter()
                .flat_map(|(to, edges)| {
                    edges
                        .iter()
                        .map(move |(edge_type, count)| (*edge_type, *to, *count))
                })
                .collect::<Vec<(EdgeType<T>, NodeGroupID, i32)>>()
                .into_iter();
        }
        Vec::new().into_iter()
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
                console::log!("{} {}", group_id, group);
                (min, max)
            },
        )
    }

    fn get_nodes_of_group(&self, group: NodeGroupID) -> IntoIter<NodeID> {
        if let Some(group) = self.group_by_id.get(&group) {
            return group
                .nodes
                .iter()
                .map(|&id| id)
                .collect::<Vec<NodeID>>()
                .into_iter();
        }
        Vec::new().into_iter()
    }

    fn get_group(&self, node: NodeID) -> NodeGroupID {
        self.get_node_group_id(node)
    }
}
