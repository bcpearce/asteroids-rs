use crate::{
    engine::GameContext,
    math::{Point, from_polar},
};
use yew::{Html, html};
pub struct Shot {
    p: Point,
    v: Point,
    ttl: f32,
}
impl Shot {
    pub fn create(loc: Point, theta_rad: f32) -> Self {
        let v = from_polar(0.1, theta_rad);
        Shot {
            p: loc,
            v: v,
            ttl: 1400.0,
        }
    }
    pub fn update(&mut self, ctx: &GameContext) {
        self.p += self.v * ctx.t;
        self.ttl -= ctx.t;
    }
    pub fn alive(&self) -> bool {
        self.ttl > 0.0
    }
    pub fn render(&self) -> Html {
        html! { <circle cx={self.p.x.to_string()} cy={self.p.y.to_string()} r="1" stroke="white" fill="white"/> }
    }
}
