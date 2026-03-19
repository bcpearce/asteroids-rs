use crate::engine::{GameContext, GameElement};
use crate::math::Point;
use yew::Html;
use yew::html;

pub struct Debris {
    pub p: Point,
    pub v: Point,
    pub hue: f32,
}

impl GameElement for Debris {
    fn update(&mut self, ctx: &GameContext) {
        self.p += self.v * ctx.t;
        self.p.wrap(ctx.w, ctx.h);
    }

    fn alive(&self) -> bool {
        true
    }

    fn render(&self) -> Html {
        let hsl = format!("hsl({}, 100%, 50%", self.hue);
        html! {<circle
        cx={self.p.x.to_string()}
        cy={self.p.y.to_string()}
        stroke={hsl}
        r="0.1" />}
    }

    fn destroy(&mut self) {}
}

pub struct LineDebris {
    pub p1: Point,
    pub p2: Point,
    pub v: Point,
    pub w: f32,
}

impl GameElement for LineDebris {
    fn update(&mut self, ctx: &GameContext) {
        self.p1 += self.v * ctx.t;
        self.p2 += self.v * ctx.t;
        let mut midpoint = Point::midpoint(self.p1, self.p2);
        let dp1 = midpoint - self.p1;
        let dp2 = midpoint - self.p2;
        midpoint.wrap(ctx.w, ctx.h);
        self.p1 = midpoint + dp1;
        self.p2 = midpoint + dp2;
        self.p1 = self.p1.rotate_about(self.w * ctx.t, midpoint);
        self.p2 = self.p2.rotate_about(self.w * ctx.t, midpoint);
    }

    fn alive(&self) -> bool {
        true
    }

    fn render(&self) -> Html {
        html! {<line
        x1={self.p1.x.to_string()}
        y1={self.p1.y.to_string()}
        x2={self.p2.x.to_string()}
        y2={self.p2.y.to_string()}
        stroke="white" />}
    }

    fn destroy(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::point;
    use googletest::prelude::*;
    use is_svg::is_svg_string;

    #[test]
    fn it_renders_valid_svg_for_debris() {
        let p = point!(1.0, 5.0);
        let v = point!(1.0, 1.0);
        let debris = Debris { p, v, hue: 0.0 };
        let svg_wrap = format!("<svg>{:?}</svg>", debris.render());
        assert_that!(is_svg_string(svg_wrap), is_true())
    }

    #[test]
    fn it_renders_valid_svg_for_line_debris() {
        let p1 = point!(1.0, 5.0);
        let p2 = point!(5.0, 5.0);
        let v = point!(0.0, 0.0);
        let w = 0.0;
        let line_debris = LineDebris { p1, p2, v, w };
        let svg_wrap = format!("<svg>{:?}</svg>", line_debris.render());
        assert_that!(is_svg_string(svg_wrap), is_true())
    }

    #[test]
    fn it_debris_is_always_alive() {
        let p = point!(1.0, 5.0);
        let v = point!(1.0, 1.0);
        let mut debris = Debris { p, v, hue: 0.0 };
        assert!(debris.alive());
        debris.destroy();
        assert!(debris.alive());
    }
    #[test]
    fn it_line_debris_is_always_alive() {
        let p1 = point!(1.0, 5.0);
        let p2 = point!(5.0, 5.0);
        let v = point!(0.0, 0.0);
        let w = 0.0;
        let mut line_debris = LineDebris { p1, p2, v, w };
        assert!(line_debris.alive());
        line_debris.destroy();
        assert!(line_debris.alive());
    }
}
