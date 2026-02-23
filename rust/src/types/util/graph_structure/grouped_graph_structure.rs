use std::{borrow::Borrow, cell::RefCell, collections::HashSet, hash::Hash, rc::Rc, vec::IntoIter};

use oxidd::LevelNo;
use oxidd_core::Tag;

use crate::wasm_interface::{NodeGroupID, NodeID};

use super::graph_structure::{DrawTag, EdgeType};

pub trait GroupedGraphStructure {
    type T: DrawTag;
    type GL;
    type LL;
    type Tracker: NodeTracker;
    fn get_roots(&self) -> Vec<NodeGroupID>;
    fn get_all_groups(&self) -> Vec<NodeGroupID>;
    fn get_hidden(&self) -> Vec<NodeGroupID>;
    fn get_group(&self, node: NodeID) -> NodeGroupID;
    fn get_group_label(&self, group: NodeID) -> Self::GL;
    fn get_parents(&self, group: NodeGroupID) -> Vec<EdgeCountData<Self::T>>;
    fn get_children(&self, group: NodeGroupID) -> Vec<EdgeCountData<Self::T>>;
    fn get_nodes_of_group(&self, group: NodeGroupID) -> Vec<NodeID>;
    fn get_level_range(&self, group: NodeGroupID) -> (LevelNo, LevelNo);
    fn get_level_label(&self, level: LevelNo) -> Self::LL;
    /// Refreshes the node groups according to changes of the underlying graph
    fn refresh(&mut self);
    /// Retrieves a node-tracker that for every node tracks its source (that it got created from), and whether it and its source ids can be reused
    fn create_node_tracker(&mut self) -> Self::Tracker;
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub struct EdgeData<T: DrawTag> {
    pub to: NodeGroupID,
    pub from_level: LevelNo,
    pub to_level: LevelNo,
    pub edge_type: EdgeType<T>,
}
impl<T: DrawTag> EdgeData<T> {
    pub fn new(
        to: NodeGroupID,
        from_level: LevelNo,
        to_level: LevelNo,
        edge_type: EdgeType<T>,
    ) -> EdgeData<T> {
        EdgeData {
            to,
            from_level,
            to_level,
            edge_type,
        }
    }
    pub fn to_string(&self, from: usize) -> String {
        format!(
            "{}:{} -{}-> {}:{}",
            from, self.from_level, self.edge_type.index, self.to, self.to_level
        )
    }
}

#[derive(PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct EdgeCountData<T: DrawTag> {
    pub to: NodeGroupID,
    pub from_level: LevelNo,
    pub to_level: LevelNo,
    pub edge_type: EdgeType<T>,
    pub count: usize,
}
impl<T: DrawTag> EdgeCountData<T> {
    pub fn new(
        to: NodeGroupID,
        from_level: LevelNo,
        to_level: LevelNo,
        edge_type: EdgeType<T>,
        count: usize,
    ) -> EdgeCountData<T> {
        EdgeCountData {
            to,
            from_level,
            to_level,
            edge_type,
            count,
        }
    }

    pub fn drop_count(&self) -> EdgeData<T> {
        EdgeData {
            to: self.to,
            from_level: self.from_level,
            to_level: self.to_level,
            edge_type: self.edge_type,
        }
    }
}
impl<T: DrawTag> Hash for EdgeCountData<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to.hash(state);
        self.from_level.hash(state);
        self.to_level.hash(state);
        self.edge_type.hash(state);
        self.count.hash(state);
    }
}
pub trait NodeTracker: SourceReader {
    // Deletes all nodes that do not pass the provided filter
    fn retain<F: Fn(NodeGroupID) -> bool>(&mut self, filter: F) -> ();
}

pub trait SourceReader {
    /// Retrieves the group(s) that the given group originates (is created/split up/merged) from, such that for Some(s) = get_source(group) we have get_source(s) = None
    fn get_sources(&self, group: NodeGroupID) -> Vec<NodeGroupID>;
    /// Removes all of the sources from this reader, so they are no longer returned
    fn remove_sources(&mut self);
}
