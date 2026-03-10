use std::collections::HashSet;

use crate::ship::Ship;
use crate::shot::Shot;
use crate::{asteroid::Asteroid, math::Circle};
use gloo::events::EventListener;
use gloo::timers::callback::Interval;
use web_sys::wasm_bindgen::JsCast;
use yew::{Component, Context, Html, html};

const INTERVAL_DURATION_MILLIS: u32 = 33;
const WIDTH: u32 = 480;
const HEIGHT: u32 = 480;
const BASE_DIFFICULTY: u32 = 10;
const MAX_ASTEROIDS: usize = 10;

pub enum Msg {
    Tick,
    Keydown(web_sys::KeyboardEvent),
    Keyup(web_sys::KeyboardEvent),
}

pub struct GameContext {
    pub w: f32,
    pub h: f32,
    pub t: f32,
}

pub trait GameElement {
    fn update(&mut self, ctx: &GameContext);
    fn alive(&self) -> bool;
    fn hitbox(&self) -> Circle;
    fn render(&self) -> Html;
}

pub struct Engine {
    pub w: u32,
    pub h: u32,
    pub t: f32,
    ship: Ship,
    shots: Vec<Shot>,
    asteroids: Vec<Asteroid>,
    difficulty: u32,
    score: u32,
    _interval: Option<Interval>,
    _keydown: Option<EventListener>,
    _keyup: Option<EventListener>,
}
impl Engine {
    fn get_context(&self) -> GameContext {
        GameContext {
            w: self.w as f32,
            h: self.h as f32,
            t: self.t,
        }
    }

    fn handle_loop_update(&mut self) {
        let ctx = self.get_context();
        let mut game_elements: Vec<&mut dyn GameElement> = Vec::new();
        game_elements.push(&mut self.ship);
        game_elements.extend(self.asteroids.iter_mut().map(|a| a as &mut dyn GameElement));
        game_elements.extend(self.shots.iter_mut().map(|s| s as &mut dyn GameElement));

        for ge in game_elements.iter_mut() {
            ge.update(&ctx);
        }
        self.handle_collision();
        self.shots.retain(|s| s.alive());
        self.asteroids.retain(|a| a.alive());
    }

    fn spawn_asteroid(&mut self) {
        self.asteroids
            .push(Asteroid::spawn(self.w as f32, self.h as f32));
        self.difficulty -= 1;
    }

    fn handle_collision(&mut self) {
        let mut shot_indexes_used: HashSet<usize> = HashSet::new();
        for (shot_index, shot) in self.shots.iter().enumerate() {
            let mut maybe_hit_index: Option<usize> = None;
            for (i, asteroid) in self.asteroids.iter().enumerate() {
                if shot.hitbox() | asteroid.hitbox() {
                    self.score += asteroid.score();
                    maybe_hit_index = Some(i);
                    shot_indexes_used.insert(shot_index);
                    break;
                }
            }
            if let Some(hit_index) = maybe_hit_index {
                let maybe_new_asteroids = self.asteroids[hit_index].split();
                if let Some(new_asteroids) = maybe_new_asteroids {
                    self.asteroids.extend(new_asteroids);
                }
                // Remove destroyed or split
                self.asteroids.swap_remove(hit_index);
            }
        }
        // Clear used-up shots
        for index in shot_indexes_used {
            self.shots.swap_remove(index);
        }
    }
}

impl Component for Engine {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let interval = {
            let link = ctx.link().clone();
            Interval::new(INTERVAL_DURATION_MILLIS, move || {
                link.send_message(Msg::Tick);
            })
        };
        let window = gloo::utils::window();
        let keydown = {
            let link = ctx.link().clone();
            EventListener::new(&window, "keydown", move |e| {
                e.prevent_default();
                let event = e.dyn_ref::<web_sys::KeyboardEvent>().unwrap().clone();
                link.send_message(Msg::Keydown(event));
            })
        };
        let keyup = {
            let link = ctx.link().clone();
            EventListener::new(&window, "keyup", move |e| {
                e.prevent_default();
                let event = e.dyn_ref::<web_sys::KeyboardEvent>().unwrap().clone();
                link.send_message(Msg::Keyup(event));
            })
        };
        Self {
            w: WIDTH,
            h: HEIGHT,
            t: INTERVAL_DURATION_MILLIS as f32,
            ship: Ship::create(WIDTH as f32, HEIGHT as f32),
            shots: Vec::new(),
            asteroids: Vec::new(),
            difficulty: BASE_DIFFICULTY,
            score: 0,
            _interval: Some(interval),
            _keydown: Some(keydown),
            _keyup: Some(keyup),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                self.handle_loop_update();
                while self.asteroids.len() < MAX_ASTEROIDS && self.difficulty > 0 {
                    self.spawn_asteroid();
                }
                true
            }
            Msg::Keydown(e) => {
                match e.key().as_str() {
                    "w" | "W" => self.ship.thrust(),
                    "a" | "A" => self.ship.rotate_left(),
                    "d" | "D" => self.ship.rotate_right(),
                    "." | ">" | "+" => self.ship.hyperspace(),
                    "Spacebar" | " " => {
                        let maybe_shot = self.ship.shoot();
                        if let Some(shot) = maybe_shot {
                            self.shots.push(shot)
                        }
                    }
                    _ => (),
                }
                false
            }
            Msg::Keyup(e) => {
                match e.key().as_str() {
                    "a" | "A" => self.ship.stop_rotate(),
                    "d" | "D" => self.ship.stop_rotate(),
                    _ => (),
                }
                false
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let view_box = format!("0 0 {} {}", self.w, self.h);
        html! {
            <svg class="svg-container" viewBox={view_box}>
                <text
                    x={(self.w as f32 * 0.1).to_string()}
                    y={(self.h as f32 * 0.1).to_string()}
                    fill="#FFFFFF" font-size="20" font-family="Verdana">
                    {self.score}
                </text>
                {self.ship.render()}
                {self.shots.iter().map(|s| s.render()).collect::<Html>()}
                {self.asteroids.iter().map(|a| a.render()).collect::<Html>()}
            </svg>
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    fn create_test_engine() -> Engine {
        Engine {
            w: WIDTH,
            h: HEIGHT,
            t: INTERVAL_DURATION_MILLIS as f32,
            ship: Ship::create(WIDTH as f32, HEIGHT as f32),
            shots: Vec::new(),
            asteroids: Vec::new(),
            difficulty: BASE_DIFFICULTY,
            score: 0,
            _interval: None,
            _keydown: None,
            _keyup: None,
        }
    }

    #[quickcheck]
    fn it_runs_a_game_engine(iter_count: u32) {
        let mut engine = create_test_engine();
        let iter_count = iter_count % 50_000; // limit to 5000 iterations
        for _ in 0..iter_count {
            engine.handle_loop_update();
        }
        // Pass if reaches here without panic
    }
}
