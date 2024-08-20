use std::{borrow::Borrow, cell::RefCell, collections::HashSet, hash::Hash, rc::Rc, vec::IntoIter};

use oxidd::LevelNo;
use oxidd_core::Tag;

use crate::wasm_interface::{NodeGroupID, NodeID};

use super::graph_structure::{DrawTag, EdgeType};

pub trait GroupedGraphStructure<T: DrawTag, GL, LL> {
    type Tracker: SourceTracker;
    fn get_root(&self) -> NodeGroupID;
    fn get_all_groups(&self) -> Vec<NodeGroupID>;
    fn get_hidden(&self) -> Option<NodeGroupID>;
    fn get_group(&self, node: NodeID) -> NodeGroupID;
    fn get_group_label(&self, node: NodeID) -> GL;
    fn get_parents(&self, group: NodeGroupID) -> IntoIter<EdgeCountData<T>>;
    fn get_children(&self, group: NodeGroupID) -> IntoIter<EdgeCountData<T>>;
    fn get_nodes_of_group(&self, group: NodeGroupID) -> IntoIter<NodeID>;
    fn get_level_range(&self, group: NodeGroupID) -> (LevelNo, LevelNo);
    fn get_level_label(&self, level: LevelNo) -> LL;
    /// Refreshes the node groups according to changes of the underlying graph
    fn refresh(&mut self);
    /// Retrieves a source reader, which can be used to animate creation of new groups
    fn get_source_reader(&mut self) -> Self::Tracker;
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
pub trait SourceTracker: SourceReader {
    /// Deletes the source (tracking) of the given group, such that a feature call with give get_source(group) = group, until a new group with the same ID is created
    fn delete_source(&mut self, group: NodeGroupID) -> ();
}

pub trait SourceReader {
    /// Retrieves the group that the given group originates (is created/split up/merged) from, such that for s = get_source(group) we have get_source(s) = s
    fn get_source(&self, group: NodeGroupID) -> NodeGroupID;
    /// Retrieves all nodes for which sources are stored
    fn get_sourced_nodes(&self) -> HashSet<NodeGroupID>;
}
