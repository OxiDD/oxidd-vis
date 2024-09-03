use oxidd_core::Tag;

use crate::{
    types::util::graph_structure::graph_structure::DrawTag, util::transformation::Transformation,
    wasm_interface::NodeGroupID,
};

use super::diagram_layout::DiagramLayout;

/// A trait for rendering a given layout
pub trait Renderer<T: DrawTag> {
    fn set_transform(&mut self, transform: Transformation);
    fn update_layout(&mut self, layout: &DiagramLayout<T>);
    fn render(&mut self, time: u32);
    fn select_groups(&mut self, selection: GroupSelection, old_selection: GroupSelection);
}

pub type GroupSelection<'a> = (
    // Selected groups
    &'a [NodeGroupID],
    // Partially selected groups
    &'a [NodeGroupID],
    // Hovered groups
    &'a [NodeGroupID],
    // Partially hovered groups
    &'a [NodeGroupID],
);
