use crate::common;
use crate::common::rng::get_rng;
use crate::debris::LineDebris;
use crate::engine::{GameContext, GameElement};
use crate::math::{Point, point, polar_point};
use crate::shot::Shot;
use itertools::Itertools;
use rand::RngExt;
use yew::{Html, html};

const BASE_SHOT_COOLDOWN_MS: f32 = 75.0;
const BASE_HYPERSPACE_COOLDOWN_MS: f32 = 1500.0;
const THRUST_FACTOR: f32 = 3e-3;
const THRUST_TTL_MS: f32 = 300.0;
const THRUST_TTL_INC_MS: f32 = 30.0;
const ROTATION_FACTOR: f32 = std::f32::consts::PI / 250.0;
const SHIP_SZ: f32 = 10.0;

pub struct Ship {
    pub p: Point,
    pub v: Point,
    omega_rad: f32,
    theta_rad: f32,
    w: f32,
    h: f32,
    shot_cooldown_ms: f32,
    hyperspace_p: Point,
    hyperspace_cooldown_ms: f32,
    maybe_seed: Option<u64>,
    is_destroyed: bool,
    thrust_ttl_ms: f32,
}
impl Ship {
    pub fn create(w: f32, h: f32, maybe_seed: Option<u64>) -> Ship {
        Ship {
            p: point!(w / 2.0, h / 2.0),
            v: point!(0, 0),
            omega_rad: 0.0,
            theta_rad: std::f32::consts::PI * 0.25,
            w,
            h,
            shot_cooldown_ms: 0.0,
            hyperspace_p: point!(w / 2.0, h / 2.0),
            hyperspace_cooldown_ms: 0.0,
            maybe_seed,
            is_destroyed: false,
            thrust_ttl_ms: 0.0,
        }
    }

    #[cfg(test)]
    pub fn create_for_test(p: Point) -> Ship {
        Ship {
            p,
            v: Point { x: 0.0, y: 0.0 },
            omega_rad: 0.0,
            theta_rad: std::f32::consts::PI * 0.25,
            w: 100.0,
            h: 100.0,
            shot_cooldown_ms: 0.0,
            hyperspace_p: point!(0, 0),
            hyperspace_cooldown_ms: 0.0,
            maybe_seed: Some(0),
            is_destroyed: false,
            thrust_ttl_ms: 0.0,
        }
    }

    pub fn thrust(&mut self) {
        let dv = Point::from_polar(THRUST_FACTOR, self.theta_rad);
        self.thrust_ttl_ms += THRUST_TTL_INC_MS;
        self.thrust_ttl_ms = self.thrust_ttl_ms.clamp(0.0, THRUST_TTL_MS);
        self.v.x += dv.x;
        self.v.y += dv.y;
    }

    pub fn rotate_left(&mut self) {
        self.omega_rad = -ROTATION_FACTOR;
    }

    pub fn rotate_right(&mut self) {
        self.omega_rad = ROTATION_FACTOR;
    }

    pub fn stop_rotate(&mut self) {
        self.omega_rad = 0.0;
    }

    pub fn shoot(&mut self) -> Option<Shot> {
        if self.shot_cooldown_ms > 0.0 || self.is_destroyed {
            None
        } else {
            self.shot_cooldown_ms = BASE_SHOT_COOLDOWN_MS;
            Some(Shot::create(self.p, self.v, self.theta_rad))
        }
    }

    pub fn hyperspace(&mut self) {
        if self.hyperspace_cooldown_ms < 0.0 {
            self.hyperspace_p = self.p;
            let mut rng = get_rng(self.maybe_seed);
            self.p.x = rng.random_range(0.0..=self.w);
            self.p.y = rng.random_range(0.0..=self.h);
            self.theta_rad = rng.random_range(0.0..=2.0 * std::f32::consts::PI);
            self.v = Point { x: 0.0, y: 0.0 };
            self.hyperspace_cooldown_ms = BASE_HYPERSPACE_COOLDOWN_MS;
        }
    }

    pub fn polygon(&self) -> Vec<Point> {
        Self::polygon_at_point_and_rotation(self.p, self.theta_rad)
    }

    pub fn polygon_at_point_and_rotation(p: Point, theta_rad: f32) -> Vec<Point> {
        let p1 = Point::from_polar(SHIP_SZ, theta_rad) + p;
        let p2 = Point::from_polar(SHIP_SZ * 0.6, theta_rad + 0.75 * std::f32::consts::PI) + p;
        let p3 = Point::from_polar(SHIP_SZ * 0.6, theta_rad - 0.75 * std::f32::consts::PI) + p;
        vec![p1, p2, p3]
    }

    pub fn spawn_debris(&self, impact_velocity: Point) -> Vec<LineDebris> {
        let mut rng = common::rng::get_rng(self.maybe_seed);
        let points = self.polygon();
        (0..points.len())
            .map(move |i| LineDebris {
                p1: points[i],
                p2: points[(i + 1) % points.len()],
                v: self.v
                    + impact_velocity
                    + point!(rng.random_range(0.005..0.01), rng.random_range(0.005..0.01)),
                w: rng.random_range(-0.005..=0.005),
            })
            .collect()
    }
}

impl GameElement for Ship {
    fn update(&mut self, ctx: &GameContext) {
        self.theta_rad += self.omega_rad * ctx.t;
        self.p += self.v * ctx.t;
        self.p.wrap(ctx.w, ctx.h);
        self.shot_cooldown_ms -= ctx.t;
        self.thrust_ttl_ms -= ctx.t;
        self.hyperspace_cooldown_ms -= ctx.t;
        self.thrust_ttl_ms = self.thrust_ttl_ms.clamp(0.0, THRUST_TTL_MS);
    }

    fn alive(&self) -> bool {
        !self.is_destroyed
    }

    fn render(&self) -> Html {
        let points = self.polygon().into_iter().join(" ");
        let thrusters = {
            let p1 = polar_point!(SHIP_SZ * 0.3, self.theta_rad + 0.5 * std::f32::consts::PI);
            let p2 = polar_point!(SHIP_SZ * 0.3, self.theta_rad - 0.5 * std::f32::consts::PI);
            let p3 = polar_point!(
                SHIP_SZ * self.thrust_ttl_ms / THRUST_TTL_MS,
                self.theta_rad + std::f32::consts::PI
            );
            [p1, p2, p3]
                .iter()
                .map(|p| *p + self.p)
                .collect::<Vec<Point>>()
        };
        let hyperspace_path = {
            const STAR_POINT_MAG: f32 = 3.0;
            const CONTROL_MAG: f32 = 0.02;
            let factor =
                self.hyperspace_cooldown_ms * (1.0 / BASE_HYPERSPACE_COOLDOWN_MS) * STAR_POINT_MAG;
            let corners = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)]
                .iter()
                .map(|(xp, yp)| self.hyperspace_p + point!(*xp, *yp) * factor)
                .collect::<Vec<Point>>();
            let controls = [
                (CONTROL_MAG, CONTROL_MAG),
                (-CONTROL_MAG, CONTROL_MAG),
                (-CONTROL_MAG, -CONTROL_MAG),
                (-CONTROL_MAG, CONTROL_MAG),
            ]
            .iter()
            .map(|(xp, yp)| self.hyperspace_p + point!(*xp, *yp) * factor)
            .collect::<Vec<Point>>();
            format!(
                "M {} {} Q {} {}, {} {} M {} {} Q {} {}, {} {} M {} {} Q {} {}, {} {} M {} {} Q {} {}, {} {}",
                corners[0].x,
                corners[0].y,
                controls[0].x,
                controls[0].y,
                corners[1].x,
                corners[1].y,
                corners[1].x,
                corners[1].y,
                controls[1].x,
                controls[1].y,
                corners[2].x,
                corners[2].y,
                corners[2].x,
                corners[2].y,
                controls[2].x,
                controls[2].y,
                corners[3].x,
                corners[3].y,
                corners[3].x,
                corners[3].y,
                controls[3].x,
                controls[3].y,
                corners[0].x,
                corners[0].y,
            )
        };
        html! {
        <g>
            if self.alive() {
                <polygon points={thrusters.into_iter().join(" ")} stroke="orange" />
                <polygon points={points} stroke="white" />
            } else {<></>}
            if self.hyperspace_cooldown_ms > 0.0 {
                <path d={hyperspace_path} stroke="white"/>
            }
        </g>
         }
    }

    fn destroy(&mut self) {
        self.is_destroyed = true
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
    fn it_renders_valid_svg(
        w: PositiveFloat,
        h: PositiveFloat,
        seed: u64,
        is_destroyed: bool,
    ) -> TestResult {
        let mut s = Ship::create(w.0, h.0, Some(seed));
        if is_destroyed {
            s.destroy();
        }
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
