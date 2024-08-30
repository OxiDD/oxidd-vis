use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    ops::{Add, Mul, Sub},
    rc::Rc,
};

use oxidd::LevelNo;
use oxidd_core::Tag;

use crate::{
    types::util::graph_structure::{graph_structure::DrawTag, grouped_graph_structure::EdgeData},
    util::rectangle::Rectangle,
    wasm_interface::{NodeGroupID, NodeID},
};

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn distance(&self, other: &Point) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
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
impl<R: Clone> Mul<R> for Point
where
    f32: Mul<R, Output = f32>,
{
    type Output = Point;

    fn mul(self, rhs: R) -> Self::Output {
        Point {
            x: self.x * rhs.clone(),
            y: self.y * rhs,
        }
    }
}

impl Mul<Point> for f32 {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Point {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}
impl Default for Point {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
        }
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
            old_time: u32::max(rhs.old_time, self.old_time),
            duration: u32::max(rhs.duration, self.duration),
            old: self.old + rhs.old,
            new: self.new + rhs.new,
        }
    }
}
impl<T: Display> Display for Transition<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {} @ {} for {}",
            self.old, self.new, self.old_time, self.duration
        )
    }
}

// impl<T, L: Clone + Sized + Mul<T, Output = T>> Mul<Transition<T>> for L {
//     type Output = Transition<T>;

//     fn mul(self, rhs: Transition<T>) -> Self::Output {
//         Transition {
//             old_time: rhs.old_time,
//             duration: rhs.duration,
//             old: self * rhs.old,
//             new: self * rhs.new,
//         }
//     }
// }
impl<R: Clone, T: Mul<R, Output = T>> Mul<R> for Transition<T> {
    type Output = Transition<T>;

    fn mul(self, rhs: R) -> Self::Output {
        Transition {
            old_time: self.old_time,
            duration: self.duration,
            old: self.old * rhs.clone(),
            new: self.new * rhs,
        }
    }
}

#[derive(Clone)]
pub struct NodeGroupLayout<T: DrawTag> {
    pub position: Transition<Point>, // Bottom left point
    pub size: Transition<Point>,
    pub label: String,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
    pub edges: HashMap<EdgeData<T>, EdgeLayout>,
    pub level_range: (LevelNo, LevelNo),
    pub color: Transition<(f32, f32, f32)>,
}
impl<T: DrawTag> NodeGroupLayout<T> {
    // TODO: possibly consider the selection time? (animations should be quick and not have a huge effect however)
    pub fn get_rect(&self) -> Rectangle {
        let width = self.size.new.x;
        let height = self.size.new.y;
        Rectangle::new(self.position.new.x, self.position.new.y, width, height)
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
pub struct LayerLayout {
    pub start_layer: LevelNo,
    pub end_layer: LevelNo,
    pub label: String,
    pub top: Transition<f32>,
    pub bottom: Transition<f32>,
    pub index: Transition<f32>,
    pub exists: Transition<f32>,
}

#[derive(Clone)]
pub struct DiagramLayout<T: DrawTag> {
    pub groups: HashMap<NodeGroupID, NodeGroupLayout<T>>,
    /// Note: this vector has to be sorted in increasing order of start_layer
    pub layers: Vec<LayerLayout>,
}
