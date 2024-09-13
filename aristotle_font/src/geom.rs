use std::ops::{Add, Mul};

#[derive(Clone, Copy, Default, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<Point> for ab_glyph_rasterizer::Point {
    fn from(value: Point) -> Self {
        ab_glyph_rasterizer::Point {
            x: value.x,
            y: value.y,
        }
    }
}
impl Add<Point> for Point {
    type Output = Self;
    fn add(self, rhs: Point) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Mul<f32> for Point {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}
impl Mul<f32> for Rect {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            min: self.min * rhs,
            max: self.max * rhs,
        }
    }
}
