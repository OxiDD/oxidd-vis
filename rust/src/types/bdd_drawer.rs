use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Deref;
use std::rc::Rc;

use crate::traits::Diagram;
use crate::traits::DiagramDrawer;
use crate::util::free_id_manager::FreeIdManager;
use crate::wasm_interface::NodeGroupID;
use crate::wasm_interface::NodeID;
use crate::wasm_interface::TargetID;
use crate::wasm_interface::TargetIDType;
use oxidd::bdd;
use oxidd::util::Borrowed;
use oxidd::Edge;
use oxidd::InnerNode;
use oxidd::Manager;
use oxidd_core::HasApplyCache;
use oxidd_core::HasLevel;
use oxidd_core::Node;
use oxidd_core::{util::DropWith, Tag};
use oxidd_rules_bdd::simple::BDDOp;
use oxidd_rules_bdd::simple::BDDTerminal;

pub struct BDDDiagram<'a, M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    manager: M,
    root: &'a Node<'a, M>,
}

impl<'a, M> Diagram for BDDDiagram<'a, M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    fn create_drawer(&self) -> Box<crate::traits::DiagramDrawer> {
        todo!()
    }
}

type EdgeSet<M: Manager> = HashMap<NodeGroupID, HashMap<EdgeType<M::EdgeTag>, i32>>;
struct NodeGroup<M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    nodes: HashSet<NodeID>,
    out_edges: EdgeSet<M>,
    in_edges: EdgeSet<M>,
}

#[derive(Eq, PartialEq, Copy, Clone)]
struct EdgeType<T: Tag> {
    tag: T,
    index: i32,
}
impl<T: Tag> EdgeType<T> {
    fn new(tag: T, index: i32) -> EdgeType<T> {
        EdgeType { tag, index }
    }
}

impl<T: Tag> Hash for EdgeType<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

// #[derive(Eq, Hash, PartialEq)]
struct NodeParentEdge<M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    parent: NodeID,
    edge_type: EdgeType<M::EdgeTag>,
}
impl<M> Clone for NodeParentEdge<M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone(),
            edge_type: self.edge_type.clone(),
        }
    }
}

impl<'a, M> Hash for NodeParentEdge<M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.parent.hash(state);
        self.edge_type.hash(state);
    }
}
impl<M> Eq for NodeParentEdge<M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
}
impl<M> PartialEq for NodeParentEdge<M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.edge_type == other.edge_type
    }
}

pub struct BDDDiagramDrawer<'a, M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    manager: &'a M,
    root: &'a Node<'a, M>,
    node_by_id: HashMap<NodeID, Node<'a, M>>,
    // Nodes are implicitly in group 0 by default, I.e either:
    // - group_by_id[group_id_by_node[node]].nodes.contains(node)
    // - or !group_id_by_node.contains(node) && !exists g. group_by_id[g].nodes.contains(node)
    group_id_by_node: HashMap<NodeID, NodeGroupID>,
    group_by_id: HashMap<NodeGroupID, NodeGroup<M>>,
    free_ids: FreeIdManager<usize>,
    // The known parents of a node (based on what node have been moved out of the default group)
    node_parents: HashMap<NodeID, HashSet<NodeParentEdge<M>>>,
}

impl<'a, M> BDDDiagramDrawer<'a, M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    pub fn new(diagram: &'a BDDDiagram<'a, M>) -> BDDDiagramDrawer<'a, M> {
        let mut groups = HashMap::new();
        groups.insert(
            0,
            NodeGroup {
                nodes: HashSet::new(),
                out_edges: HashMap::new(),
                in_edges: HashMap::new(),
            },
        );
        BDDDiagramDrawer {
            manager: &diagram.manager,
            root: diagram.root,
            node_by_id: HashMap::new(),
            group_id_by_node: HashMap::new(),
            group_by_id: groups,
            free_ids: FreeIdManager::new(1),
            node_parents: HashMap::new(),
        }
    }
}

impl<'a, M> BDDDiagramDrawer<'a, M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    fn add_parent(&mut self, node: NodeID, parent: NodeID, edge_type: EdgeType<M::EdgeTag>) {
        let parents = self
            .node_parents
            .entry(node)
            .or_insert_with(|| HashSet::new());
        let edge: NodeParentEdge<M> = NodeParentEdge { parent, edge_type };
        parents.insert(edge);
    }

    fn get_parents(&self, node: NodeID) -> Option<&HashSet<NodeParentEdge<M>>> {
        return self.node_parents.get(&node);
    }

    fn get_node_group(&mut self, group_id: NodeGroupID) -> &mut NodeGroup<M> {
        self.group_by_id.get_mut(&group_id).unwrap()
    }

    fn get_node_group_id(&self, node: NodeID) -> NodeGroupID {
        if let Some(group_id) = self.group_id_by_node.get(&node) {
            *group_id
        } else {
            0
        }
    }

    fn get_id_by_edge(&mut self, edge: &M::Edge) -> NodeID {
        let id = edge.node_id();
        self.node_by_id.insert(id, self.manager.get_node(edge));
        return id;
    }
    fn get_node_by_id(&self, id: NodeID) -> Option<&Node<'a, M>> {
        return self.node_by_id.get(&id);
    }

    fn remove_group(&mut self, id: NodeGroupID) {
        self.node_by_id.remove(&id);
        self.free_ids.make_available(id);
    }

    fn remove_edges_to_set(
        edges: &mut EdgeSet<M>,
        edge_type: EdgeType<M::EdgeTag>,
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
        edge_type: EdgeType<M::EdgeTag>,
        count: i32,
    ) {
        let from_group = self.get_node_group(from);
        BDDDiagramDrawer::<'a, M>::remove_edges_to_set(
            &mut from_group.out_edges,
            edge_type,
            to,
            count,
        );

        let to_group = self.get_node_group(to);
        BDDDiagramDrawer::<'a, M>::remove_edges_to_set(
            &mut to_group.in_edges,
            edge_type,
            from,
            count,
        );
    }

    fn add_edges_to_set(
        edges: &mut EdgeSet<M>,
        edge_type: EdgeType<M::EdgeTag>,
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
        edge_type: EdgeType<M::EdgeTag>,
        count: i32,
    ) {
        let from_group = self.get_node_group(from);
        BDDDiagramDrawer::<'a, M>::add_edges_to_set(
            &mut from_group.out_edges,
            edge_type,
            to,
            count,
        );

        let to_group = self.get_node_group(to);
        BDDDiagramDrawer::<'a, M>::add_edges_to_set(&mut to_group.in_edges, edge_type, from, count);
    }
}
impl<'a, M> DiagramDrawer for BDDDiagramDrawer<'a, M>
where
    M: Manager<Terminal = BDDTerminal> + HasApplyCache<M, Operator = BDDOp>,
    M::InnerNode: HasLevel,
{
    fn render(&self, time: i64, selected_ids: &[u32], hovered_ids: &[u32]) -> () {
        todo!()
    }

    fn layout(&mut self) -> () {
        todo!()
    }

    fn set_transform(&mut self, x: i32, y: i32, scale: f32) -> () {
        todo!()
    }

    fn set_step(&mut self, step: i32) -> Option<crate::wasm_interface::StepData> {
        todo!()
    }

    fn set_group(
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
                let cur_group = self.get_node_group(cur_group_id);
                let contained = cur_group.nodes.remove(&from_id);
                let from_empty = cur_group.nodes.is_empty();

                self.get_node_group(to).nodes.insert(from_id);

                if let Some(Node::Inner(inner_node)) = self.get_node_by_id(from_id) {
                    let mut index = 0;
                    for edge in inner_node.children() {
                        let edge_type = EdgeType::new(edge.tag(), index);
                        index += 1;

                        let child_id = self.get_id_by_edge(&edge);
                        self.add_parent(child_id, from_id, edge_type);

                        let child_group_id = self.get_node_group_id(child_id);
                        if (contained && cur_group_id != child_group_id) {
                            self.remove_edges(cur_group_id, child_group_id, edge_type, 1);
                        }
                        if (to != child_group_id) {
                            self.add_edges(to, child_group_id, edge_type, 1);
                        }

                        // Ensure the child id is in there,
                        if (child_group_id == 0) {
                            let child_group = self.get_node_group(child_group_id);
                            child_group.nodes.insert(child_id);
                        }
                    }
                }
                if let Some(parents) = self.get_parents(from_id) {
                    let p = (*parents).clone();
                    for parent_edge in p {
                        let edge_type = parent_edge.edge_type;
                        let edge_from = parent_edge.parent;

                        if (contained && edge_from != cur_group_id) {
                            self.remove_edges(edge_from, cur_group_id, edge_type, 1);
                        }
                        if (edge_from != to) {
                            self.add_edges(edge_from, to, edge_type, 1);
                        }
                    }
                }

                if cur_group_id != 0 && from_empty {
                    self.remove_group(cur_group_id);
                }
            } else if from_id == to {
                continue;
            } else if from_id == 0 {
                let init_nodes = self.get_node_group(from_id).nodes.clone();
                let mut found: HashSet<NodeID> = init_nodes.clone().into_iter().collect();
                let mut queue: LinkedList<NodeID> = init_nodes.into_iter().collect();

                while !queue.is_empty() {
                    let node_id = queue.pop_front().unwrap();
                    let Some(Node::Inner(inner_node)) = self.get_node_by_id(node_id) else {
                        continue;
                    };

                    for edge in inner_node.children() {
                        let child_id = self.get_id_by_edge(&edge);
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
                let from_group = self.get_node_group(from_id);
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

                let to_group = self.get_node_group(to);
                to_group.nodes.extend(&from_nodes);
                self.remove_group(from_id);
            } else {
                return false;
            }
        }

        return true;
    }

    fn create_group(
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

    fn get_nodes(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Vec<crate::wasm_interface::NodeGroupID> {
        todo!()
    }
}
