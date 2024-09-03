use crate::{
    types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceRemainder,
    util::rectangle::Rectangle, wasm_interface::NodeID,
};

use super::wasm_interface::{NodeGroupID, StepData, TargetID};
use web_sys::HtmlCanvasElement;

pub trait Diagram {
    fn create_section_from_dddmp(&mut self, dddmp: String) -> Option<Box<dyn DiagramSection>>; // TODO: error type
    fn create_section_from_id(
        &self,
        section: &Box<dyn DiagramSection>,
        id: NodeID,
    ) -> Option<Box<dyn DiagramSection>>;
}

pub trait DiagramSection {
    fn create_drawer(&self, canvas: HtmlCanvasElement) -> Box<dyn DiagramSectionDrawer>;
}

pub trait DiagramSectionDrawer {
    fn render(&mut self, time: u32) -> ();
    fn layout(&mut self, time: u32) -> ();
    fn set_transform(&mut self, width: u32, height: u32, x: f32, y: f32, scale: f32) -> ();
    fn set_step(&mut self, step: i32) -> Option<StepData>;
    fn set_group(&mut self, from: Vec<TargetID>, to: NodeGroupID) -> bool;
    fn create_group(&mut self, from: Vec<TargetID>) -> NodeGroupID;
    /// Retrieves the nodes in the given rectangle, expanding each node group up to at most max_group_expansion nodes of the nodes it contains
    fn get_nodes(&self, area: Rectangle, max_group_expansion: usize) -> Vec<NodeID>;
    /// The selected and hover _ids are node ids, not node group ids
    fn set_selected_nodes(&mut self, selected_ids: &[NodeID], hovered_ids: &[NodeID]);

    /// Splits the edges of a given group such that each edge type goes to a unique group, if fully is specified it also ensures that each group that an edge goes to only contains a single node
    fn split_edges(&mut self, nodes: &[NodeID], fully: bool) -> ();
    fn set_terminal_mode(&mut self, terminal: String, mode: PresenceRemainder) -> ();
}
