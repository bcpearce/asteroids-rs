use core::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

macro_rules! point {
    ($x:expr, $y:expr) => {
        Point {
            x: $x as f32,
            y: $y as f32,
        }
    };
}
pub(crate) use point;

macro_rules! polar_point {
    ($r:expr, $theta:expr) => {
        Point::from_polar($r as f32, $theta as f32)
    };
}
pub(crate) use polar_point;

impl Point {
    pub fn from_polar(r: f32, theta: f32) -> Point {
        let (sin, cos) = theta.sin_cos();
        Point {
            x: r * cos,
            y: r * sin,
        }
    }

    pub fn cross(p1: Point, p2: Point) -> f32 {
        p1.x * p2.y - p2.x * p1.y
    }

    pub fn dot(p1: Point, p2: Point) -> f32 {
        p1.x * p2.x + p1.y * p2.y
    }

    pub fn midpoint(p1: Point, p2: Point) -> Point {
        (p1 + p2) * 0.5
    }

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

    pub fn orthogonal(&self) -> Point {
        point!(-self.y, self.x)
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

    pub fn rotate_about(&self, theta_rad: f32, about: Point) -> Point {
        let tmp = *self - about;
        let tmp = tmp.rotate(theta_rad);
        tmp + about
    }

    /// Checks if it exists inside a polygon using ray-casting
    pub fn in_polygon(&self, polygon: &[Point]) -> Result<bool, &'static str> {
        if polygon.len() < 3 {
            return Err("Polygon must have at least 3 points");
        }
        fn check_ray_intersection(origin: Point, p1: Point, p2: Point) -> bool {
            let ortho = point!(0, 1);
            let p1_to_origin = origin - p1;
            let p1_to_p2 = p2 - p1;
            let den: f32 = Point::dot(p1_to_p2, ortho);
            if den == 0.0 {
                return origin.x == p1.x || origin.x == p2.x;
            }

            let t1 = Point::cross(p1_to_p2, p1_to_origin) / den;
            let t2 = Point::dot(p1_to_origin, ortho) / den;

            (0.0..=1.0).contains(&t2) && t1 >= 0.0
        }
        let mut intersections = 0;
        for (i, j) in (0..polygon.len()).zip(1..(polygon.len() + 1)) {
            let &p1 = &polygon[i];
            let &p2 = &polygon[j % polygon.len()];
            if check_ray_intersection(*self, p1, p2) {
                intersections += 1;
            }
        }
        Ok(intersections % 2 == 1)
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

impl Mul<Point> for Point {
    type Output = f32;
    fn mul(self, point: Point) -> Self::Output {
        Self::dot(self, point)
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

#[cfg(test)]
mod point_tests {
    use core::f32;

    use super::*;
    use googletest::prelude::*;
    use p_test::p_test;
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
    fn it_subs() {
        let p1 = Point { x: 1.0, y: 5.0 };
        let p2 = Point { x: 2.5, y: 4.2 };
        let p3 = p1 - p2;
        expect_that!(p3.x, near(-1.5, 1e-6));
        expect_that!(p3.y, near(0.8, 1e-6));
    }

    #[gtest]
    fn it_sub_assigns() {
        let mut p1 = Point { x: 1.0, y: 5.0 };
        let p2 = Point { x: 2.5, y: 4.2 };
        p1 -= p2;
        expect_that!(p1.x, near(-1.5, 1e-6));
        expect_that!(p1.y, near(0.8, 1e-6));
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
        let p1 = Point::from_polar(2.0, std::f32::consts::PI * 0.5);
        expect_that!(p1.x, near(0.0, 1e-6));
        expect_that!(p1.y, near(2.0, 1e-6));
        let p1 = Point::from_polar(2.0, std::f32::consts::PI * -0.25);
        expect_that!(p1.x, near(1.41421356237, 1e-6));
        expect_that!(p1.y, near(-1.41421356237, 1e-6));
    }

    #[gtest]
    fn it_does_equality() {
        let p1 = point!(1, 1);
        let p2 = point!(1, 1);
        assert_that!(p1, eq(p2));
    }

    #[p_test(
        "at origin", (point!(0.0, 0.0), true),
        "outside at(n2.0,0.0)", (point!(-2.0, 0.0), false), 
        "outside at(2.0,0.0)", (point!(2.0, 0.0), false), 
        "inside at(0.5,0.5)", (point!(0.5, 0.5), true),
        "on edge(1.0,0.0)", (point!(1.0, 0.0), true),
        "on edge(1.0,n1.0)", (point!(1.0, -1.0), true),
        "outside at(0.0,2.0", (point!(0.0, 2.0), false),
        "outside at(NaN,0.0)", (point!(f32::NAN, 0.0), false),
    )]
    fn it_determines_point_in_polygon(point: Point, expect_inside: bool) {
        let polygon = vec![point!(-1, -1), point!(-1, 1), point!(1, 1), point!(1, -1)];
        assert_that!(point.in_polygon(&polygon).unwrap(), eq(expect_inside));
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
