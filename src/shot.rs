use crate::math::polar_point;
use crate::{
    engine::{GameContext, GameElement},
    math::Point,
};
use yew::{Html, html};

const SHOT_RADIUS: f32 = 1.0;
const BASE_SHOT_VELOCITY: f32 = 0.125;

#[derive(Debug, Copy, Clone)]
pub struct Shot {
    pub p: Point,
    v: Point,
    ttl: f32,
}
impl Shot {
    pub fn create(loc: Point, shooter_v: Point, theta_rad: f32) -> Self {
        let v = polar_point!(BASE_SHOT_VELOCITY, theta_rad) + shooter_v;
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

    fn render(&self) -> Html {
        html! { <circle cx={self.p.x.to_string()} cy={self.p.y.to_string()} r={SHOT_RADIUS.to_string()} stroke="white" fill="white"/> }
    }

    fn destroy(&mut self) {
        self.ttl = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{math::Point, shot::Shot};
    use googletest::prelude::*;
    use is_svg::is_svg_string;

    #[gtest]
    fn it_renders() {
        let shot = Shot::create(Point { x: 0.0, y: 0.0 }, Point { x: 0.0, y: 0.0 }, 0.0);
        let svg_wrap = format!("<svg>{:?}</svg>", shot.render());
        assert_that!(is_svg_string(svg_wrap), is_true())
    }
}
