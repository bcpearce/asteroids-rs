use std::rc::Rc;

use crate::{
    engine::{GameContext, GameElement},
    math::{Circle, Point, from_polar},
};
use itertools::Itertools;
use rand::RngExt;
use yew::{Html, html};

const MIN_ASTEROID_RADIUS: f32 = 7.0;
const MAX_ASTEROID_RADIUS: f32 = 15.0;
const MIN_ASTEROID_VELOCITY: f32 = 0.03;
const MAX_ASTEROID_VELOCITY: f32 = 0.11;
const SPLIT_ANGLE_RADS: f32 = 0.3;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Size {
    Small,
    Medium,
    Large,
    Destroyed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Asteroid {
    p: Point,
    v: Point,
    edge_points: Rc<Vec<Point>>,
    pub sz: Size,
}

impl Asteroid {
    pub fn spawn(w: f32, h: f32) -> Asteroid {
        let mut rng = rand::rng();
        let max_angle_rads = std::f32::consts::PI / 3.0; // 6 side ish
        let min_angle_rads = std::f32::consts::PI / 5.5; // 11 side ish
        let mut edge_points = Vec::new();
        let mut t = rng.random_range(min_angle_rads..max_angle_rads);
        let p = Point {
            x: rng.random_range(0.0..=w),
            y: rng.random_range(0.0..=h),
        };
        while t < std::f32::consts::PI * 2.0 {
            let r = rng.random_range(MIN_ASTEROID_RADIUS..=MAX_ASTEROID_RADIUS);
            edge_points.push(from_polar(r, t));
            t += rng.random_range(min_angle_rads..max_angle_rads);
        }
        let proto = rng.random_range(0..3);
        let sz = match proto {
            0 => Size::Large,
            1 => Size::Medium,
            2 => Size::Small,
            _ => Size::Destroyed,
        };
        Asteroid {
            p,
            v: from_polar(
                rng.random_range(MIN_ASTEROID_VELOCITY..=MAX_ASTEROID_VELOCITY),
                rng.random_range(0.0..=2.0 * std::f32::consts::PI),
            ),
            edge_points: Rc::new(edge_points),
            sz,
        }
    }

    fn scale(&self) -> f32 {
        match self.sz {
            Size::Large => 2.0,
            Size::Medium => 1.0,
            Size::Small => 0.55,
            Size::Destroyed => 0.0,
        }
    }

    pub fn score_from_size(sz: Size) -> u32 {
        match sz {
            Size::Large => 10,
            Size::Medium => 20,
            Size::Small => 50,
            Size::Destroyed => 0,
        }
    }

    pub fn score(&self) -> u32 {
        Self::score_from_size(self.sz)
    }

    pub fn split(&self) -> Option<[Self; 2]> {
        match self.sz {
            Size::Large => Some([
                Asteroid {
                    p: self.p,
                    v: self.v.rotate(SPLIT_ANGLE_RADS),
                    edge_points: self.edge_points.clone(),
                    sz: Size::Medium,
                },
                Asteroid {
                    p: self.p,
                    v: self.v.rotate(-SPLIT_ANGLE_RADS),
                    edge_points: self.edge_points.clone(),
                    sz: Size::Medium,
                },
            ]),
            Size::Medium => Some([
                Asteroid {
                    p: self.p,
                    v: self.v.rotate(SPLIT_ANGLE_RADS),
                    edge_points: self.edge_points.clone(),
                    sz: Size::Small,
                },
                Asteroid {
                    p: self.p,
                    v: self.v.rotate(-SPLIT_ANGLE_RADS),
                    edge_points: self.edge_points.clone(),
                    sz: Size::Small,
                },
            ]),
            Size::Small => None,
            Size::Destroyed => None,
        }
    }
}

impl GameElement for Asteroid {
    fn update(&mut self, ctx: &GameContext) {
        self.p += self.v * ctx.t;
        self.p.wrap(ctx.w, ctx.h);
    }

    fn alive(&self) -> bool {
        self.sz != Size::Destroyed
    }

    fn hitbox(&self) -> Circle {
        Circle {
            c: self.p,
            r: self.scale() * MAX_ASTEROID_RADIUS,
        }
    }

    fn render(&self) -> Html {
        let points = self
            .edge_points
            .iter()
            .map(|&p| p * self.scale() + self.p)
            .join(" ");
        html! {<polygon points={points} stroke="white" />}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::tests::PositiveFloat;
    use googletest::prelude::*;
    use is_svg::is_svg_string;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn it_spawns_an_asteroid_in_bounds(w: PositiveFloat, h: PositiveFloat) -> TestResult {
        let a = Asteroid::spawn(w.0, h.0);
        TestResult::from_bool(a.p.x <= w.0 && a.p.y <= h.0)
    }

    #[quickcheck]
    fn it_stays_in_bounds(
        w: PositiveFloat,
        h: PositiveFloat,
        t: PositiveFloat,
        iter_count: u32,
    ) -> Result<()> {
        let w = w.0;
        let h = h.0;
        let t = t.0 % 10_000.0;
        let mut a = Asteroid::spawn(w, h);
        let ctx = GameContext { w, h, t };
        let iter_count = iter_count % 5000; // limit to 5000 iterations
        for i in 0..iter_count {
            a.update(&ctx);
            let fail_msg = || format!("Failed on iteration {}", i);
            verify_that!(a.p.x, ge(0.0)).with_failure_message(fail_msg)?;
            verify_that!(a.p.y, ge(0.0)).with_failure_message(fail_msg)?;
            verify_that!(a.p.x, le(w)).with_failure_message(fail_msg)?;
            verify_that!(a.p.y, le(h)).with_failure_message(fail_msg)?;
        }
        Ok(())
    }

    #[quickcheck]
    fn it_is_a_polygon(w: PositiveFloat, h: PositiveFloat) -> TestResult {
        let a = Asteroid::spawn(w.0, h.0);
        TestResult::from_bool(a.edge_points.len() >= 3)
    }

    #[quickcheck]
    fn it_renders_valid_svg(w: PositiveFloat, h: PositiveFloat) -> TestResult {
        let a = Asteroid::spawn(w.0, h.0);
        let svg_wrap = format!("<svg>{:?}</svg>", a.render());
        TestResult::from_bool(is_svg_string(svg_wrap))
    }
}
