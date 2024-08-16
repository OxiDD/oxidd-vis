use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    iter::FromIterator,
    marker::PhantomData,
    rc::Rc,
};

use oxidd::{Edge, Function, InnerNode, LevelNo, Manager, NodeID};
use oxidd_core::{DiagramRules, HasLevel, Node};

use super::graph_structure::{DrawTag, EdgeType, GraphListener, GraphStructure};

pub struct OxiddGraphStructure<DT: DrawTag, F: Function, T, S: Fn(&T) -> String>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = DT>,
{
    root: F,
    node_by_id: HashMap<NodeID, F>,
    node_parents: HashMap<NodeID, HashSet<(EdgeType<DT>, NodeID)>>,
    terminal_to_string: S,
    terminal: PhantomData<T>,
}

#[derive(Clone)]
pub enum NodeLabel<T> {
    Inner(String),
    Terminal(T),
}

impl<DT: DrawTag, F: Function, T, S: Fn(&T) -> String> OxiddGraphStructure<DT, F, T, S>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = DT, Terminal = T>,
{
    pub fn new(root: F, terminal_to_string: S) -> OxiddGraphStructure<DT, F, T, S> {
        OxiddGraphStructure {
            node_by_id: HashMap::from([(
                root.with_manager_shared(|manager, edge| edge.node_id()),
                root.clone(),
            )]),
            root,
            node_parents: HashMap::new(),
            terminal_to_string,
            terminal: PhantomData,
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

    fn add_parent(&mut self, node: NodeID, parent: NodeID, edge_type: EdgeType<DT>) {
        let parents = self
            .node_parents
            .entry(node)
            .or_insert_with(|| HashSet::new());
        parents.insert((edge_type, parent));
    }
}

impl<
        ET: DrawTag + 'static,
        T: Clone + 'static,
        E: Edge<Tag = ET> + 'static,
        N: InnerNode<E> + HasLevel + 'static,
        R: DiagramRules<E, N, T> + 'static,
        F: Function + 'static,
        S: Fn(&T) -> String,
    > GraphStructure<ET, NodeLabel<String>, String> for OxiddGraphStructure<ET, F, T, S>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn get_root(&self) -> NodeID {
        self.root
            .with_manager_shared(|manager, edge| edge.node_id())
    }

    fn get_terminals(&self) -> Vec<NodeID> {
        self.root
            .with_manager_shared(|manager, edge| manager.terminals().map(|t| t.node_id()).collect())
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<ET>, NodeID)> {
        if let Some(edges) = self.node_parents.get(&node) {
            return Vec::from_iter(edges.iter().map(|&r| r));
        }
        return Vec::new();
    }

    fn get_children(&mut self, node_id: NodeID) -> Vec<(EdgeType<ET>, NodeID)> {
        let opt_node = &self.get_node_by_id(node_id);
        if let Some(node) = opt_node {
            let cofactors = node.with_manager_shared(move |manager, edge| {
                let tag = edge.tag();
                let internal_node = manager.get_node(edge);
                if let Node::Inner(node) = internal_node {
                    return Some(Vec::from_iter(
                        R::cofactors(tag, node)
                            .map(|f| F::from_edge_ref(manager, &f))
                            .enumerate(),
                    ));
                }
                return None;
            });
            if let Some(cofactors) = cofactors {
                let out = cofactors.iter().map(|(i, f)| {
                    f.with_manager_shared(|manager, edge| {
                        let edge_type = EdgeType::new(edge.tag(), *i as i32);
                        let child_id = self.get_id_by_node(&f);
                        self.add_parent(child_id, node_id, edge_type);
                        (edge_type, child_id)
                    })
                });
                return out.collect();
            }
        }
        return Vec::new();
    }

    fn get_level(&mut self, node_id: NodeID) -> LevelNo {
        if let Some(node) = self.get_node_by_id(node_id) {
            return node.with_manager_shared(|manager, edge| {
                // manager.get_node(edge).unwrap_inner().level()
                manager.get_node(edge).level()
            });
        }
        0
    }

    fn get_level_label(&self, level: LevelNo) -> String {
        // TODO: get actual level vars
        level.to_string()
    }

    fn on_change(&mut self, listener: Box<GraphListener>) -> usize {
        // This diagram never changes
        0
    }
    fn off_change(&mut self, listener: usize) {}

    fn get_node_label(&self, node: NodeID) -> NodeLabel<String> {
        if let Some(node) = self.get_node_by_id(node) {
            return node.with_manager_shared(|manager, edge| match manager.get_node(edge) {
                Node::Inner(n) => NodeLabel::Inner(edge.node_id().to_string()),
                Node::Terminal(t) => NodeLabel::Terminal((&self.terminal_to_string)(t.borrow())),
            });
        } else {
            NodeLabel::Inner("Not found".to_string())
        }
    }
}
