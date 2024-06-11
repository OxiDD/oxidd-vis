use std::{hash::Hash, vec::IntoIter};

use oxidd::LevelNo;
use oxidd_core::Tag;

use crate::wasm_interface::{NodeGroupID, NodeID};

use super::{edge_type::EdgeType, group_manager::EdgeData};

pub trait GroupedGraphStructure<T: Tag> {
    fn get_root(&self) -> NodeGroupID;
    fn get_all_groups(&self) -> Vec<NodeGroupID>;
    fn get_hidden(&self) -> Option<NodeGroupID>;
    fn get_group(&self, node: NodeID) -> NodeGroupID;
    fn get_parents(&self, group: NodeGroupID) -> IntoIter<EdgeCountData<T>>;
    fn get_children(&self, group: NodeGroupID) -> IntoIter<EdgeCountData<T>>;
    fn get_nodes_of_group(&self, group: NodeGroupID) -> IntoIter<NodeID>;
    fn get_level_range(&self, group: NodeGroupID) -> (LevelNo, LevelNo);
}

#[derive(PartialEq, Eq, Clone)]
pub struct EdgeCountData<T: Tag> {
    pub to: NodeGroupID,
    pub from_level: LevelNo,
    pub to_level: LevelNo,
    pub edge_type: EdgeType<T>,
    pub count: usize,
}
impl<T: Tag> EdgeCountData<T> {
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
impl<T: Tag> Hash for EdgeCountData<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to.hash(state);
        self.from_level.hash(state);
        self.to_level.hash(state);
        self.edge_type.hash(state);
        self.count.hash(state);
    }
}
