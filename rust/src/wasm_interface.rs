use std::rc::Rc;

use super::traits::{Diagram, DiagramDrawer};
use oxidd::ManagerRef;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct VisualizationManager();

#[wasm_bindgen]
impl VisualizationManager {
    pub fn create_diagram(&self) -> DiagramBox // And some DD type param
    {
        todo!();
    }
}

// pub trait VisualizationManager: ReturnWasmAbi {
//     fn addDiagram(&self) -> &Drawable; // And some DD type param
//     fn removeDiagram(&self, diagram: &Drawable) -> ();
// }
#[wasm_bindgen]
pub struct DiagramBox(Box<dyn Diagram>);

#[wasm_bindgen]
// Mirror Diagram trait in terms of interface, but using non-dynamic structs
impl DiagramBox {
    pub fn create_drawer(&self) -> DiagramDrawerBox {
        DiagramDrawerBox(self.0.create_drawer())
    }
}

#[wasm_bindgen]
pub struct DiagramDrawerBox(Box<dyn DiagramDrawer>);
#[wasm_bindgen]
impl DiagramDrawerBox {
    pub fn render(&self, time: i64, selected_ids: &[u32], hovered_ids: &[u32]) -> () {
        self.0.render(time, selected_ids, hovered_ids);
    }
    pub fn layout(&mut self) -> () {
        self.0.layout();
    }
    pub fn set_transform(&mut self, x: i32, y: i32, scale: f32) -> () {
        self.0.set_transform(x, y, scale);
    }
    pub fn set_step(&mut self, step: i32) -> Option<StepData> {
        self.0.set_step(step)
    }
    pub fn set_group(&mut self, from: Vec<TargetID>, to: NodeGroupID) -> bool {
        self.0.set_group(from, to)
    }
    pub fn create_group(&mut self, from: Vec<TargetID>) -> NodeGroupID {
        self.0.create_group(from)
    }
    pub fn get_nodes(&self, x: i32, y: i32, width: i32, height: i32) -> Vec<NodeGroupID> {
        self.0.get_nodes(x, y, width, height)
    }
}

#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct StepData {
    pub description: String,
    pub group: StepGroup,
}

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct StepGroup {
    pub start: i32, // Inclusive
    pub end: i32,   // Exclusive
    pub description: String,
    parent: Option<Rc<StepGroup>>,
}

#[wasm_bindgen]
impl StepGroup {
    pub fn get_parent(&self) -> Option<StepGroup> {
        match &self.parent {
            Some(x) => Some((**x).clone()),
            None => None,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub enum TargetIDType {
    NodeID,
    NodeGroupID,
}
// pub type TargetID = (TargetIDType, NodeID);
#[derive(PartialEq)]
#[wasm_bindgen]
pub struct TargetID(pub TargetIDType, pub NodeID);

pub type NodeGroupID = usize;
pub type NodeID = usize;
