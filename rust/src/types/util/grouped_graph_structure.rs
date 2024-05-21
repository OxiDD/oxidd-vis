use std::vec::IntoIter;

use oxidd::LevelNo;
use oxidd_core::Tag;

use crate::wasm_interface::{NodeGroupID, NodeID};

use super::edge_type::EdgeType;

pub trait GroupedGraphStructure<T: Tag> {
    fn get_root(&self) -> NodeGroupID;
    fn get_all_groups(&self) -> Vec<NodeGroupID>;
    fn get_hidden(&self) -> Option<NodeGroupID>;
    fn get_group(&self, node: NodeID) -> NodeGroupID;
    fn get_parents(&self, group: NodeGroupID) -> IntoIter<(EdgeType<T>, NodeGroupID, i32)>;
    fn get_children(&self, group: NodeGroupID) -> IntoIter<(EdgeType<T>, NodeGroupID, i32)>;
    fn get_nodes_of_group(&self, group: NodeGroupID) -> IntoIter<NodeID>;
    fn get_level_range(&self, group: NodeGroupID) -> (LevelNo, LevelNo);
}
