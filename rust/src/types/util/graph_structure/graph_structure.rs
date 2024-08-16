use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
    iter::{self, FromIterator},
    rc::Rc,
};

use oxidd::{Edge, Function, InnerNode, LevelNo, Manager};
use oxidd_core::{DiagramRules, HasLevel, Node, Tag};

use crate::{util::logging::console, wasm_interface::NodeID};

/// A graph structure trait used as the data to visualize
pub trait GraphStructure<T: DrawTag, NL: Clone, LL: Clone> {
    fn get_root(&self) -> NodeID;
    /// Only returns connections that have already been discovered by calling get_children
    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)>;
    /// This is only supported for nodeIDs that have been obtained from this interface before
    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)>;
    fn get_level(&mut self, node: NodeID) -> LevelNo;

    // Labels for displaying information about nodes
    fn get_node_label(&self, node: NodeID) -> NL;
    fn get_level_label(&self, level: LevelNo) -> LL;

    /// Registers a change listener.
    /// Change events are invoked for all nodes that have been obtained from this interface before, but might not be invoked for not yet "discovered" nodes
    fn on_change(&mut self, listener: Box<GraphListener>) -> usize;
    fn off_change(&mut self, listener: usize);
}

pub type GraphListener = dyn Fn(&Vec<Change>) -> ();
#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Change {
    NodeLabelChange { node: NodeID },
    LevelChange { node: NodeID },
    LevelLabelChange { level: LevelNo },
    NodeConnectionsChange { node: NodeID },
    NodeRemoval { node: NodeID },
    NodeInsertion { node: NodeID },
}

pub trait DrawTag: Tag + Hash + Ord {}
impl DrawTag for () {}

#[derive(Eq, PartialEq, Copy, Clone, PartialOrd, Ord, Hash)]
pub struct EdgeType<T: DrawTag> {
    tag: T,
    index: i32,
}
impl<T: DrawTag> EdgeType<T> {
    pub fn new(tag: T, index: i32) -> EdgeType<T> {
        EdgeType { tag, index }
    }
}
