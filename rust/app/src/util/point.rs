use std::{
    fmt::Display,
    hash::Hash,
    ops::{Add, Mul, Sub},
};

#[derive(Copy, Clone, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Hash for Point {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ((self.x * 100.) as usize).hash(state);
        ((self.y * 100.) as usize).hash(state);
    }
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
impl<R: Clone> Mul<R> for &Point
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
impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
