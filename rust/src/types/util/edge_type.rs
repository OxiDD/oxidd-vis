use oxidd_core::Tag;
use std::hash::Hash;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct EdgeType<T: Tag> {
    tag: T,
    index: i32,
}
impl<T: Tag> EdgeType<T> {
    pub fn new(tag: T, index: i32) -> EdgeType<T> {
        EdgeType { tag, index }
    }
}
impl<T: Tag> Hash for EdgeType<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}
