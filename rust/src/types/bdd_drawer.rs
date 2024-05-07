use oxidd_core::DiagramRules;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::hash::Hash;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use web_sys::console::log;

use crate::traits::Diagram;
use crate::traits::DiagramDrawer;
use crate::util::free_id_manager::FreeIdManager;
use crate::util::logging::console;
use crate::wasm_interface::NodeGroupID;
use crate::wasm_interface::NodeID;
use crate::wasm_interface::TargetID;
use crate::wasm_interface::TargetIDType;
use oxidd::bdd;
use oxidd::bdd::BDDFunction;
use oxidd::util::Borrowed;
use oxidd::BooleanFunction;
use oxidd::Edge;
use oxidd::Function;
use oxidd::InnerNode;
use oxidd::{Manager, ManagerRef};
use oxidd_core::HasApplyCache;
use oxidd_core::HasLevel;
use oxidd_core::Node;
use oxidd_core::{util::DropWith, Tag};
use oxidd_rules_bdd::simple::BDDOp;
use oxidd_rules_bdd::simple::BDDTerminal;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

pub struct BDDDiagram<MR: ManagerRef, F: Function<ManagerRef = MR> + 'static>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    manager_ref: MR,
    root: Rc<F>,
}
impl<MR: ManagerRef, F: Function<ManagerRef = MR>> BDDDiagram<MR, F>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    pub fn new(mut manager_ref: MR, root: impl Fn(&mut MR) -> F) -> BDDDiagram<MR, F> {
        // let mut manager_ref = manager_ref;
        BDDDiagram {
            root: Rc::new(root(&mut manager_ref)),
            manager_ref,
        }
    }

    // fn okay(&self, canvas: HtmlCanvasElement) -> Box<dyn BoxB<Rc<F>>> {
    //     let root_clone = self.root.clone();
    //     Box::new(Box::new(root_clone))
    // }
}

// struct BoxB<V>(Rc<V>);

// struct SomeData<T>(Rc<T>);
// impl<T> SomeData<T> {
//     fn some_func(&self) -> Box<dyn SomeTrait<T>> {
//         return Box::new(WrapperData(self.0.clone()));
//     }
// }

// trait SomeTrait<T> {}
// struct WrapperData<T>(Rc<T>);
// impl<T> SomeTrait<T> for WrapperData<T> {}

impl<
        ET: Tag + 'static,
        T,
        E: Edge<Tag = ET>,
        N: InnerNode<E> + HasLevel,
        R: DiagramRules<E, N, T>,
        MR: ManagerRef,
        F: Function<ManagerRef = MR> + 'static,
    > Diagram for BDDDiagram<MR, F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn create_drawer(&self, canvas: HtmlCanvasElement) -> Box<dyn DiagramDrawer> {
        let context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();

        let root = &self.root;
        let root_clone = root.clone();
        let diagram = BDDDiagramDrawer::new(root_clone, context);
        Box::new(diagram)
    }
}

fn k<F: Clone>(f: F) -> Box<F> {
    Box::new(f)
}

type EdgeSet<T: Tag> = HashMap<NodeGroupID, HashMap<EdgeType<T>, i32>>;
struct NodeGroup<T: Tag> {
    nodes: HashSet<NodeID>,
    out_edges: EdgeSet<T>,
    in_edges: EdgeSet<T>,
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
struct NodeParentEdge<T: Tag> {
    parent: NodeID,
    edge_type: EdgeType<T>,
}
impl<T: Tag> Clone for NodeParentEdge<T> {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone(),
            edge_type: self.edge_type.clone(),
        }
    }
}

impl<T: Tag> Hash for NodeParentEdge<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.parent.hash(state);
        self.edge_type.hash(state);
    }
}
impl<T: Tag> Eq for NodeParentEdge<T> {}
impl<T: Tag> PartialEq for NodeParentEdge<T> {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.edge_type == other.edge_type
    }
}

#[derive(Copy, Clone)]
pub struct Point {
    x: f32,
    y: f32,
}
pub struct Transition<T> {
    old_time: i32, // ms
    duration: i32, // ms
    old: T,
    new: T,
}
impl<T: Copy> Transition<T> {
    fn new(val: T) -> Transition<T> {
        Transition {
            old: val,
            new: val,
            old_time: 0,
            duration: 0,
        }
    }
}

pub struct NodeGroupLayout<T: Tag> {
    top_left: Transition<Point>,
    bottom_right: Transition<Point>,
    label: String,
    exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
    edges: HashMap<NodeGroupID, HashMap<EdgeType<T>, EdgeLayout>>,
}
pub struct EdgeLayout {
    points: Vec<EdgePoint>,
    to: NodeGroupID,
}
pub struct EdgePoint {
    point: Transition<Point>,
    is_jump: Transition<f32>, // Whether this point represents an edge crossing, or just a bend
}

pub struct LayerLayout {
    start_layer: i32,
    end_layer: i32,
    top: Transition<f32>,
    bottom: Transition<f32>,
}

pub struct DiagramLayout<T: Tag> {
    groups: HashMap<NodeGroupID, NodeGroupLayout<T>>,
    layers: HashMap<i32, Rc<LayerLayout>>,
}

pub struct BDDDiagramDrawer<T: Tag, F: Function>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = T>,
{
    root: Rc<F>,
    webgl_context: WebGl2RenderingContext,
    node_by_id: HashMap<NodeID, F>,
    // Nodes are implicitly in group 0 by default, I.e either:
    // - group_by_id[group_id_by_node[node]].nodes.contains(node)
    // - or !group_id_by_node.contains(node) && !exists g. group_by_id[g].nodes.contains(node)
    group_id_by_node: HashMap<NodeID, NodeGroupID>,
    group_by_id: HashMap<NodeGroupID, NodeGroup<T>>,
    free_ids: FreeIdManager<usize>,
    // The known parents of a node (based on what node have been moved out of the default group)
    node_parents: HashMap<NodeID, HashSet<NodeParentEdge<T>>>,
    layout: DiagramLayout<T>,
}

impl<ET: Tag, T, E: Edge<Tag = ET>, N: InnerNode<E>, R: DiagramRules<E, N, T>, F: Function>
    BDDDiagramDrawer<ET, F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    pub fn new(root: Rc<F>, webgl_context: WebGl2RenderingContext) -> BDDDiagramDrawer<ET, F> {
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
            webgl_context,
            root: root,
            node_by_id: HashMap::new(),
            group_id_by_node: HashMap::new(),
            group_by_id: groups,
            free_ids: FreeIdManager::new(1),
            node_parents: HashMap::new(),
            layout: DiagramLayout {
                layers: HashMap::from([(
                    0,
                    Rc::new(LayerLayout {
                        start_layer: 0,
                        end_layer: 0,
                        top: Transition::new(0.),
                        bottom: Transition::new(0.),
                    }),
                )]),
                groups: HashMap::from([(
                    0,
                    NodeGroupLayout {
                        top_left: Transition::new(Point { x: 0., y: 0. }),
                        bottom_right: Transition::new(Point { x: 10., y: 10. }),
                        label: "hello".to_string(),
                        exists: Transition::new(1.),
                        edges: HashMap::from([]),
                    },
                )]),
            },
        }
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

    fn get_node_group(&mut self, group_id: NodeGroupID) -> &mut NodeGroup<ET> {
        self.group_by_id.get_mut(&group_id).unwrap()
    }

    fn get_node_group_id(&self, node: NodeID) -> NodeGroupID {
        if let Some(group_id) = self.group_id_by_node.get(&node) {
            *group_id
        } else {
            0
        }
    }

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

    fn remove_group(&mut self, id: NodeGroupID) {
        self.node_by_id.remove(&id);
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
        let from_group = self.get_node_group(from);
        BDDDiagramDrawer::<ET, F>::remove_edges_to_set(
            &mut from_group.out_edges,
            edge_type,
            to,
            count,
        );

        let to_group = self.get_node_group(to);
        BDDDiagramDrawer::<ET, F>::remove_edges_to_set(
            &mut to_group.in_edges,
            edge_type,
            from,
            count,
        );
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
        let from_group = self.get_node_group(from);
        BDDDiagramDrawer::<ET, F>::add_edges_to_set(
            &mut from_group.out_edges,
            edge_type,
            to,
            count,
        );

        let to_group = self.get_node_group(to);
        BDDDiagramDrawer::<ET, F>::add_edges_to_set(&mut to_group.in_edges, edge_type, from, count);
    }
}

impl<
        ET: Tag,
        T,
        E: Edge<Tag = ET>,
        N: InnerNode<E> + HasLevel,
        R: DiagramRules<E, N, T>,
        F: Function,
    > DiagramDrawer for BDDDiagramDrawer<ET, F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn render(&self, time: i32, selected_ids: &[u32], hovered_ids: &[u32]) -> () {
        let children = self.get_children(&self.root);
        for (_, child) in children {
            let c: F = child;

            let level = c.with_manager_shared(|mgr, f| mgr.get_node(f).unwrap_inner().level());
            console::log!("{}", level);
        }
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

                        if contained && edge_from != cur_group_id {
                            self.remove_edges(edge_from, cur_group_id, edge_type, 1);
                        }
                        if edge_from != to {
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
