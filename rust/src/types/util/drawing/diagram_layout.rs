use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::{Add, Sub},
    rc::Rc,
};

use oxidd_core::Tag;

use crate::{
    types::util::{edge_type::EdgeType, group_manager::EdgeData},
    util::rectangle::Rectangle,
    wasm_interface::{NodeGroupID, NodeID},
};

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    fn distance(&self, other: &Point) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}
impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Copy, Clone)]
pub struct Transition<T> {
    pub old_time: u32, // ms
    pub duration: u32, // ms
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
impl<T: Add<Output = T>> Add for Transition<T> {
    type Output = Transition<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Transition {
            old_time: u32::min(rhs.old_time, self.old_time),
            duration: u32::max(rhs.duration, self.duration),
            old: self.old + rhs.old,
            new: self.new + rhs.new,
        }
    }
}

#[derive(Clone)]
pub struct NodeGroupLayout<T: Tag> {
    pub center_position: Transition<Point>,
    pub size: Transition<Point>,
    pub label: String,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
    pub edges: HashMap<EdgeData<T>, EdgeLayout>,
}
impl<T: Tag> NodeGroupLayout<T> {
    // TODO: possibly consider the selection time? (animations should be quick and not have a huge effect however)
    pub fn get_rect(&self) -> Rectangle {
        let width = self.size.new.x;
        let height = self.size.new.y;
        Rectangle::new(
            self.center_position.new.x - width / 2.0,
            self.center_position.new.y - height / 2.0,
            width,
            height,
        )
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

pub struct LayerLayout {
    pub start_layer: i32,
    pub end_layer: i32,
    pub top: Transition<f32>,
    pub bottom: Transition<f32>,
}

#[derive(Clone)]
pub struct DiagramLayout<T: Tag> {
    pub groups: HashMap<NodeGroupID, NodeGroupLayout<T>>,
    pub layers: HashMap<i32, Rc<LayerLayout>>,
}
