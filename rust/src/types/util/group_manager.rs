use std::{
    collections::{HashMap, HashSet, LinkedList},
    hash::Hash,
    iter::FromIterator,
    rc::Rc,
};

use oxidd::{Edge, Function, InnerNode, Manager};
use oxidd_core::{DiagramRules, Node, Tag};

use crate::{
    types::util::edge_type::EdgeType,
    util::{free_id_manager::FreeIdManager, logging::console},
    wasm_interface::{NodeGroupID, NodeID, TargetID, TargetIDType},
};

pub struct GroupManager<T: Tag, F: Function>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = T>,
{
    root: Rc<F>,
    node_by_id: HashMap<NodeID, F>,
    // Nodes are implicitly in group 0 by default, I.e either:
    // - group_by_id[group_id_by_node[node]].nodes.contains(node)
    // - or !group_id_by_node.contains(node) && !exists g. group_by_id[g].nodes.contains(node)
    group_id_by_node: HashMap<NodeID, NodeGroupID>,
    group_by_id: HashMap<NodeGroupID, NodeGroup<T>>,
    free_ids: FreeIdManager<usize>,
    // The known parents of a node (based on what node have been moved out of the default group)
    node_parents: HashMap<NodeID, HashSet<NodeParentEdge<T>>>,
}

type EdgeSet<T: Tag> = HashMap<NodeGroupID, HashMap<EdgeType<T>, i32>>;
pub struct NodeGroup<T: Tag> {
    pub nodes: HashSet<NodeID>,
    pub out_edges: EdgeSet<T>,
    pub in_edges: EdgeSet<T>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct NodeParentEdge<T: Tag> {
    pub parent: NodeID,
    pub edge_type: EdgeType<T>,
}
impl<T: Tag> Hash for NodeParentEdge<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.parent.hash(state);
        self.edge_type.hash(state);
    }
}

// Helper methods
impl<ET: Tag, T, E: Edge<Tag = ET>, N: InnerNode<E>, R: DiagramRules<E, N, T>, F: Function>
    GroupManager<ET, F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn get_id_by_node(&mut self, node: &F) -> NodeID {
        node.with_manager_shared(|manager, edge| {
            let id = edge.node_id();
            self.node_by_id.insert(id, node.clone());
            return id;
        })
    }
    fn get_node_by_id(&self, id: NodeID) -> Option<&F> {
        return self.node_by_id.get(&id);
    }

    fn add_parent(&mut self, node: NodeID, parent: NodeID, edge_type: EdgeType<ET>) {
        let parents = self
            .node_parents
            .entry(node)
            .or_insert_with(|| HashSet::new());
        let edge: NodeParentEdge<ET> = NodeParentEdge { parent, edge_type };
        parents.insert(edge);
    }

    fn get_parents(&self, node: NodeID) -> Option<&HashSet<NodeParentEdge<ET>>> {
        return self.node_parents.get(&node);
    }

    fn get_children(&self, node: &F) -> Vec<(ET, F)> {
        node.with_manager_shared(|manager, edge| {
            let tag = edge.tag();
            let internal_node = manager.get_node(edge);
            if let Node::Inner(node) = internal_node {
                let cofactors = R::cofactors(tag, node);
                return Vec::from_iter(cofactors.map(|f| (f.tag(), F::from_edge_ref(manager, &f))));
            }
            return Vec::new();
        })
    }

    fn get_node_group_mut(&mut self, group_id: NodeGroupID) -> &mut NodeGroup<ET> {
        self.group_by_id.get_mut(&group_id).unwrap()
    }

    pub fn get_node_group(&self, group_id: NodeGroupID) -> &NodeGroup<ET> {
        self.group_by_id.get(&group_id).unwrap()
    }

    pub fn get_node_group_id(&self, node: NodeID) -> NodeGroupID {
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
        edges: &mut EdgeSet<ET>,
        edge_type: EdgeType<ET>,
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
        edge_type: EdgeType<ET>,
        count: i32,
    ) {
        let from_group = self.get_node_group_mut(from);
        GroupManager::<ET, F>::remove_edges_to_set(&mut from_group.out_edges, edge_type, to, count);

        let to_group = self.get_node_group_mut(to);
        GroupManager::<ET, F>::remove_edges_to_set(&mut to_group.in_edges, edge_type, from, count);
    }

    fn add_edges_to_set(
        edges: &mut EdgeSet<ET>,
        edge_type: EdgeType<ET>,
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
        edge_type: EdgeType<ET>,
        count: i32,
    ) {
        let from_group = self.get_node_group_mut(from);
        GroupManager::<ET, F>::add_edges_to_set(&mut from_group.out_edges, edge_type, to, count);

        let to_group = self.get_node_group_mut(to);
        GroupManager::<ET, F>::add_edges_to_set(&mut to_group.in_edges, edge_type, from, count);
    }
}

// Main methods
impl<ET: Tag, T, E: Edge<Tag = ET>, N: InnerNode<E>, R: DiagramRules<E, N, T>, F: Function>
    GroupManager<ET, F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    pub fn new(root: Rc<F>) -> GroupManager<ET, F> {
        let root_id = root.with_manager_shared(|manager, edge| edge.node_id());
        GroupManager {
            root: root.clone(),
            node_by_id: HashMap::from([(root_id, (*root).clone())]),
            group_id_by_node: HashMap::new(),
            group_by_id: HashMap::from([(
                0,
                NodeGroup {
                    nodes: HashSet::from([root_id]),
                    out_edges: HashMap::new(),
                    in_edges: HashMap::new(),
                },
            )]),
            free_ids: FreeIdManager::new(1),
            node_parents: HashMap::new(),
        }
    }

    pub fn get_groups(&self) -> &HashMap<NodeGroupID, NodeGroup<ET>> {
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
                let cur_group = self.get_node_group_mut(cur_group_id);
                let contained = cur_group.nodes.remove(&from_id);

                self.group_id_by_node.insert(from_id, to);
                self.get_node_group_mut(to).nodes.insert(from_id);

                if let Some(node) = self.get_node_by_id(from_id) {
                    let mut index = 0;
                    for (edge_tag, child) in self.get_children(node) {
                        let edge_type = EdgeType::new(edge_tag, index);
                        index += 1;

                        let child_id = self.get_id_by_node(&child);
                        self.add_parent(child_id, from_id, edge_type);

                        let child_group_id = self.get_node_group_id(child_id);
                        if contained && cur_group_id != child_group_id {
                            self.remove_edges(cur_group_id, child_group_id, edge_type, 1);
                        }
                        if to != child_group_id {
                            self.add_edges(to, child_group_id, edge_type, 1);
                        }

                        // Ensure the child id is in there
                        if child_group_id == 0 {
                            let child_group = self.get_node_group_mut(child_group_id);
                            child_group.nodes.insert(child_id);
                        }
                    }
                }
                if let Some(parents) = self.get_parents(from_id) {
                    let p = (*parents).clone();
                    for parent_edge in p {
                        let edge_type = parent_edge.edge_type;
                        let edge_from = parent_edge.parent;

                        let from_group = self.get_node_group_id(edge_from);

                        if contained && from_group != cur_group_id {
                            self.remove_edges(from_group, cur_group_id, edge_type, 1);
                        }
                        if from_group != to {
                            self.add_edges(from_group, to, edge_type, 1);
                        }
                    }
                }

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
                    let Some(node) = self.get_node_by_id(node_id) else {
                        continue;
                    };

                    for (_, child) in self.get_children(node) {
                        let child_id = self.get_id_by_node(&child);
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
                let from_group = self.get_node_group_mut(from_id);
                let out_edges = from_group.out_edges.clone();
                let in_edges = from_group.in_edges.clone();
                let from_nodes = from_group.nodes.clone();

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
            },
        );
        self.set_group(from, new_id);
        new_id
    }

    pub fn get_nodes(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Vec<crate::wasm_interface::NodeGroupID> {
        todo!()
    }
}
