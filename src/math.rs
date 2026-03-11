use core::fmt;
use std::ops::{Add, AddAssign, BitOr, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub fn from_polar(r: f32, theta: f32) -> Point {
    let (sin, cos) = theta.sin_cos();
    Point {
        x: r * cos,
        y: r * sin,
    }
}

impl Point {
    pub fn wrap(&mut self, w: f32, h: f32) {
        self.x = if self.x < 0.0 {
            w - (self.x % w).abs()
        } else if self.x > w {
            self.x % w
        } else {
            self.x
        };
        self.y = if self.y < 0.0 {
            h - (self.y % h).abs()
        } else if self.y > h {
            self.y % h
        } else {
            self.y
        };
    }

    pub fn mag(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn rotate(&self, theta_rad: f32) -> Point {
        if theta_rad.is_normal() {
            let (sin, cos) = theta_rad.sin_cos();
            Point {
                x: self.x * cos - self.y * sin,
                y: self.x * sin + self.y * cos,
            }
        } else {
            *self
        }
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

pub struct Circle {
    pub c: Point,
    pub r: f32,
}

impl BitOr for Circle {
    type Output = bool;
    fn bitor(self, other: Self) -> bool {
        let dist = (self.c - other.c).mag();
        dist < self.r + other.r
    }
}

#[cfg(test)]
mod point_tests {
    use super::*;
    use googletest::prelude::*;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use quickcheck_macros::quickcheck;

    #[gtest]
    fn it_adds() {
        let p1 = Point { x: 1.0, y: 5.0 };
        let p2 = Point { x: 2.5, y: 4.2 };
        let p3 = p1 + p2;
        expect_that!(p3.x, near(3.5, 1e-6));
        expect_that!(p3.y, near(9.2, 1e-6));
    }

    #[gtest]
    fn it_add_assigns() {
        let mut p1 = Point { x: 1.0, y: 5.0 };
        let p2 = Point { x: 2.5, y: 4.2 };
        p1 += p2;
        expect_that!(p1.x, near(3.5, 1e-6));
        expect_that!(p1.y, near(9.2, 1e-6));
    }

    #[gtest]
    fn it_muls() {
        let p1 = Point { x: 1.0, y: 5.0 } * 3.0;
        expect_that!(p1.x, near(3.0, 1e-6));
        expect_that!(p1.y, near(15.0, 1e-6));
    }

    #[gtest]
    fn it_mul_assigns() {
        let mut p1 = Point { x: 1.0, y: 5.0 };
        p1 *= 3.0;
        expect_that!(p1.x, near(3.0, 1e-6));
        expect_that!(p1.y, near(15.0, 1e-6));
    }

    #[gtest]
    fn it_converts_from_polar() {
        let p1 = from_polar(2.0, std::f32::consts::PI * 0.5);
        expect_that!(p1.x, near(0.0, 1e-6));
        expect_that!(p1.y, near(2.0, 1e-6));
        let p1 = from_polar(2.0, std::f32::consts::PI * -0.25);
        expect_that!(p1.x, near(1.41421356237, 1e-6));
        expect_that!(p1.y, near(-1.41421356237, 1e-6));
    }

    impl Arbitrary for Point {
        fn arbitrary(g: &mut Gen) -> Self {
            Point {
                x: f32::arbitrary(g),
                y: f32::arbitrary(g),
            }
        }
    }

    #[quickcheck]
    fn it_keeps_the_same_magnitude_on_rotation(p: Point, theta_rad: f32) -> TestResult {
        if p.x.is_finite()
            && p.y.is_finite()
            && theta_rad.is_finite()
            && p.x.abs() < 1e18
            && p.y.abs() < 1e18
        {
            let actual = p.mag();
            let expected = p.rotate(theta_rad).mag();
            if actual - expected < 1e-6 {
                return TestResult::passed();
            }
            let relative_error = ((actual - expected) / expected).abs();
            assert_that!(relative_error, lt(1e-6));
            TestResult::passed()
        } else {
            TestResult::discard()
        }
    }
}

#[cfg(test)]
mod circle_tests {
    use super::*;
    use googletest::prelude::*;

    #[gtest]
    fn it_finds_intersecting_circles() {
        let c1 = Circle {
            c: Point { x: 0.0, y: 0.0 },
            r: 3.0,
        };
        let c2 = Circle {
            c: Point { x: 1.0, y: 1.0 },
            r: 0.5,
        };
        expect_true!(c1 | c2);
    }
}
