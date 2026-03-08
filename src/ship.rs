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
            w: w,
            h: h,
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
        self.theta_rad = rng.random_range(0.0..=2.0 * std::f32::consts::PI)
    }
}

impl GameElement for Ship {
    fn update(&mut self, ctx: &GameContext) {
        self.theta_rad += self.omega_rad * ctx.t;
        self.p += self.v * ctx.t;
        self.p.wrap(ctx.w as f32, ctx.h as f32);
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
