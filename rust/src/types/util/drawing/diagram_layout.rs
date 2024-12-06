use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
    ops::{Add, Mul, Sub},
    rc::Rc,
};

use oxidd::LevelNo;
use oxidd_core::Tag;

use crate::{
    types::util::graph_structure::{graph_structure::DrawTag, grouped_graph_structure::EdgeData},
    util::{
        point::Point,
        rectangle::Rectangle,
        transition::{Interpolatable, Transition},
    },
    wasm_interface::{NodeGroupID, NodeID},
};

#[derive(Clone)]
pub struct NodeGroupLayout<T: DrawTag, S: NodeStyle> {
    /// Bottom center point of the node
    pub position: Transition<Point>,
    pub size: Transition<Point>,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
    pub edges: HashMap<EdgeData<T>, EdgeLayout>,
    pub level_range: (LevelNo, LevelNo),
    pub style: Transition<S>,
}
impl<T: DrawTag, S: NodeStyle> NodeGroupLayout<T, S> {
    // TODO: possibly consider the selection time? (animations should be quick and not have a huge effect however)

    pub fn get_rect(&self, time: Option<u32>) -> Rectangle {
        match time {
            Some(time) => {
                let pos = self.position.get(time);
                let size = self.size.get(time);
                Rectangle::new(pos.x - 0.5 * size.x, pos.y, size.x, size.y)
            }
            _ => {
                let pos = self.position.new;
                let size = self.size.new;
                Rectangle::new(pos.x - 0.5 * size.x, pos.y, size.x, size.y)
            }
        }
    }
}

#[derive(Clone)]
pub struct EdgeLayout {
    pub start_offset: Transition<Point>,
    pub end_offset: Transition<Point>,
    pub points: Vec<EdgePoint>,
    pub exists: Transition<f32>, // Transition for newly created edges
    pub curve_offset: Transition<f32>, // If no bendpoints are used, this curve offset can be used for curving the edge
}

#[derive(Copy, Clone)]
pub struct EdgePoint {
    pub point: Transition<Point>,
    // TODO: give more thought to jumps
    // pub is_jump: Transition<f32>, // Whether this point represents an edge crossing, or just a bend
    pub exists: Transition<f32>, // Whether this point actually exists in the output (it might be used only to transition shape)
}

#[derive(Clone)]
pub struct LayerLayout<S: LayerStyle> {
    pub start_layer: LevelNo,
    pub end_layer: LevelNo,
    pub top: Transition<f32>,
    pub bottom: Transition<f32>,
    pub index: Transition<f32>,
    pub exists: Transition<f32>,
    pub style: Transition<S>,
}

#[derive(Clone)]
pub struct DiagramLayout<T: DrawTag, S: NodeStyle, LS: LayerStyle> {
    pub groups: HashMap<NodeGroupID, NodeGroupLayout<T, S>>,
    /// Note: this vector has to be sorted in increasing order of start_layer
    pub layers: Vec<LayerLayout<LS>>,
}

pub trait LayerStyle: Interpolatable + Clone + Sized {
    fn squash(layers: Vec<Self>) -> Self;
}
pub trait NodeStyle: Interpolatable + Clone {}
