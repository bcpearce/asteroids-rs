use crate::engine::{GameContext, GameElement};
use crate::math::Point;
use yew::Html;
use yew::html;

pub struct Debris {
    pub p: Point,
    pub v: Point,
    pub hue: u32,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::point;
    use googletest::prelude::*;
    use is_svg::is_svg_string;

    #[test]
    fn it_renders_valid_svg() {
        let p = point!(1.0, 5.0);
        let v = point!(1.0, 1.0);
        let debris = Debris { p, v, hue: 0 };
        let svg_wrap = format!("<svg>{:?}</svg>", debris.render());
        assert_that!(is_svg_string(svg_wrap), is_true())
    }
}
