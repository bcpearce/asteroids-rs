use crate::common::rng::get_rng;
use crate::engine::{GameContext, GameElement};
use crate::math::{Circle, Point, from_polar};
use crate::shot::Shot;
use rand::RngExt;
use yew::{Html, html};

const BASE_SHOT_COOLDOWN_MS: f32 = 150.0;
const THRUST_FACTOR: f32 = 3e-3;

pub struct Ship {
    p: Point,
    v: Point,
    omega_rad: f32,
    theta_rad: f32,
    sz: f32,
    w: f32,
    h: f32,
    shot_cooldown: f32,
    maybe_seed: Option<u64>,
    is_destroyed: bool,
}
impl Ship {
    pub fn create(w: f32, h: f32, maybe_seed: Option<u64>) -> Ship {
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
            shot_cooldown: 0.0,
            maybe_seed,
            is_destroyed: false,
        }
    }

    pub fn thrust(&mut self) {
        let dv = from_polar(THRUST_FACTOR, self.theta_rad);
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

    pub fn shoot(&mut self) -> Option<Shot> {
        if self.shot_cooldown > 0.0 || self.is_destroyed {
            None
        } else {
            self.shot_cooldown = BASE_SHOT_COOLDOWN_MS;
            Some(Shot::create(self.p, self.v, self.theta_rad))
        }
    }

    pub fn hyperspace(&mut self) {
        let mut rng = get_rng(self.maybe_seed);
        self.p.x = rng.random_range(0.0..=self.w);
        self.p.y = rng.random_range(0.0..=self.h);
        self.theta_rad = rng.random_range(0.0..=2.0 * std::f32::consts::PI);
        self.v = Point { x: 0.0, y: 0.0 };
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true
    }
}

impl GameElement for Ship {
    fn update(&mut self, ctx: &GameContext) {
        self.theta_rad += self.omega_rad * ctx.t;
        self.p += self.v * ctx.t;
        self.p.wrap(ctx.w, ctx.h);
        self.shot_cooldown -= ctx.t;
    }

    fn alive(&self) -> bool {
        self.is_destroyed
    }

    fn hitbox(&self) -> Circle {
        Circle {
            c: self.p,
            r: self.sz,
        }
    }

    fn render(&self) -> Html {
        let p1 = from_polar(self.sz, self.theta_rad) + self.p;
        let p2 = from_polar(self.sz * 0.6, self.theta_rad + 0.75 * std::f32::consts::PI) + self.p;
        let p3 = from_polar(self.sz * 0.6, self.theta_rad - 0.75 * std::f32::consts::PI) + self.p;
        let points = format!("{} {} {}", p1, p2, p3);

        if self.is_destroyed {
            html! { <text x={(self.w / 2.0).to_string()} y={(self.h / 2.0).to_string()}
            text-anchor="middle"
            dominant-baseline="middle"
            fill="#FF0000"
            font-size="20"
            font-family="monospace">
                {"Game Over"}
            </text> }
        } else {
            html! { <polygon points={points} stroke="white" /> }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::common::tests::{PositiveFloat, ShipCommand};
    use googletest::prelude::*;
    use is_svg::is_svg_string;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn it_renders_valid_svg(w: PositiveFloat, h: PositiveFloat, seed: u64) -> TestResult {
        let s = Ship::create(w.0, h.0, Some(seed));
        let svg_wrap = format!("<svg>{:?}</svg>", s.render());
        TestResult::from_bool(is_svg_string(svg_wrap))
    }

    #[quickcheck]
    fn it_stays_in_bounds(
        w: PositiveFloat,
        h: PositiveFloat,
        t: PositiveFloat,
        cmds: Vec<ShipCommand>,
        seed: u64,
    ) -> Result<()> {
        let w = w.0;
        let h = h.0;
        let t = t.0 % 10_000.0;
        let mut ship = Ship::create(w, h, Some(seed));
        let ctx = GameContext { w, h, t };
        ship.update(&ctx);
        for (i, cmd) in cmds.iter().enumerate() {
            match cmd {
                ShipCommand::Thrust => ship.thrust(),
                ShipCommand::RotateLeft => ship.rotate_left(),
                ShipCommand::RotateRight => ship.rotate_right(),
                ShipCommand::Hyperspace => ship.hyperspace(),
                ShipCommand::Shoot => {
                    ship.shoot();
                    ()
                }
                ShipCommand::NoOp => (),
            };
            ship.update(&ctx);
            let fail_msg = || format!("Failed on command {}: {:?}", i, cmd);
            verify_that!(ship.p.x, ge(0.0)).with_failure_message(fail_msg)?;
            verify_that!(ship.p.y, ge(0.0)).with_failure_message(fail_msg)?;
            verify_that!(ship.p.x, le(w)).with_failure_message(fail_msg)?;
            verify_that!(ship.p.y, le(h)).with_failure_message(fail_msg)?;
        }
        Ok(())
    }

    #[gtest]
    fn it_cannot_shoot_if_destroyed() {
        let w = 500.0;
        let h = 500.0;
        let mut ship = Ship::create(w, h, Some(0));
        ship.destroy();
        assert_that!(ship.shoot(), none())
    }

    #[gtest]
    fn it_creates_a_shot_if_the_cooldown_elapsed() {
        let w = 500.0;
        let h = 500.0;
        let t = 50.0;
        let mut ship = Ship::create(w, h, Some(0));
        let ctx = GameContext { w, h, t };
        ship.update(&ctx);
        expect_that!(
            ship.shoot(),
            some(anything()),
            "should start not in cooldown"
        );
        // Cooldown starts
        let iters_required = BASE_SHOT_COOLDOWN_MS / t;
        for i in 0..(iters_required.ceil() as u32) {
            expect_that!(
                ship.shoot(),
                none(),
                "should not create shot cooldown is active, elapsed={}, updates={}",
                t * i as f32,
                i
            );
            ship.update(&ctx);
        }
        // Cooldown ends
        expect_that!(
            ship.shoot(),
            some(anything()),
            "should create shot when cooldown ends"
        );
    }
}
