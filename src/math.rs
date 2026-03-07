use core::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use approx::assert_relative_eq;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub fn from_polar(r: f32, theta: f32) -> Point {
    let (sin, cos) = theta.sin_cos();
    return Point {
        x: r * cos,
        y: r * sin,
    };
}

impl Point {
    pub fn wrap(&mut self, w: f32, h: f32) {
        self.x = if self.x < 0.0 {
            w - self.x
        } else if self.x > w {
            self.x - w
        } else {
            self.x
        };
        self.y = if self.y < 0.0 {
            h - self.y
        } else if self.y > h {
            self.y - h
        } else {
            self.y
        };
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul<f32> for Point {
    type Output = Self;

    fn mul(self, mag: f32) -> Self::Output {
        Self {
            x: self.x * mag,
            y: self.y * mag,
        }
    }
}

impl Mul<Point> for f32 {
    type Output = Point;
    fn mul(self, point: Point) -> Self::Output {
        Point {
            x: point.x * self,
            y: point.y * self,
        }
    }
}

impl MulAssign<f32> for Point {
    fn mul_assign(&mut self, mag: f32) {
        self.x *= mag;
        self.y *= mag;
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[test]
fn it_adds() {
    let p1 = Point { x: 1.0, y: 5.0 };
    let p2 = Point { x: 2.5, y: 4.2 };
    let p3 = p1 + p2;
    assert_relative_eq!(p3.x, 3.5);
    assert_relative_eq!(p3.y, 9.2);
}

#[test]
fn it_add_assigns() {
    let mut p1 = Point { x: 1.0, y: 5.0 };
    let p2 = Point { x: 2.5, y: 4.2 };
    p1 += p2;
    assert_relative_eq!(p1.x, 3.5);
    assert_relative_eq!(p1.y, 9.2);
}

#[test]
fn it_muls() {
    let p1 = Point { x: 1.0, y: 5.0 } * 3.0;
    assert_relative_eq!(p1.x, 3.0);
    assert_relative_eq!(p1.y, 15.0);
}

#[test]
fn it_mul_assigns() {
    let mut p1 = Point { x: 1.0, y: 5.0 };
    p1 *= 3.0;
    assert_relative_eq!(p1.x, 3.0);
    assert_relative_eq!(p1.y, 15.0);
}

#[test]
fn it_converts_from_polar() {
    let p1 = from_polar(2.0, std::f32::consts::PI * 0.5);
    assert_relative_eq!(p1.x, 0.0);
    assert_relative_eq!(p1.y, 2.0);
    let p1 = from_polar(2.0, std::f32::consts::PI * -0.25);
    assert_relative_eq!(p1.x, 1.41, epsilon = 0.01);
    assert_relative_eq!(p1.y, -1.41, epsilon = 0.01);
}
