use crate::{
    engine::GameContext,
    engine::GameElement,
    math::{Point, from_polar},
};
use itertools::Itertools;
use rand::RngExt;
use yew::{Html, html};

const MIN_ASTEROID_RADIUS: f32 = 3.0;
const MAX_ASTEROID_RADIUS: f32 = 15.0;
const MIN_ASTEROID_VELOCITY: f32 = 0.03;
const MAX_ASTEROID_VELOCITY: f32 = 0.2;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Size {
    Small,
    Medium,
    Large,
    Destroyed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Asteroid {
    p: Point,
    v: Point,
    edge_points: Vec<Point>,
    sz: Size,
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
            p: p,
            v: from_polar(
                rng.random_range(MIN_ASTEROID_VELOCITY..=MAX_ASTEROID_VELOCITY),
                rng.random_range(0.0..=2.0 * std::f32::consts::PI),
            ),
            edge_points: edge_points,
            sz: sz,
        }
    }
}

impl GameElement for Asteroid {
    fn update(&mut self, ctx: &GameContext) {
        self.p += self.v * ctx.t;
        self.p.wrap(ctx.w as f32, ctx.h as f32);
    }

    fn alive(&self) -> bool {
        self.sz != Size::Destroyed
    }

    fn render(&self) -> Html {
        let scale = match self.sz {
            Size::Large => 2.0,
            Size::Medium => 1.0,
            Size::Small => 0.55,
            Size::Destroyed => 0.0,
        };
        let points = self
            .edge_points
            .iter()
            .map(|&p| return p * scale + self.p)
            .join(" ");
        html! {<polygon points={points} stroke="white" />}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use is_svg::is_svg_string;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use quickcheck_macros::quickcheck;

    #[derive(Clone, Debug)]
    struct PositiveFloat(f32);

    impl Arbitrary for PositiveFloat {
        fn arbitrary(g: &mut Gen) -> Self {
            let f = f32::arbitrary(g);
            if !f.is_finite() {
                PositiveFloat::arbitrary(g)
            } else {
                PositiveFloat(f.abs())
            }
        }
    }

    #[quickcheck]
    fn it_spawns_an_asteroid_in_bounds(w: PositiveFloat, h: PositiveFloat) -> TestResult {
        let a = Asteroid::spawn(w.0, h.0);
        TestResult::from_bool(a.p.x <= w.0 && a.p.y <= h.0)
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
