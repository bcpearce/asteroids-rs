use crate::{
    engine::{GameContext, GameElement},
    math::{Circle, Point, from_polar},
};
use yew::{Html, html};

const SHOT_RADIUS: f32 = 1.0;
const BASE_SHOT_VELOCITY: f32 = 0.125;

#[derive(Debug, Copy, Clone)]
pub struct Shot {
    p: Point,
    v: Point,
    ttl: f32,
}
impl Shot {
    pub fn create(loc: Point, shooter_v: Point, theta_rad: f32) -> Self {
        let v = from_polar(BASE_SHOT_VELOCITY, theta_rad) + shooter_v;
        Shot {
            p: loc,
            v,
            ttl: 1400.0,
        }
    }
}
impl GameElement for Shot {
    fn update(&mut self, ctx: &GameContext) {
        self.p += self.v * ctx.t;
        self.ttl -= ctx.t;
    }

    fn alive(&self) -> bool {
        self.ttl > 0.0
    }

    fn hitbox(&self) -> Circle {
        Circle {
            c: self.p,
            r: SHOT_RADIUS,
        }
    }

    fn render(&self) -> Html {
        html! { <circle cx={self.p.x.to_string()} cy={self.p.y.to_string()} r={SHOT_RADIUS.to_string()} stroke="white" fill="white"/> }
    }
}
