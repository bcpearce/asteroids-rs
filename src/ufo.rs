use crate::collisions::{ShipCollidable, ShotCollidable};
use crate::common::rng::get_rng;
use crate::debris::Debris;
use crate::engine::{GameContext, GameElement};
use crate::ferris;
use crate::math::{Ellipse, Point, ellipse, point, polar_point};
use crate::shot::Shot;
use rand::RngExt;
use rand::seq::IndexedRandom;
use yew::{Html, html};

const SCORE_TTL_BASE_MS: f32 = 800.0;
const RESPAWN_TTL_BASE_MS: f32 = 10000.0;
const LARGE_WIDTH: f32 = 25.0;
const SMALL_WIDTH: f32 = 15.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Destroyed,
    Hidden,
    InViewSmall,
    InViewLarge,
}

#[derive(Clone, Copy, Debug)]
pub struct Ufo {
    pub p: Point,
    v: Point,
    pub state: State,
    score_ttl: f32,
    respawn_ttl: f32,
}

impl Ufo {
    pub fn create() -> Ufo {
        Ufo {
            p: point!(0, 0),
            v: point!(0, 0),
            state: State::Hidden,
            score_ttl: SCORE_TTL_BASE_MS,
            respawn_ttl: 0.0,
        }
    }

    pub fn maybe_spawn(&self, ctx: GameContext, maybe_seed: Option<u64>) -> Option<Ufo> {
        let (w, h) = (ctx.w, ctx.h);
        if self.state == State::Hidden && self.respawn_ttl < 0.0 {
            let mut rng = get_rng(maybe_seed);
            if rng.random_range(0..10) == 0 {
                let spawn_choices = [
                    (State::InViewSmall, point!(0.15, 0)),
                    (State::InViewSmall, point!(-0.15, 0)),
                    (State::InViewLarge, point!(0.1, 0)),
                    (State::InViewLarge, point!(-0.1, 0)),
                ];
                let (state, v) = spawn_choices
                    .choose(&mut rng)
                    .unwrap_or(&(State::InViewLarge, point!(0.1, 0)));
                let px = if v.x > 0.0 { 0.0 } else { w };
                let py = rng.random_range(0.1..0.9) * h;
                return Some(Ufo {
                    p: point!(px, py),
                    v: *v,
                    state: *state,
                    score_ttl: SCORE_TTL_BASE_MS,
                    respawn_ttl: RESPAWN_TTL_BASE_MS,
                });
            }
        }
        None
    }

    pub fn get_hitbox(&self) -> Ellipse {
        let width = match self.state {
            State::Destroyed => 0.0,
            State::Hidden => 0.0,
            State::InViewLarge => LARGE_WIDTH * 0.5,
            State::InViewSmall => SMALL_WIDTH * 0.5,
        };
        let height = width / ferris::ASPECT_RATIO;
        ellipse!(self.p.x, self.p.y, width, height)
    }

    pub fn get_debris(&self) -> Vec<Debris> {
        const ORANGE_RED: f32 = 16.2;
        if self.alive() {
            Vec::new()
        } else {
            let mut res = (0..5)
                .map(|t| {
                    let theta = t as f32 / 2.5 * std::f32::consts::PI - 0.5 * std::f32::consts::PI;
                    Debris {
                        p: self.p + polar_point!(self.get_hitbox().by / 4.0, theta),
                        v: polar_point!(self.v.mag() / 2.0, theta),
                        hue: ORANGE_RED,
                    }
                })
                .collect::<Vec<Debris>>();
            res.extend((0..32).map(|t| {
                let theta = t as f32 / 16.0 * std::f32::consts::PI - 0.5 * std::f32::consts::PI;
                Debris {
                    p: self.p + polar_point!(self.get_hitbox().by / 2.0, theta),
                    v: polar_point!(self.v.mag(), theta),
                    hue: ORANGE_RED,
                }
            }));
            res
        }
    }
}

impl GameElement for Ufo {
    fn update(&mut self, ctx: &GameContext) {
        self.p += self.v * ctx.t;
        match self.state {
            State::InViewLarge | State::InViewSmall => {
                if !(0.0..(ctx.w)).contains(&self.p.x) || !(0.0..(ctx.h)).contains(&self.p.y) {
                    self.state = State::Hidden
                }
            }
            State::Destroyed => {
                self.v = point!(0, 0.01);
                self.score_ttl -= ctx.t;
                if self.score_ttl < 0.0 {
                    self.state = State::Hidden;
                }
            }
            State::Hidden => {
                self.respawn_ttl -= ctx.t;
            }
        }
    }

    fn alive(&self) -> bool {
        self.state != State::Destroyed
    }

    fn render(&self) -> Html {
        let hitbox = if cfg!(debug_assertions) {
            let hitbox = self.get_hitbox();
            html! {
                    <ellipse
                    cx={hitbox.center.x.to_string()}
                    cy={hitbox.center.y.to_string()}
                    rx={hitbox.ax.to_string()}
                    ry={hitbox.by.to_string()}
                    fill="none"
                    stroke="purple"
                    stroke-width="1"
                    />
            }
        } else {
            html! {<></>}
        };

        match self.state {
            State::Destroyed => {
                html! {
                    <text></text>
                }
            }
            State::Hidden => html! {},
            State::InViewLarge => html! {
            <>
                {ferris::center_at(self.p, LARGE_WIDTH)}
                {hitbox}
            </>},
            State::InViewSmall => html! {
            <>
                {ferris::center_at(self.p, SMALL_WIDTH)}
                {hitbox}
            </>},
        }
    }

    fn destroy(&mut self) {
        self.state = State::Destroyed
    }
}

impl ShotCollidable for Ufo {
    fn did_collide(&self, shot: &Shot) -> bool {
        shot.p.in_ellipse(&self.get_hitbox())
    }

    fn score(&self) -> i32 {
        match self.state {
            State::InViewSmall => 1000,
            State::InViewLarge => 200,
            _ => 0,
        }
    }
}

impl ShipCollidable for Ufo {
    fn did_collide(&self, ship: &crate::ship::Ship) -> bool {
        ship.polygon()
            .iter()
            .any(|p| p.in_ellipse(&self.get_hitbox()))
    }

    fn v(&self) -> Point {
        self.v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;
    use is_svg::is_svg_string;

    #[gtest]
    fn it_renders_valid_svg() {
        let ufo = Ufo::create();
        let svg_wrap = format!("<svg>{:?}</svg>", ufo.render());
        assert_that!(is_svg_string(&svg_wrap), is_true(), "{}", &svg_wrap);
    }
}
