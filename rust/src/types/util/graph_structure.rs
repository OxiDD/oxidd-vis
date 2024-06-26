use std::{
    collections::{HashMap, HashSet},
    iter::{self, FromIterator},
    rc::Rc,
};

use oxidd::{Edge, Function, InnerNode, LevelNo, Manager};
use oxidd_core::{DiagramRules, HasLevel, Node, Tag};

use crate::{util::logging::console, wasm_interface::NodeID};

use super::edge_type::EdgeType;

/// A graph structure trait used as the data to visualize
pub trait GraphStructure<T: Tag> {
    fn get_root(&self) -> NodeID;
    /// Only returns connections that have already been discovered by calling get_children
    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)>;
    /// This is only supported for nodeIDs that have been obtained from this interface before
    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)>;
    fn get_level(&mut self, node: NodeID) -> LevelNo;
    fn get_level_label(&self, level: LevelNo) -> String;
}

pub struct OxiddGraphStructure<T: Tag, F: Function>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = T>,
{
    root: F,
    node_by_id: HashMap<NodeID, F>,
    node_parents: HashMap<NodeID, HashSet<(EdgeType<T>, NodeID)>>,
}

impl<T: Tag, F: Function> OxiddGraphStructure<T, F>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = T>,
{
    pub fn new(root: F) -> OxiddGraphStructure<T, F> {
        OxiddGraphStructure {
            node_by_id: HashMap::from([(
                root.with_manager_shared(|manager, edge| edge.node_id()),
                root.clone(),
            )]),
            root,
            node_parents: HashMap::new(),
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

    fn add_parent(&mut self, node: NodeID, parent: NodeID, edge_type: EdgeType<T>) {
        let parents = self
            .node_parents
            .entry(node)
            .or_insert_with(|| HashSet::new());
        parents.insert((edge_type, parent));
    }
}

impl<
        ET: Tag + 'static,
        T: 'static,
        E: Edge<Tag = ET> + 'static,
        N: InnerNode<E> + HasLevel + 'static,
        R: DiagramRules<E, N, T> + 'static,
        F: Function + 'static,
    > GraphStructure<ET> for OxiddGraphStructure<ET, F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn get_root(&self) -> NodeID {
        self.root
            .with_manager_shared(|manager, edge| edge.node_id())
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<ET>, NodeID)> {
        if let Some(edges) = self.node_parents.get(&node) {
            return Vec::from_iter(edges.iter().map(|&r| r));
        }
        return Vec::new();
    }

    fn get_children(&mut self, node_id: NodeID) -> Vec<(EdgeType<ET>, NodeID)> {
        let root_node = &self.get_node_by_id(node_id);
        if let Some(node) = root_node {
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
                manager.get_node(edge).unwrap_inner().level()
            });
        }
        0
    }

    fn get_level_label(&self, level: LevelNo) -> String {
        // TODO: get actual level vars
        level.to_string()
    }
}
