use oxidd_core::Tag;

use crate::{
    types::util::graph_structure::{
        graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
    },
    util::{transformation::Transformation, transition::Interpolatable},
    wasm_interface::NodeGroupID,
};

use super::{
    diagram_layout::{DiagramLayout, LayerStyle, NodeStyle},
    layout_rules::LayoutRules,
};

/// A trait for rendering a given layout
pub trait Renderer<L: LayoutRules> {
    fn set_transform(&mut self, transform: Transformation);
    fn update_layout(&mut self, layout: &DiagramLayout<L::T, L::NS, L::LS>);
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
