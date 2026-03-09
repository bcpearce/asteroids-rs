use crate::engine::{GameContext, GameElement};
use crate::math::{Point, from_polar};
use crate::shot::Shot;
use rand::RngExt;
use yew::{Html, html};

pub struct Ship {
    p: Point,
    v: Point,
    omega_rad: f32,
    theta_rad: f32,
    sz: f32,
    w: f32,
    h: f32,
}
impl Ship {
    pub fn create(w: f32, h: f32) -> Ship {
        Ship {
            p: Point {
                x: w / 2.0,
                y: h / 2.0,
            },
            v: Point { x: 0.0, y: 0.0 },
            omega_rad: 0.0,
            theta_rad: std::f32::consts::PI * 0.25,
            sz: 10.0,
            w,
            h,
        }
    }

    pub fn thrust(&mut self) {
        let dv = from_polar(0.01, self.theta_rad);
        self.v.x += dv.x;
        self.v.y += dv.y;
    }

    pub fn rotate_left(&mut self) {
        self.omega_rad = -std::f32::consts::PI / 180.0;
    }

    pub fn rotate_right(&mut self) {
        self.omega_rad = std::f32::consts::PI / 180.0;
    }

    pub fn stop_rotate(&mut self) {
        self.omega_rad = 0.0;
    }

    pub fn shoot(&self) -> Shot {
        Shot::create(self.p, self.theta_rad)
    }

    pub fn hyperspace(&mut self) {
        let mut rng = rand::rng();
        self.p.x = rng.random_range(0.0..=self.w);
        self.p.y = rng.random_range(0.0..=self.h);
        self.theta_rad = rng.random_range(0.0..=2.0 * std::f32::consts::PI);
        self.v = Point { x: 0.0, y: 0.0 };
    }
}

impl GameElement for Ship {
    fn update(&mut self, ctx: &GameContext) {
        self.theta_rad += self.omega_rad * ctx.t;
        self.p += self.v * ctx.t;
        self.p.wrap(ctx.w, ctx.h);
    }

    fn alive(&self) -> bool {
        true
    }

    fn render(&self) -> Html {
        let p1 = from_polar(self.sz, self.theta_rad) + self.p;
        let p2 = from_polar(self.sz * 0.6, self.theta_rad + 0.75 * std::f32::consts::PI) + self.p;
        let p3 = from_polar(self.sz * 0.6, self.theta_rad - 0.75 * std::f32::consts::PI) + self.p;
        let points = format!("{} {} {}", p1, p2, p3);

        html! { <polygon points={points} stroke="white" /> }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::tests::PositiveFloat;
    use googletest::prelude::*;
    use is_svg::is_svg_string;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn it_renders_valid_svg(w: PositiveFloat, h: PositiveFloat) -> TestResult {
        let s = Ship::create(w.0, h.0);
        let svg_wrap = format!("<svg>{:?}</svg>", s.render());
        TestResult::from_bool(is_svg_string(svg_wrap))
    }

    #[derive(Clone, Debug)]
    enum ShipCommands {
        Thrust,
        RotateLeft,
        RotateRight,
        Hyperspace,
    }

    impl Arbitrary for ShipCommands {
        fn arbitrary(g: &mut Gen) -> Self {
            let i = u32::arbitrary(g) % 4;
            match i {
                0 => ShipCommands::Thrust,
                1 => ShipCommands::RotateLeft,
                2 => ShipCommands::RotateRight,
                3 => ShipCommands::Hyperspace,
                _ => panic!("Unreachable"),
            }
        }
    }

    #[quickcheck]
    fn it_stays_in_bounds(
        w: PositiveFloat,
        h: PositiveFloat,
        t: PositiveFloat,
        cmds: Vec<ShipCommands>,
    ) -> Result<()> {
        let w = w.0;
        let h = h.0;
        let t = t.0 % 10_000.0;
        let mut ship = Ship::create(w, h);
        let ctx = GameContext { w, h, t };
        ship.update(&ctx);
        for (i, cmd) in cmds.iter().enumerate() {
            match cmd {
                ShipCommands::Thrust => ship.thrust(),
                ShipCommands::RotateLeft => ship.rotate_left(),
                ShipCommands::RotateRight => ship.rotate_right(),
                ShipCommands::Hyperspace => ship.hyperspace(),
            }
            ship.update(&ctx);
            let fail_msg = || format!("Failed on command {}: {:?}", i, cmd);
            verify_that!(ship.p.x, ge(0.0)).with_failure_message(fail_msg)?;
            verify_that!(ship.p.y, ge(0.0)).with_failure_message(fail_msg)?;
            verify_that!(ship.p.x, le(w)).with_failure_message(fail_msg)?;
            verify_that!(ship.p.y, le(h)).with_failure_message(fail_msg)?;
        }
        Ok(())
    }
}
