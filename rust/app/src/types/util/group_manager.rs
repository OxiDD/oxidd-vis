use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{HashMap, HashSet, LinkedList, VecDeque},
    fmt::Display,
    hash::Hash,
    io::{Cursor, Result, Write},
    iter::FromIterator,
    marker::PhantomData,
    rc::Rc,
    vec::IntoIter,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
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
    storage::state_storage::StateStorage,
};

pub struct GroupManager<G: GraphStructure> {
    graph: G,
    graph_events: GraphEventsReader,

    /// Nodes are implicitly in group 0 by default, I.e either:
    /// - group_by_id[group_id_by_node[node]].nodes.contains(node)
    /// - or !group_id_by_node.contains(node) && !exists g. group_by_id[g].nodes.contains(node)
    group_id_by_node: HashMap<NodeID, NodeGroupID>,
    group_by_id: HashMap<NodeGroupID, NodeGroup<G::T>>,
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
impl<G: GraphStructure> GroupManager<G> {
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
                        if self.get_node_group_id(node) == None {
                            // // Don't add it to the group in which it's already implicitly present
                            // if group == 0 {
                            //     return;
                            // }

                            self.add_node_to_group(node, group);
                        }
                    } else {
                        let group_id =
                            self.create_group(vec![TargetID(TargetIDType::NodeID, node)]);

                        // Add a source for the new group
                        if let Some(source) = source {
                            if let Some(&source_group_id) = removed_from
                                .get(&source)
                                .or_else(|| self.group_id_by_node.get(&source))
                            {
                                self.group_ids.add_sources(group_id, vec![source_group_id]);
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

    fn get_node_group_id(&self, node: NodeID) -> Option<NodeGroupID> {
        if let Some(group_id) = self.group_id_by_node.get(&node) {
            Some(*group_id)
        } else {
            None
        }
    }

    fn remove_group(&mut self, id: NodeGroupID) {
        self.group_by_id.remove(&id);
        self.group_ids.make_available(id);
    }

    fn remove_edges_to_set(edges: &mut EdgeCounts<G::T>, edge_data: EdgeData<G::T>, count: usize) {
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
        edge_type: EdgeType<G::T>,
        count: usize,
    ) {
        if let Some(from_group) = self.group_by_id.get_mut(&from) {
            GroupManager::<G>::remove_edges_to_set(
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
            GroupManager::<G>::remove_edges_to_set(
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

    fn add_edges_to_set(edges: &mut EdgeCounts<G::T>, edge_data: EdgeData<G::T>, count: usize) {
        let cur_count = edges.entry(edge_data).or_insert(0);
        *cur_count += count;
    }

    fn add_edges(
        &mut self,
        from: NodeGroupID,
        from_source: NodeID,
        to: NodeGroupID,
        to_source: NodeID,
        edge_type: EdgeType<G::T>,
        count: usize,
    ) {
        let from_level = self.graph.get_level(from_source);
        let to_level = self.graph.get_level(to_source);

        let has_from = self
            .group_by_id
            .get_mut(&from)
            .and_then(|from_group| from_group.nodes.get_mut(&from_source))
            .is_some();
        let has_to = self
            .group_by_id
            .get_mut(&to)
            .and_then(|to_group| to_group.nodes.get_mut(&to_source))
            .is_some();

        if !has_from {
            self.ensure_node_in_group(from_source);
            return;
        }
        if !has_to {
            self.ensure_node_in_group(to_source);
            return;
        }

        // self.ensure_node_in_group(from_source);
        if let Some(from_group) = self.group_by_id.get_mut(&from) {
            GroupManager::<G>::add_edges_to_set(
                &mut from_group.out_edges,
                EdgeData::new(to, from_level, to_level, edge_type),
                count,
            );
            from_group
                .nodes
                .get_mut(&from_source)
                .unwrap()
                .children
                .insert((from_level, to_level, edge_type, to_source));
        }

        // self.ensure_node_in_group(to_source);
        if let Some(to_group) = self.group_by_id.get_mut(&to) {
            GroupManager::<G>::add_edges_to_set(
                &mut to_group.in_edges,
                EdgeData::new(from, to_level, from_level, edge_type),
                count,
            );
            to_group.nodes.get_mut(&to_source).unwrap().parents.insert((
                from_level,
                to_level,
                edge_type,
                from_source,
            ));
        }
    }

    fn remove_node_from_group(&mut self, node: NodeID) -> NodeGroupID {
        let cur_group_id = self.get_node_group_id(node);
        let Some(cur_group_id) = cur_group_id else {
            return 0;
        };

        // Check if the node is explicitly contained (instead of implicitly, which can happen for the 0 group)
        let Some(cur_group) = self.group_by_id.get(&cur_group_id) else {
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
                let Some(child_group_id) = self.get_node_group_id(child_id) else {
                    continue;
                };
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
                let Some(parent_group_id) = self.get_node_group_id(parent_id) else {
                    continue;
                };
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

        self.group_id_by_node.remove(&node);

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
            let child_group_id = self.get_node_group_id(child_id).unwrap_or(0);

            if group_id != child_group_id {
                self.add_edges(group_id, node, child_group_id, child_id, edge_type, 1);
            }
        }

        for (edge_type, parent_id) in self.graph.get_known_parents(node) {
            let parent_group_id = self.get_node_group_id(parent_id).unwrap_or(0);

            if parent_group_id != group_id {
                self.add_edges(parent_group_id, parent_id, group_id, node, edge_type, 1);
            }
        }
    }

    // Makes sure that a node is in a group (group 0 if it was not yet found before)
    fn ensure_node_in_group(&mut self, node_id: NodeID) {
        let group_id = self.get_node_group_id(node_id).unwrap_or(0);
        let Some(node_group) = self.group_by_id.get_mut(&group_id) else {
            return;
        };
        if !node_group.nodes.contains_key(&node_id) {
            self.add_node_to_group(node_id, group_id);
        }
    }

    fn refill_if_hidden_emptying(&mut self) {
        let Some(hidden_group) = self.group_by_id.get_mut(&0) else {
            return;
        };

        // If the group is almost empty, also make sure its connections are known
        if hidden_group.nodes.len() == 1 {
            let &node_id = hidden_group.nodes.keys().next().unwrap();
            for (_, neighbor) in self
                .graph
                .get_children(node_id)
                .into_iter()
                .chain(self.graph.get_known_parents(node_id).into_iter())
            {
                self.ensure_node_in_group(neighbor);
            }
        }
    }

    fn format_groups(&self) -> String {
        format!(
            "({})",
            self.group_by_id
                .iter()
                .map(|(&group_id, group)| format!(
                    "{}: {{\n    range: ({}, {}), \n    nodes:({}),\n    in:[{}],\n    out:[{}]\n}}",
                    group_id,
                    group.layer_min.peek().map(|v| format!("{}", v.1.0)).unwrap_or_default(),
                    group.layer_max.peek().map(|v| format!("{}", v.1)).unwrap_or_default(),
                    group
                        .nodes
                        .iter()
                        .map(|(node, edge)| format!(
                            "{}: {{in: [{}], out: [{}]}}",
                            node,
                            edge.parents
                                .iter()
                                .map(|(_, _, _, to)| format!("{}", to))
                                .join(", "),
                            edge.children
                                .iter()
                                .map(|(_, _, _, to)| format!("{}", to))
                                .join(", ")
                        ))
                        .join(", "),
                    group
                        .in_edges
                        .iter()
                        .map(|(ed, count)| format!("({}, {})", ed.to_string(group_id), count))
                        .join(", "),
                    group
                        .out_edges
                        .iter()
                        .map(|(ed, count)| format!("({}, {})", ed.to_string(group_id), count))
                        .join(", ")
                ))
                .join(", \n")
        )
    }
}

// Main methods
impl<G: GraphStructure> GroupManager<G> {
    pub fn new(mut graph: G) -> GroupManager<G> {
        let mut gm = GroupManager {
            graph_events: graph.create_event_reader(),
            group_id_by_node: HashMap::new(),
            group_by_id: HashMap::new(),
            graph,
            group_ids: NodeTrackerManager::new(1),
        };
        gm.reset();
        gm
    }

    pub fn reset(&mut self) {
        let root_ids = self.graph.get_roots();
        for &group_id in self.group_by_id.keys().collect_vec() {
            self.group_ids.make_available(group_id);
        }
        self.group_id_by_node.clear();
        self.group_by_id.clear();
        let layer_min = root_ids
            .iter()
            .map(|&root_id| (root_id, Reverse(self.graph.get_level(root_id))))
            .collect();
        let layer_max = root_ids
            .iter()
            .map(|&root_id| (root_id, self.graph.get_level(root_id)))
            .collect();
        self.group_by_id.insert(
            0,
            NodeGroup {
                nodes: root_ids
                    .iter()
                    .map(|&root_id| (root_id, ConnectionData::new()))
                    .collect(),
                out_edges: HashMap::new(),
                in_edges: HashMap::new(),
                layer_min,
                layer_max,
            },
        );
        for root_id in root_ids {
            self.group_id_by_node.insert(root_id, 0);
        }
        self.graph.consume_events(&self.graph_events);
    }

    pub fn get_groups(&self) -> &HashMap<NodeGroupID, NodeGroup<G::T>> {
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
                if old_group_id == 0 {
                    self.refill_if_hidden_emptying();
                }
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
                // TODO: make a more efficient merge that doesn't need to handle the group node by node
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
        let sources = from
            .iter()
            .map(|&TargetID(target_type, id)| match target_type {
                TargetIDType::NodeID => self.get_node_group_id(id).unwrap_or(0),
                _ => id,
            })
            .filter(|id| self.group_by_id.contains_key(id))
            .collect_vec();

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

        if sources.len() > 0 {
            self.group_ids.add_sources(new_id, sources);
        }
        self.set_group(from, new_id);
        new_id
    }

    pub fn split_edges(&mut self, node_ids: &[NodeID], max_layers: usize, mut max_nodes: usize) {
        // TODO: come up with a better splitting approach that considers nodes together
        let mut split = HashSet::new();

        // We use expand dir to go both up and down from the root, but only up or down from there forward
        #[derive(PartialEq, Eq)]
        enum ExpandDir {
            Up,
            Down,
            Both,
        };

        let mut queue = node_ids
            .iter()
            .map(|&node| (node, 0, ExpandDir::Both))
            .collect::<VecDeque<_>>();

        while let Some((node_id, depth, dir)) = queue.pop_front() {
            if max_nodes == 0 {
                return;
            }
            max_nodes -= 1;

            let mut neighbors = Vec::new();
            if dir != ExpandDir::Up {
                neighbors.extend(
                    self.graph
                        .get_children(node_id)
                        .into_iter()
                        .map(|(_, node)| (node, ExpandDir::Down)),
                )
            };
            if dir != ExpandDir::Down {
                neighbors.extend(
                    self.graph
                        .get_known_parents(node_id)
                        .into_iter()
                        .map(|(_, node)| (node, ExpandDir::Up)),
                )
            };

            let next_depth = depth + 1;
            for (child, dir) in neighbors {
                if split.contains(&child) {
                    continue;
                }

                split.insert(child);
                let group_id = self.get_group(child);
                if self.get_nodes_of_group(group_id).len() == 1 && group_id != 0 {
                    continue;
                }

                self.create_group(vec![TargetID(TargetIDType::NodeID, child)]);

                if next_depth < max_layers {
                    queue.push_back((child, next_depth, dir));
                }
            }
        }

        // for &node_id in node_ids {
        //     let neighbors = self
        //         .graph
        //         .get_children(node_id)
        //         .into_iter()
        //         .chain(self.graph.get_known_parents(node_id).into_iter());
        //     for (_, child) in neighbors {
        //         if split.contains(&child) {
        //             continue;
        //         }

        //         split.insert(child);
        //         let group_id = self.get_group(child);
        //         if self.get_nodes_of_group(group_id).len() == 1 && group_id != 0 {
        //             continue;
        //         }

        //         self.create_group(vec![TargetID(TargetIDType::NodeID, child)]);
        //     }
        // }
    }
}

impl<G: GraphStructure> GroupedGraphStructure for GroupManager<G> {
    type T = G::T;
    type GL = Vec<G::NL>;
    type LL = G::LL;
    type Tracker = NodeTrackerM;
    fn get_roots(&self) -> Vec<NodeGroupID> {
        let root_nodes = self.graph.get_roots();
        root_nodes
            .into_iter()
            .map(|root_node| self.get_node_group_id(root_node).unwrap_or(0))
            .collect()
    }

    fn get_all_groups(&self) -> Vec<NodeGroupID> {
        self.group_by_id
            .keys()
            .into_iter()
            .map(|&id| id)
            .sorted()
            .collect()
    }

    fn get_hidden(&self) -> Vec<NodeGroupID> {
        if self.group_by_id.contains_key(&0) {
            vec![0]
        } else {
            vec![]
        }
    }

    fn get_parents(&self, group: NodeGroupID) -> Vec<EdgeCountData<G::T>> {
        self.group_by_id.get(&group).map_or_else(
            || Vec::default(),
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
                    .sorted()
                    .collect::<Vec<_>>()
            },
        )
    }

    fn get_children(&self, group: NodeGroupID) -> Vec<EdgeCountData<G::T>> {
        self.group_by_id.get(&group).map_or_else(
            || Vec::default(),
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
                    .sorted()
                    .collect::<Vec<_>>()
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

    fn get_nodes_of_group(&self, group: NodeGroupID) -> Vec<NodeID> {
        self.group_by_id
            .get(&group)
            .map_or_else(
                || Vec::default().into_iter(),
                |group| group.nodes.keys().cloned().collect_vec().into_iter(),
            )
            .sorted()
            .collect_vec()
    }

    fn get_group(&self, node: NodeID) -> NodeGroupID {
        self.get_node_group_id(node).unwrap_or(0)
    }

    fn create_node_tracker(&mut self) -> Self::Tracker {
        self.group_ids.create_reader()
    }

    fn get_level_label(&self, level: LevelNo) -> G::LL {
        self.graph.get_level_label(level)
    }

    fn get_group_label(&self, group_id: NodeID) -> Vec<G::NL> {
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

impl<G: GraphStructure + StateStorage> StateStorage for GroupManager<G> {
    fn write(&self, stream: &mut Cursor<&mut Vec<u8>>) -> Result<()> {
        self.graph.write(stream)?;
        let group_count = self.group_by_id.len();
        stream.write_u32::<LittleEndian>(group_count as u32)?;
        for (group_id, group) in &self.group_by_id {
            let nodes = group.nodes.keys();
            let node_count = nodes.len();
            stream.write_u32::<LittleEndian>(*group_id as u32)?;
            stream.write_u32::<LittleEndian>(node_count as u32)?;
            for node in nodes {
                stream.write_u32::<LittleEndian>(*node as u32)?;
            }
        }
        Ok(())
    }

    fn read(&mut self, stream: &mut Cursor<&Vec<u8>>) -> Result<()> {
        self.graph.consume_events(&self.graph_events);
        self.reset();

        self.graph.read(stream)?;
        // No events should be created, but just in case, throw away events
        let events = self.graph.consume_events(&self.graph_events);
        if events.len() > 0 {
            console::log!(
                "Deserialization should not have caused any events, Event count: {}",
                events.len()
            );
            console::log!("Created events: {}", events.iter().join(",\n"));
        }

        let mut all_found_nodes = HashSet::new();
        let group_count = stream.read_u32::<LittleEndian>()?;
        let mut to_add = Vec::new();
        for _ in 0..group_count {
            let group_id = stream.read_u32::<LittleEndian>()? as usize;
            let node_count = stream.read_u32::<LittleEndian>()?;

            let mut targets = Vec::<TargetID>::new();
            for _ in 0..node_count {
                let node = stream.read_u32::<LittleEndian>()? as usize;
                targets.push(TargetID::new(TargetIDType::NodeID, node));
                all_found_nodes.insert(node);
            }

            if !self.group_by_id.contains_key(&group_id) {
                self.group_ids.add_group_id(group_id, true);
                self.group_by_id.insert(
                    group_id,
                    NodeGroup {
                        nodes: HashMap::new(),
                        in_edges: HashMap::new(),
                        out_edges: HashMap::new(),
                        layer_min: PriorityQueue::new(),
                        layer_max: PriorityQueue::new(),
                    },
                );
            }

            to_add.push((targets, group_id));
            // self.set_group(targets, group_id);
        }

        self.explore_from_root(all_found_nodes);

        for (targets, group_id) in to_add {
            self.set_group(targets, group_id);
        }
        Ok(())
    }
}

impl<G: GraphStructure> GroupManager<G> {
    // This function can be used to make sure that all nodes in the given set are found through "children" accesses. The graph structure assumes that all reference node IDs are found this way, so it's important that if the ID is obtained another way it's found using this function before further use
    fn explore_from_root(&mut self, nodes: HashSet<NodeID>) {
        let mut found = HashSet::new();
        let mut frontier = Vec::new();

        for node in self.graph.get_roots() {
            if !nodes.contains(&node) {
                continue;
            }
            found.insert(node);
            frontier.push(node);
        }

        while !frontier.is_empty() {
            let node = frontier.pop().unwrap();
            for (_, child) in self.graph.get_children(node) {
                if !nodes.contains(&child) {
                    continue;
                }
                if found.contains(&child) {
                    continue;
                }
                found.insert(child);
                frontier.push(child);
            }
        }
    }
}
