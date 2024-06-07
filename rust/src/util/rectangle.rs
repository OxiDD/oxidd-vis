use std::fmt::Display;

use super::matrix4::Matrix4;

pub struct Rectangle {
    pub x: f32, // left
    pub y: f32, // bottom
    pub width: f32,
    pub height: f32,
}

impl Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bottom_left: ({}, {}), size: ({}, {})",
            self.x, self.y, self.width, self.height,
        )
    }
}

impl Rectangle {
    pub fn new(
        x: f32, // left
        y: f32, // bottom
        width: f32,
        height: f32,
    ) -> Rectangle {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }
    pub fn overlaps(&self, other: &Rectangle) -> bool {
        self.x + self.width >= other.x
            && other.x + other.width >= self.x
            && self.y + self.height >= other.y
            && other.y + other.height >= self.y
    }

    pub fn contains(&self, other: &Rectangle) -> bool {
        self.x + self.width >= other.x + other.width
            && other.x >= self.x
            && self.y + self.height >= other.y + other.height
            && other.y >= self.y
    }

    /// Retrieves the bounding box rectangle of the transformation matrix being applied to this rectangle
    pub fn transform(&self, matrix: Matrix4) -> Rectangle {
        let p1 = matrix.mul_vec3((self.x, self.y, 0.0));
        let p2 = matrix.mul_vec3((self.x + self.width, self.y, 0.0));
        let p3 = matrix.mul_vec3((self.x + self.width, self.y + self.height, 0.0));
        let p4 = matrix.mul_vec3((self.x, self.y + self.height, 0.0));
        let points = vec![p2, p3, p4];
        let x_min = points.iter().fold(p1.0, |a, b| f32::min(a, b.0));
        let x_max = points.iter().fold(p1.0, |a, b| f32::max(a, b.0));
        let y_min = points.iter().fold(p1.1, |a, b| f32::min(a, b.1));
        let y_max = points.iter().fold(p1.1, |a, b| f32::max(a, b.1));
        Rectangle {
            x: x_min,
            y: y_min,
            width: x_max - x_min,
            height: y_max - y_min,
        }
    }
}
