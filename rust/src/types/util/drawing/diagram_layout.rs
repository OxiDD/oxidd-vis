use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use oxidd_core::Tag;

use crate::{
    types::util::edge_type::EdgeType,
    wasm_interface::{NodeGroupID, NodeID},
};

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
pub struct Transition<T> {
    pub old_time: i32, // ms
    pub duration: i32, // ms
    pub old: T,
    pub new: T,
}
impl<T: Copy> Transition<T> {
    pub fn plain(val: T) -> Transition<T> {
        Transition {
            old: val,
            new: val,
            old_time: 0,
            duration: 0,
        }
    }
}

pub struct NodeGroupLayout<T: Tag> {
    pub top_left: Transition<Point>,
    pub size: Transition<Point>,
    pub label: String,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
    pub edges: HashMap<NodeGroupID, HashMap<EdgeType<T>, EdgeLayout>>,
}
pub struct EdgeLayout {
    pub points: Vec<EdgePoint>,
}
pub struct EdgePoint {
    pub point: Transition<Point>,
    pub is_jump: Transition<f32>, // Whether this point represents an edge crossing, or just a bend
}

pub struct LayerLayout {
    pub start_layer: i32,
    pub end_layer: i32,
    pub top: Transition<f32>,
    pub bottom: Transition<f32>,
}

pub struct DiagramLayout<T: Tag> {
    pub groups: HashMap<NodeGroupID, NodeGroupLayout<T>>,
    pub layers: HashMap<i32, Rc<LayerLayout>>,
}
