use super::wasm_interface::{NodeGroupID, StepData, TargetID};

pub trait Diagram {
    fn create_drawer(&self) -> Box<DiagramDrawer>;
}

pub trait DiagramDrawer {
    fn render(&self, time: i64, selected_ids: &[u32], hovered_ids: &[u32]) -> ();
    fn layout(&mut self) -> ();
    fn set_transform(&mut self, x: i32, y: i32, scale: f32) -> ();
    fn set_step(&mut self, step: i32) -> Option<StepData>;
    fn set_group(&mut self, from: Vec<TargetID>, to: NodeGroupID) -> bool;
    fn create_group(&mut self, from: Vec<TargetID>) -> NodeGroupID;
    fn get_nodes(&self, x: i32, y: i32, width: i32, height: i32) -> Vec<NodeGroupID>;
}
