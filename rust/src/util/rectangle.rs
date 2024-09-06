use std::fmt::Display;

use crate::types::util::drawing::diagram_layout::Point;

use super::matrix4::Matrix4;

#[derive(Clone)]
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
            && self.x <= other.x
            && self.y + self.height >= other.y + other.height
            && self.y <= other.y
    }

    pub fn x_range(&self) -> Range {
        Range {
            start: self.x,
            end: self.x + self.width,
        }
    }

    pub fn y_range(&self) -> Range {
        Range {
            start: self.y,
            end: self.y + self.height,
        }
    }

    pub fn pos(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub fn size(&self) -> Point {
        Point {
            x: self.width,
            y: self.height,
        }
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

#[derive(Clone)]
pub struct Range {
    pub start: f32,
    pub end: f32,
}

impl Range {
    pub fn overlaps(&self, other: &Range) -> bool {
        self.start >= other.end && other.start >= self.end
    }

    pub fn contains(&self, other: &Range) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub fn size(&self) -> f32 {
        self.end - self.start
    }

    /// Creates a range x, such that x.size() == self.size() && (other.contains(X) || x.contains(other)), minimizing the distance between self and x
    pub fn bounded_to(&self, other: &Range) -> Range {
        if self.start > other.start && self.end > other.end {
            let size_self_not_other = self.end - other.end;
            let size_other_not_self = self.start - other.start;
            if size_self_not_other < size_other_not_self {
                Range {
                    start: other.end - self.size(),
                    end: other.end,
                }
            } else {
                Range {
                    start: other.start,
                    end: other.start + self.size(),
                }
            }
        } else if self.start < other.start && self.end < other.end {
            let size_self_not_other = other.start - self.start;
            let size_other_not_self = other.end - self.end;
            if size_self_not_other < size_other_not_self {
                Range {
                    start: other.start,
                    end: other.start + self.size(),
                }
            } else {
                Range {
                    start: other.end - self.size(),
                    end: other.end,
                }
            }
        } else {
            self.clone()
        }
    }

    /// Creates a range x, such that x.size() == self.size() &&  x.intersect(other).size() is maximized while |x.start - self.start| is minimized
    pub fn maximize_overlap(&self, other: &Range) -> Range {
        if self.start < other.start {
            if self.end >= other.end {
                self.clone()
            } else {
                Range {
                    start: other.end - self.size(),
                    end: other.end,
                }
            }
        } else if self.end > other.end {
            Range {
                start: other.start,
                end: other.start + self.size(),
            }
        } else {
            self.clone()
        }
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end,)
    }
}
