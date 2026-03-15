use std::rc::Rc;
use strum_macros::EnumIter;

use crate::{
    common,
    debris::Debris,
    engine::{GameContext, GameElement},
    math::Point,
};
use itertools::Itertools;
use rand::RngExt;
use yew::{Html, html};

const MIN_ASTEROID_RADIUS: f32 = 2.0;
const MAX_ASTEROID_RADIUS: f32 = 15.0;
const MIN_ASTEROID_VELOCITY: f32 = 0.03;
const MAX_ASTEROID_VELOCITY: f32 = 0.11;
const SPLIT_ANGLE_RADS: f32 = 0.3;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, EnumIter)]
pub enum Size {
    Small,
    Medium,
    Large,
    Destroyed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Asteroid {
    pub p: Point,
    pub v: Point,
    edge_points: Rc<Vec<Point>>,
    pub sz: Size,
    pub hue: u32,
}

impl Asteroid {
    #[cfg(test)]
    pub fn create(p: Point, v: Point, edge_points: Vec<Point>, sz: Size) -> Asteroid {
        Asteroid {
            p,
            v,
            edge_points: Rc::from(edge_points),
            sz,
            hue: 0,
        }
    }

    pub fn spawn(w: f32, h: f32, maybe_seed: Option<u64>) -> Asteroid {
        let mut rng = common::rng::get_rng(maybe_seed);
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
            edge_points.push(Point::from_polar(r, t));
            t += rng.random_range(min_angle_rads..max_angle_rads);
        }
        let proto = rng.random_range(0..3);
        let sz = match proto {
            0 => Size::Large,
            1 => Size::Medium,
            2 => Size::Small,
            _ => Size::Destroyed,
        };
        let hue = rng.random_range(0..360);
        Asteroid {
            p,
            v: Point::from_polar(
                rng.random_range(MIN_ASTEROID_VELOCITY..=MAX_ASTEROID_VELOCITY),
                rng.random_range(0.0..=2.0 * std::f32::consts::PI),
            ),
            edge_points: Rc::new(edge_points),
            sz,
            hue,
        }
    }

    pub fn scale(&self) -> f32 {
        match self.sz {
            Size::Large => 2.0,
            Size::Medium => 1.0,
            Size::Small => 0.55,
            Size::Destroyed => 0.0,
        }
    }

    pub fn score_from_size(sz: &Size) -> i32 {
        match sz {
            Size::Large => 10,
            Size::Medium => 20,
            Size::Small => 50,
            Size::Destroyed => 0,
        }
    }

    pub fn score(&self) -> i32 {
        Self::score_from_size(&self.sz)
    }

    pub fn split(&self) -> Option<[Self; 2]> {
        fn helper(a: &Asteroid, rotation: f32, new_size: Size) -> Asteroid {
            Asteroid {
                p: a.p,
                v: a.v.rotate(rotation),
                edge_points: a.edge_points.clone(),
                sz: new_size,
                hue: a.hue,
            }
        }
        match self.sz {
            Size::Large => Some([
                helper(self, SPLIT_ANGLE_RADS, Size::Medium),
                helper(self, -SPLIT_ANGLE_RADS, Size::Medium),
            ]),
            Size::Medium => Some([
                helper(self, SPLIT_ANGLE_RADS, Size::Small),
                helper(self, -SPLIT_ANGLE_RADS, Size::Small),
            ]),
            Size::Small => Some([
                helper(self, SPLIT_ANGLE_RADS, Size::Destroyed),
                helper(self, -SPLIT_ANGLE_RADS, Size::Destroyed),
            ]),
            Size::Destroyed => None,
        }
    }

    pub fn polygon(&self) -> Vec<Point> {
        self.edge_points
            .iter()
            .map(|&p| p * self.scale() + self.p)
            .collect()
    }

    pub fn make_debris(&self) -> Debris {
        Debris {
            p: self.p,
            v: self.v,
            hue: self.hue,
        }
    }
}

impl GameElement for Asteroid {
    fn update(&mut self, ctx: &GameContext) {
        self.p += self.v * ctx.t;
        self.p.wrap(ctx.w, ctx.h);
    }

    fn alive(&self) -> bool {
        !matches!(self.sz, Size::Destroyed)
    }

    fn render(&self) -> Html {
        let hsl = format!("hsl({}, 100%, 50%", self.hue);
        match self.sz {
            Size::Destroyed => {
                html! {<circle cx={self.p.x.to_string()} cy={self.p.y.to_string()} r="0.1" stroke={hsl}/>}
            }
            _ => {
                let points = self.polygon().into_iter().join(" ");
                html! {<polygon points={points} stroke={hsl}/>}
            }
        }
    }

    fn destroy(&mut self) {
        self.sz = Size::Destroyed;
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
    fn it_spawns_an_asteroid_in_bounds(
        w: PositiveFloat,
        h: PositiveFloat,
        seed: u64,
    ) -> TestResult {
        let a = Asteroid::spawn(w.0, h.0, Some(seed));
        TestResult::from_bool(a.p.x <= w.0 && a.p.y <= h.0)
    }

    #[quickcheck]
    fn it_stays_in_bounds(
        w: PositiveFloat,
        h: PositiveFloat,
        t: PositiveFloat,
        iter_count: u32,
        seed: u64,
    ) -> Result<()> {
        let w = w.0;
        let h = h.0;
        let t = t.0 % 10_000.0;
        let mut a = Asteroid::spawn(w, h, Some(seed));
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
    fn it_is_a_polygon(w: PositiveFloat, h: PositiveFloat, seed: u64) -> TestResult {
        let a = Asteroid::spawn(w.0, h.0, Some(seed));
        TestResult::from_bool(a.edge_points.len() >= 3)
    }

    #[quickcheck]
    fn it_renders_valid_svg(w: PositiveFloat, h: PositiveFloat, seed: u64) -> TestResult {
        let a = Asteroid::spawn(w.0, h.0, Some(seed));
        let svg_wrap = format!("<svg>{:?}</svg>", a.render());
        TestResult::from_bool(is_svg_string(svg_wrap))
    }
}
