use std::collections::HashMap;

use crate::asteroid::Size as AsteroidSize;
use crate::ship::Ship;
use crate::shot::Shot;
use crate::{asteroid::Asteroid, math::Circle};
use gloo::events::{EventListener, EventListenerOptions};
use gloo::timers::callback::Interval;
use web_sys::KeyboardEvent;
use web_sys::wasm_bindgen::JsCast;
use yew::{Component, Context, Html, html};

const INTERVAL_DURATION_MILLIS: u32 = 10;
const WIDTH: u32 = 480;
const HEIGHT: u32 = 480;
const BASE_DIFFICULTY: u32 = 10;
const MAX_ASTEROIDS: usize = 10;

const A_CODE_U: u32 = b'A' as u32;
const A_CODE_L: u32 = b'a' as u32;
const D_CODE_U: u32 = b'D' as u32;
const D_CODE_L: u32 = b'd' as u32;
#[cfg(test)]
const W_CODE_U: u32 = b'W' as u32;
const W_CODE_L: u32 = b'w' as u32;
const H_CODE_U: u32 = b'H' as u32;
const H_CODE_L: u32 = b'h' as u32;
const SPACE_CODE: u32 = b' ' as u32;

pub enum Msg {
    Tick,
    Keydown(u32),
    Keyup(u32),
}

#[derive(Clone, Debug)]
pub enum KeyAction {
    Up,
    Down,
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
    fn destroy(&mut self);
}

type KeyMap = HashMap<u32, KeyAction>;

struct WindowEventHandler {
    _interval: Interval,
    _keydown: EventListener,
    _keyup: EventListener,
}
pub struct Engine {
    pub w: u32,
    pub h: u32,
    pub t: f32,
    ship: Ship,
    shots: Vec<Shot>,
    asteroids: Vec<Asteroid>,
    difficulty: u32,
    score: i32,
    maybe_seed: Option<u64>,
    keymap: KeyMap,
    _maybe_window_event_handler: Option<WindowEventHandler>,
}
impl Engine {
    fn create_impl(
        maybe_ctx: Option<&Context<Self>>,
        difficulty: u32,
        maybe_engine_seed: Option<u64>,
        maybe_ship_seed: Option<u64>,
    ) -> Self {
        let maybe_window_event_handler = {
            if let Some(ctx) = maybe_ctx {
                let link = ctx.link().clone();
                let interval = Interval::new(INTERVAL_DURATION_MILLIS, move || {
                    link.send_message(Msg::Tick);
                });
                let window = gloo::utils::window();
                let options = EventListenerOptions::enable_prevent_default();
                let keydown = {
                    let link = ctx.link().clone();
                    EventListener::new_with_options(&window, "keydown", options, move |e| {
                        let event = e.dyn_ref::<KeyboardEvent>().unwrap().clone();
                        link.send_message(Msg::Keydown(event.key_code()));
                    })
                };
                let keyup = {
                    let link = ctx.link().clone();
                    EventListener::new_with_options(&window, "keyup", options, move |e| {
                        let event = e.dyn_ref::<KeyboardEvent>().unwrap().clone();
                        link.send_message(Msg::Keyup(event.key_code()));
                    })
                };
                Some(WindowEventHandler {
                    _interval: interval,
                    _keydown: keydown,
                    _keyup: keyup,
                })
            } else {
                None
            }
        };
        Self {
            w: WIDTH,
            h: HEIGHT,
            t: INTERVAL_DURATION_MILLIS as f32,
            ship: Ship::create(WIDTH as f32, HEIGHT as f32, maybe_ship_seed),
            shots: Vec::new(),
            asteroids: Vec::new(),
            difficulty,
            score: 0,
            maybe_seed: maybe_engine_seed,
            keymap: HashMap::new(),
            _maybe_window_event_handler: maybe_window_event_handler,
        }
    }

    fn update_impl(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::Tick => {
                self.handle_loop_update();
                while self.asteroids.len() < MAX_ASTEROIDS && self.difficulty > 0 {
                    self.spawn_asteroid();
                }
                true
            }
            Msg::Keydown(key) => {
                self.handle_keydown(key);
                false
            }
            Msg::Keyup(key) => {
                self.handle_keyup(key);
                false
            }
        }
    }

    fn get_context(&self) -> GameContext {
        GameContext {
            w: self.w as f32,
            h: self.h as f32,
            t: self.t,
        }
    }

    fn handle_loop_update(&mut self) {
        let ctx = self.get_context();
        if let Some(thrust) = self.keymap.get(&W_CODE_L) {
            match thrust {
                KeyAction::Down => self.ship.thrust(),
                KeyAction::Up => (),
            }
        }
        let mut game_elements: Vec<&mut dyn GameElement> = Vec::new();
        game_elements.push(&mut self.ship);
        game_elements.extend(self.asteroids.iter_mut().map(|a| a as &mut dyn GameElement));
        game_elements.extend(self.shots.iter_mut().map(|s| s as &mut dyn GameElement));

        for ge in game_elements.iter_mut() {
            ge.update(&ctx);
        }
        self.handle_shot_collision();
        self.handle_ship_collision();
        self.shots.retain(|s| s.alive());
        self.asteroids.retain(|a| a.alive());
    }

    fn spawn_asteroid(&mut self) {
        self.asteroids.push(Asteroid::spawn(
            self.w as f32,
            self.h as f32,
            self.maybe_seed,
        ));
        self.difficulty -= 1;
    }

    fn handle_shot_collision(&mut self) {
        for shot in self.shots.iter_mut().filter(|s| s.alive()) {
            let mut maybe_hit_index: Option<usize> = None;
            for (i, asteroid) in self
                .asteroids
                .iter()
                .filter(|&a| a.sz != AsteroidSize::Destroyed)
                .enumerate()
            {
                if shot.hitbox() | asteroid.hitbox() {
                    self.score += asteroid.score();
                    maybe_hit_index = Some(i);
                    shot.destroy();
                    break;
                }
            }
            if let Some(hit_index) = maybe_hit_index {
                let maybe_new_asteroids = self.asteroids[hit_index].split();
                if let Some(new_asteroids) = maybe_new_asteroids {
                    self.asteroids.extend(new_asteroids);
                }
                // Remove destroyed or split
                self.asteroids[hit_index].destroy();
            }
        }
    }

    fn handle_ship_collision(&mut self) {
        for asteroid in self
            .asteroids
            .iter()
            .filter(|&a| a.sz != AsteroidSize::Destroyed)
        {
            if asteroid.hitbox() | self.ship.hitbox() {
                self.ship.destroy();
                break;
            }
        }
    }

    fn add_shot(&mut self) {
        let maybe_shot = self.ship.shoot();
        if let Some(shot) = maybe_shot {
            self.shots.push(shot)
        }
    }

    fn handle_keydown(&mut self, key_code: u32) {
        match key_code {
            A_CODE_L | A_CODE_U => self.ship.rotate_left(),
            D_CODE_L | D_CODE_U => self.ship.rotate_right(),
            H_CODE_L | H_CODE_U => self.ship.hyperspace(),
            SPACE_CODE => self.add_shot(),
            65..=90 => {
                // Force lowercase entry
                self.keymap.insert(key_code | 0x20, KeyAction::Down);
            }
            _ => {
                self.keymap.insert(key_code, KeyAction::Down);
            }
        };
    }

    fn handle_keyup(&mut self, key_code: u32) {
        match key_code {
            A_CODE_L | A_CODE_U | D_CODE_L | D_CODE_U => self.ship.stop_rotate(),
            65..=90 => {
                // Force lowercase entry
                self.keymap.insert(key_code - 65, KeyAction::Up);
            }
            _ => {
                self.keymap.insert(key_code, KeyAction::Up);
            }
        }
    }

    fn render(&self) -> Html {
        html! {
            <g>
                {self.ship.render()}
                {self.shots.iter().map(|s| s.render()).collect::<Html>()}
                {self.asteroids.iter().map(|a| a.render()).collect::<Html>()}
                <text
                    x={(self.w as f32 * 0.1).to_string()}
                    y={(self.h as f32 * 0.1).to_string()}
                    fill="#FFFFFF"
                    stroke="#000000"
                    stroke-width="0.3"
                    font-size="25"
                    font-family="monospace">
                    {self.score}
                </text>
            </g>
        }
    }
}

impl Component for Engine {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self::create_impl(Some(ctx), BASE_DIFFICULTY, None, None)
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.update_impl(msg)
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let view_box = format!("0 0 {} {}", self.w, self.h);
        html! {
        <svg class="svg-container" viewBox={view_box}>
            {self.render()}
        </svg>
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::math::Point;

    use super::*;
    use googletest::prelude::*;
    use indoc::indoc;
    use is_svg::is_svg_string;
    use p_test::p_test;
    use quickcheck::{Arbitrary, Gen, QuickCheck};
    use std::{collections::HashMap, rc::Rc};
    use strum::IntoEnumIterator;

    fn create_test_engine(difficulty: u32, engine_seed: u64, ship_seed: u64) -> Engine {
        Engine::create_impl(None, difficulty, Some(engine_seed), Some(ship_seed))
    }

    #[gtest]
    fn it_renders() {
        let engine = create_test_engine(BASE_DIFFICULTY, 42, 42);
        let svg_string = format!("<svg>{:?}</svg>", engine.render());
        assert_that!(is_svg_string(&svg_string), is_true(), "{:?}", &svg_string)
    }

    #[p_test(
        (Point { x: 10.0, y: 10.0 }, false, AsteroidSize::Large),
        (Point { x: 10.0, y: 10.0 }, true, AsteroidSize::Destroyed),
        (Point { x: 100.0, y: 100.0 }, true, AsteroidSize::Large),
    )]
    fn it_handles_ship_collisions(
        ship_point: Point,
        expect_alive: bool,
        asteroid_size: AsteroidSize,
    ) {
        let mut engine = create_test_engine(BASE_DIFFICULTY, 42, 42);
        let p = Point { x: 10.0, y: 10.0 };
        let v = Point { x: 10.0, y: 10.0 };
        let edge_points = Rc::new(vec![
            Point { x: 1.0, y: 0.0 },
            Point { x: -1.0, y: 0.0 },
            Point { x: 0.0, y: 1.0 },
            Point { x: 0.0, y: -1.0 },
        ]);
        engine.asteroids.push(Asteroid {
            p,
            v,
            edge_points,
            sz: asteroid_size,
            hue: 0,
        });
        engine.ship = Ship::create_for_test(ship_point);
        engine.handle_ship_collision();
        assert_that!(engine.ship.alive(), eq(expect_alive));
    }

    #[p_test(
        (Point { x: 10.0, y: 10.0 }, true, AsteroidSize::Large),
        (Point { x: 10.0, y: 10.0 }, false, AsteroidSize::Destroyed),
        (Point { x: 100.0, y: 100.0 }, false, AsteroidSize::Large),
    )]
    fn it_handles_shot_collisions(
        shot_point: Point,
        expect_split: bool,
        asteroid_size: AsteroidSize,
    ) {
        let mut engine = create_test_engine(BASE_DIFFICULTY, 42, 42);
        let p = Point { x: 10.0, y: 10.0 };
        let v = Point { x: 10.0, y: 10.0 };
        let edge_points = Rc::new(vec![
            Point { x: 1.0, y: 0.0 },
            Point { x: -1.0, y: 0.0 },
            Point { x: 0.0, y: 1.0 },
            Point { x: 0.0, y: -1.0 },
        ]);
        engine.asteroids.push(Asteroid {
            p,
            v,
            edge_points,
            sz: asteroid_size,
            hue: 0,
        });
        engine.ship = Ship::create_for_test(shot_point);
        engine
            .shots
            .push(engine.ship.shoot().expect("Shot should be created"));
        engine.handle_shot_collision();
        let shot_count = engine.shots.iter().filter(|&s| s.alive()).count();
        let asteroid_count = engine.asteroids.len(); // include destroyed
        if expect_split {
            assert_that!(asteroid_count, eq(3));
            assert_that!(shot_count, eq(0));
        } else {
            assert_that!(asteroid_count, eq(1));
            assert_that!(shot_count, eq(1));
        }
    }

    #[derive(Clone, Debug)]
    pub struct GameKeyInput(pub Option<(u32, KeyAction)>);

    impl Arbitrary for GameKeyInput {
        fn arbitrary(g: &mut Gen) -> Self {
            let keys = [
                Some(W_CODE_L),
                Some(W_CODE_U),
                Some(A_CODE_L),
                Some(A_CODE_U),
                Some(D_CODE_L),
                Some(D_CODE_U),
                Some(H_CODE_L),
                Some(H_CODE_U),
                Some(SPACE_CODE),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ];
            if let Some(key) = g.choose(&keys).unwrap() {
                let key_action_opts = &[KeyAction::Up, KeyAction::Down];
                GameKeyInput(Some((*key, g.choose(key_action_opts).unwrap().clone())))
            } else {
                GameKeyInput(None)
            }
        }
    }

    #[gtest]
    fn it_keeps_score_as_engine_runs() {
        fn run_engine(
            actions: Vec<GameKeyInput>,
            difficulty: u32,
            engine_seed: u64,
            ship_seed: u64,
        ) -> bool {
            let mut engine = create_test_engine(difficulty % 500, engine_seed, ship_seed);

            for game_key_input in actions {
                if let Some((key, is_down_action)) = game_key_input.0 {
                    match is_down_action {
                        KeyAction::Down => engine.update_impl(Msg::Keydown(key)),
                        KeyAction::Up => engine.update_impl(Msg::Keyup(key)),
                    };
                }
                engine.update_impl(Msg::Tick);

                struct AsteroidMap(HashMap<AsteroidSize, i32>);
                impl AsteroidMap {
                    fn create() -> AsteroidMap {
                        let mut res: AsteroidMap = AsteroidMap(HashMap::new());
                        for sz in AsteroidSize::iter() {
                            res.0.insert(sz, 0);
                        }
                        res
                    }
                }
                fn count_asteroids(asteroids: &[Asteroid]) -> AsteroidMap {
                    let mut counts = AsteroidMap::create();
                    for a in asteroids.iter() {
                        *counts.0.entry(a.sz).or_insert(0) += 1;
                    }
                    counts
                }

                let score_before_loop = engine.score;
                let shots_before_loop = engine.shots.len();
                let asteroids_before_loop = count_asteroids(&engine.asteroids);

                engine.handle_loop_update();

                let score_after_loop = engine.score;
                let shots_after_loop = engine.shots.len();
                let asteroids_after_loop = count_asteroids(&engine.asteroids);

                if score_after_loop > score_before_loop {
                    assert_that!(
                        shots_after_loop,
                        lt(shots_before_loop),
                        "if the score went up, a shot must have contacted a target"
                    );
                }

                let destroyed_asteroids = {
                    let mut map = AsteroidMap::create();

                    // Destroyed large are direct
                    let destroyed_large = asteroids_before_loop.0[&AsteroidSize::Large]
                        - asteroids_after_loop.0[&AsteroidSize::Large];
                    map.0.insert(AsteroidSize::Large, destroyed_large);

                    // Destroyed medium must account for 2 new ones from destroyed large
                    let destroyed_medium = asteroids_before_loop.0[&AsteroidSize::Medium]
                        - asteroids_after_loop.0[&AsteroidSize::Medium]
                        + destroyed_large * 2;
                    map.0.insert(AsteroidSize::Medium, destroyed_medium);

                    // Destroyed small must account for 2 new ones from destroyed medium
                    let destroyed_small = asteroids_before_loop.0[&AsteroidSize::Small]
                        - asteroids_after_loop.0[&AsteroidSize::Small]
                        + destroyed_medium * 2;
                    map.0.insert(AsteroidSize::Small, destroyed_small);

                    map
                };

                let score_from_destroyed = AsteroidSize::iter()
                    .map(|sz| destroyed_asteroids.0[&sz] * Asteroid::score_from_size(&sz))
                    .reduce(|acc, val| acc + val)
                    .expect("Score was not provided by reducer");

                assert_that!(
                    score_after_loop,
                    eq(score_from_destroyed + score_before_loop),
                    indoc! {"
                    Expected score to increase from {} to {},
                    asteroids_before={:?}, asteroids_after={:?},
                    destroyed={:?}"},
                    score_before_loop,
                    score_after_loop,
                    asteroids_before_loop.0,
                    asteroids_after_loop.0,
                    destroyed_asteroids.0
                );
            }
            true
        }

        QuickCheck::new()
            .rng(Gen::new(1000))
            .tests(100)
            .quickcheck(run_engine as fn(Vec<GameKeyInput>, u32, u64, u64) -> bool)
    }
}
