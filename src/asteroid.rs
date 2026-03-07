use crate::{
    engine::GameContext,
    math::{Point, from_polar},
};
use itertools::Itertools;
use rand::RngExt;
use yew::{Html, html};

const MIN_ASTEROID_RADIUS: f32 = 3.0;
const MAX_ASTEROID_RADIUS: f32 = 15.0;
const MIN_ASTEROID_VELOCITY: f32 = 0.03;
const MAX_ASTEROID_VELOCITY: f32 = 0.2;
enum Size {
    Small,
    Medium,
    Large,
}
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
            _ => Size::Small,
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

    pub fn update(&mut self, ctx: &GameContext) {
        self.p += self.v * ctx.t;
        self.p.wrap(ctx.w as f32, ctx.h as f32);
    }

    pub fn render(&self) -> Html {
        let scale = match self.sz {
            Size::Large => 2.0,
            Size::Medium => 1.0,
            Size::Small => 0.55,
        };
        let points = self
            .edge_points
            .iter()
            .map(|&p| return p * scale + self.p)
            .join(" ");
        html! {<polygon points={points} stroke="white" />}
    }
}
