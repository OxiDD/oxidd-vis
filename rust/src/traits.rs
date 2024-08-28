use crate::{
    types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceRemainder,
    util::rectangle::Rectangle,
};

use super::wasm_interface::{NodeGroupID, StepData, TargetID};
use web_sys::HtmlCanvasElement;

pub trait Diagram {
    fn create_drawer(&self, canvas: HtmlCanvasElement) -> Box<dyn DiagramDrawer>;
}

pub trait DiagramDrawer {
    fn render(&mut self, time: u32, selected_ids: &[u32], hovered_ids: &[u32]) -> ();
    fn layout(&mut self, time: u32) -> ();
    fn set_transform(&mut self, width: u32, height: u32, x: f32, y: f32, scale: f32) -> ();
    fn set_step(&mut self, step: i32) -> Option<StepData>;
    fn set_group(&mut self, from: Vec<TargetID>, to: NodeGroupID) -> bool;
    fn create_group(&mut self, from: Vec<TargetID>) -> NodeGroupID;
    fn get_nodes(&self, area: Rectangle) -> Vec<NodeGroupID>;
    /// Splits the edges of a given group such that each edge type goes to a unique group, if fully is specified it also ensures that each group that an edge goes to only contains a single node
    fn split_edges(&mut self, group: NodeGroupID, fully: bool) -> ();
    fn set_terminal_mode(&mut self, terminal: String, mode: PresenceRemainder) -> ();
}
