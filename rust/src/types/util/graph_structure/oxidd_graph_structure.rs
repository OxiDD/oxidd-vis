use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    iter::FromIterator,
    marker::PhantomData,
    rc::Rc,
};

use oxidd::{Edge, Function, InnerNode, LevelNo, Manager, NodeID};
use oxidd_core::{DiagramRules, HasLevel, Node};

use crate::{types::util::storage::state_storage::StateStorage, util::logging::console};

use super::{
    graph_manipulators::pointer_node_adjuster::WithPointerLabels,
    graph_structure::{
        Change, DrawTag, EdgeType, GraphEventsReader, GraphEventsWriter, GraphStructure,
    },
};

pub struct OxiddGraphStructure<DT: DrawTag, F: Function, T, S: Fn(&T) -> String>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = DT>,
{
    roots: Vec<F>,
    node_by_id: HashMap<NodeID, F>,
    pointers: HashMap<NodeID, Vec<String>>,
    node_parents: HashMap<NodeID, HashSet<(EdgeType<DT>, NodeID)>>,
    terminal_to_string: S,
    level_labels: Vec<String>,
    terminal: PhantomData<T>,
    event_writer: GraphEventsWriter,
}

#[derive(Clone)]
pub struct NodeLabel<T> {
    pub pointers: Vec<String>,
    pub kind: NodeType<T>,
}
impl<T> WithPointerLabels for NodeLabel<T> {
    fn get_pointer_labels(&self) -> Vec<String> {
        self.pointers.clone()
    }
}

#[derive(Clone)]
pub enum NodeType<T> {
    Inner(String),
    Terminal(T),
}

impl<DT: DrawTag, F: Function, T, S: Fn(&T) -> String> OxiddGraphStructure<DT, F, T, S>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = DT, Terminal = T>,
{
    pub fn new(
        roots: Vec<(F, Vec<String>)>,
        level_labels: Vec<String>,
        terminal_to_string: S,
    ) -> OxiddGraphStructure<DT, F, T, S> {
        OxiddGraphStructure {
            node_by_id: roots
                .iter()
                .map(|(root, _)| {
                    (
                        root.with_manager_shared(|_, edge| edge.node_id()),
                        root.clone(),
                    )
                })
                .collect(),
            roots: roots.iter().map(|(f, _)| f.clone()).collect(),
            pointers: roots
                .iter()
                .map(|(f, pointers)| {
                    (
                        f.with_manager_shared(|_, edge| edge.node_id()),
                        pointers.clone(),
                    )
                })
                .collect(),
            level_labels,
            node_parents: HashMap::new(),
            terminal_to_string,
            event_writer: GraphEventsWriter::new(),
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
        self.event_writer
            .write(Change::ParentDiscover { child: node });
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
    > StateStorage for OxiddGraphStructure<ET, F, T, S>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
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
    fn get_roots(&self) -> Vec<NodeID> {
        self.roots
            .iter()
            .map(|root| root.with_manager_shared(|manager, edge| edge.node_id()))
            .collect()
    }

    fn get_terminals(&self) -> Vec<NodeID> {
        if let Some(root) = self.roots.first() {
            root.with_manager_shared(|manager, edge| {
                manager.terminals().map(|t| t.node_id()).collect()
            })
        } else {
            Vec::new()
        }
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
            let r = node.with_manager_shared(|manager, edge| manager.get_node(edge).level());
            return r;
        }
        0
    }

    fn get_level_label(&self, level: LevelNo) -> String {
        self.level_labels
            .get(level as usize)
            .cloned()
            .unwrap_or("".to_string())
    }

    fn get_node_label(&self, node: NodeID) -> NodeLabel<String> {
        let kind = if let Some(node) = self.get_node_by_id(node) {
            node.with_manager_shared(|manager, edge| match manager.get_node(edge) {
                Node::Inner(n) => NodeType::Inner(edge.node_id().to_string()),
                Node::Terminal(t) => NodeType::Terminal((&self.terminal_to_string)(t.borrow())),
            })
        } else {
            NodeType::Inner("Not found".to_string())
        };

        NodeLabel {
            pointers: self.pointers.get(&node).cloned().unwrap_or_else(|| vec![]),
            kind,
        }
    }

    fn create_event_reader(&mut self) -> GraphEventsReader {
        self.event_writer.create_reader()
    }
    fn consume_events(&mut self, reader: &GraphEventsReader) -> Vec<Change> {
        self.event_writer.read(reader)
    }

    fn local_nodes_to_sources(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        nodes
    }
    fn source_nodes_to_local(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        nodes
    }
}
