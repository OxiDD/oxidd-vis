use oxidd_core::Tag;
use std::hash::Hash;

use super::graph_structure::DrawTag;

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
