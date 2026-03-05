use rand::{Rng, RngExt};
use yew::{Html, html};

pub struct Ship {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    omega_rad: f32,
    theta_rad: f32,
    sz: f32,
    w: f32,
    h: f32,
}
impl Ship {
    pub fn create(w: f32, h: f32) -> Ship {
        Ship {
            x: w / 2.0,
            y: h / 2.0,
            vx: 0.0,
            vy: 0.0,
            omega_rad: 0.0,
            theta_rad: std::f32::consts::PI * 0.25,
            sz: 10.0,
            w: w,
            h: h,
        }
    }

    pub fn update(&mut self, interval_duration_millis: f32) {
        self.theta_rad += self.omega_rad * interval_duration_millis;
        self.x += self.vx * interval_duration_millis;
        self.y += self.vy * interval_duration_millis;
        self.x = if self.x < 0.0 {
            self.w - self.x
        } else if self.x > self.w {
            self.x - self.w
        } else {
            self.x
        };
        self.y = if self.y < 0.0 {
            self.h - self.y
        } else if self.y > self.h {
            self.y - self.h
        } else {
            self.y
        };
    }

    pub fn thrust(&mut self) {
        let (sin, cos) = self.theta_rad.sin_cos();
        self.vx += 0.01 * cos;
        self.vy += 0.01 * sin;
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

    pub fn shoot(&self) {}

    pub fn hyperspace(&mut self) {
        let mut rng = rand::rng();
        self.x = rng.random_range(0.0..=self.w);
        self.y = rng.random_range(0.0..=self.h);
        self.theta_rad = rng.random_range(0.0..=2.0 * std::f32::consts::PI)
    }

    pub fn render(&self) -> Html {
        let p1x = self.theta_rad.cos() * self.sz + self.x;
        let p1y = self.theta_rad.sin() * self.sz + self.y;
        let p2x = (self.theta_rad + 0.75 * std::f32::consts::PI).cos() * self.sz * 0.6 + self.x;
        let p2y = (self.theta_rad + 0.75 * std::f32::consts::PI).sin() * self.sz * 0.6 + self.y;
        let p3x = (self.theta_rad - 0.75 * std::f32::consts::PI).cos() * self.sz * 0.6 + self.x;
        let p3y = (self.theta_rad - 0.75 * std::f32::consts::PI).sin() * self.sz * 0.6 + self.y;

        let points = format!("{},{} {},{} {},{}", p1x, p1y, p2x, p2y, p3x, p3y);

        html! { <polygon points={points} stroke="white" /> }
    }
}
