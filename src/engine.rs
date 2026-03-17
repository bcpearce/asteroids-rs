use crate::asteroid::Asteroid;
use crate::asteroid::Size as AsteroidSize;
use crate::collisions::ShipCollidable;
use crate::collisions::ShotCollidable;
use crate::debris::{Debris, LineDebris};
use crate::ship::Ship;
use crate::shot::Shot;
use crate::ufo::Ufo;
use gloo::events::{EventListener, EventListenerOptions};
use gloo::timers::callback::Interval;
use std::collections::HashMap;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
use web_sys::KeyboardEvent;
use web_sys::wasm_bindgen::JsCast;
use yew::{Component, Context, Html, html};

#[cfg(test)]
static QUICKCHECK_RUN_COUNT: AtomicUsize = AtomicUsize::new(0);

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
    ufo: Ufo,
    debris: Vec<Debris>,
    line_debris: Vec<LineDebris>,
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
            ufo: Ufo::create(),
            debris: Vec::new(),
            line_debris: Vec::new(),
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
                self.spawn_ufo();
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
        game_elements.push(&mut self.ufo);
        game_elements.extend(self.debris.iter_mut().map(|d| d as &mut dyn GameElement));
        game_elements.extend(
            self.line_debris
                .iter_mut()
                .map(|d| d as &mut dyn GameElement),
        );
        game_elements.extend(self.asteroids.iter_mut().map(|a| a as &mut dyn GameElement));
        game_elements.extend(self.shots.iter_mut().map(|s| s as &mut dyn GameElement));

        for ge in game_elements.iter_mut() {
            ge.update(&ctx);
        }
        self.handle_shot_collision();
        self.handle_ship_collision();
        self.shots.retain(|s| s.alive());
        self.debris.extend(
            self.asteroids
                .iter()
                .filter(|a| !a.alive())
                .map(|a| a.make_debris()),
        );
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

    fn spawn_ufo(&mut self) {
        if let Some(ufo) = self.ufo.maybe_spawn(self.maybe_seed) {
            self.ufo = ufo;
        }
    }

    fn handle_shot_collision(&mut self) {
        for shot in self.shots.iter_mut().filter(|s| s.alive()) {
            let maybe_hit_index: Option<usize> = (|| {
                let mut shot_collidables: Vec<&mut dyn ShotCollidable> = Vec::new();
                shot_collidables.extend(self.asteroids.iter_mut().filter_map(|a| match a.sz {
                    AsteroidSize::Destroyed => None,
                    _ => Some(a as &mut dyn ShotCollidable),
                }));
                shot_collidables.push(&mut self.ufo);
                for (i, collidable) in shot_collidables.iter().enumerate() {
                    if collidable.did_collide(shot) {
                        self.score += collidable.score();
                        shot.destroy();
                        return Some(i);
                    }
                }
                None
            })();
            if let Some(hit_index) = maybe_hit_index {
                if hit_index < self.asteroids.len() {
                    let maybe_new_asteroids = self.asteroids[hit_index].split();
                    if let Some(new_asteroids) = maybe_new_asteroids {
                        self.asteroids.extend(new_asteroids);
                    }
                    // Remove destroyed or split
                    self.asteroids[hit_index].destroy();
                } else {
                    self.ufo.destroy();
                }
            }
        }
    }

    fn handle_ship_collision(&mut self) {
        if !self.ship.alive() {
            return;
        }
        let mut ship_collidables: Vec<&mut dyn ShipCollidable> = Vec::new();
        ship_collidables.extend(self.asteroids.iter_mut().filter_map(|a| match a.sz {
            AsteroidSize::Destroyed => None,
            _ => Some(a as &mut dyn ShipCollidable),
        }));
        for collidable in ship_collidables {
            if collidable.did_collide(&self.ship) {
                self.ship.destroy();
                self.line_debris
                    .extend(self.ship.spawn_debris(collidable.v()));
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
                self.keymap.insert(key_code + 32, KeyAction::Up);
            }
            _ => {
                self.keymap.insert(key_code, KeyAction::Up);
            }
        }
    }

    fn render(&self) -> Html {
        html! {
            <>
                {self.debris.iter().map(|d| d.render()).collect::<Html>()}
                {self.line_debris.iter().map(|d| d.render()).collect::<Html>()}
                {self.ship.render()}
                {self.ufo.render()}
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
            </>
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
    use super::*;
    use crate::math::Point;
    use crate::math::{point, polar_point};
    use core::f32;
    use googletest::prelude::*;
    use indoc::indoc;
    use is_svg::is_svg_string;
    use p_test::p_test;
    use quickcheck::{Arbitrary, Gen, QuickCheck};
    use quickcheck_macros::quickcheck;
    use std::collections::HashMap;
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

    fn create_edge_points(edge_point_count: u32) -> Vec<Point> {
        (0..=edge_point_count)
            .map(|i| Point::from_polar(1.0, f32::consts::PI * i as f32 / edge_point_count as f32))
            .collect()
    }

    #[p_test(
        (point!(10, 10), false, AsteroidSize::Large),
        (point!(10, 10), true, AsteroidSize::Destroyed),
        (point!(100, 100), true, AsteroidSize::Large),
    )]
    fn it_handles_ship_collisions(
        ship_point: Point,
        expect_alive: bool,
        asteroid_size: AsteroidSize,
    ) {
        let mut engine = create_test_engine(BASE_DIFFICULTY, 42, 42);
        let p = point!(10, 10);
        let v = point!(10, 10);
        let edge_points = create_edge_points(8);
        engine
            .asteroids
            .push(Asteroid::create(p, v, edge_points, asteroid_size));
        engine.ship = Ship::create_for_test(ship_point);
        engine.handle_ship_collision();
        assert_that!(engine.ship.alive(), eq(expect_alive));
    }

    #[p_test(
        (point!(10, 10), true, AsteroidSize::Large),
        (point!(10, 10), false, AsteroidSize::Destroyed),
        (point!(100, 100), false, AsteroidSize::Large),
    )]
    fn it_handles_shot_collisions(
        shot_point: Point,
        expect_split: bool,
        asteroid_size: AsteroidSize,
    ) {
        let mut engine = create_test_engine(BASE_DIFFICULTY, 42, 42);
        let p = point!(10, 10);
        let v = point!(10, 10);
        let edge_points_sz = 8;
        let edge_points = (0..=edge_points_sz)
            .map(|i| polar_point!(1.0, f32::consts::PI * i as f32 / edge_points_sz as f32))
            .collect();
        engine
            .asteroids
            .push(Asteroid::create(p, v, edge_points, asteroid_size));
        engine.shots.push(Shot::create(shot_point, v, 0.0));
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

    fn quickcheck_params() -> (usize, u64) {
        let mut arbitrary_size: usize = 1000;
        let mut tests: u64 = 100;

        if let Ok(v) = std::env::var("QUICKCHECK_SIZE") {
            if let Ok(parsed) = v.parse::<usize>() {
                arbitrary_size = parsed;
            }
        }
        if let Ok(v) = std::env::var("QUICKCHECK_TESTS") {
            if let Ok(parsed) = v.parse::<u64>() {
                tests = parsed;
            }
        }

        (arbitrary_size, tests)
    }

    #[gtest]
    fn it_keeps_score_as_engine_runs() {
        fn run_engine(
            actions: Vec<GameKeyInput>,
            difficulty: u32,
            engine_seed: u64,
            ship_seed: u64,
        ) -> bool {
            let difficulty = difficulty % 500;
            let mut engine = create_test_engine(difficulty, engine_seed, ship_seed);
            let run_count = QUICKCHECK_RUN_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
            println!(
                "{:>6}: Running Engine PBT for {} iterations, difficulty={}",
                run_count,
                actions.len(),
                difficulty
            );
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

                match engine.ship.alive() {
                    true => assert_that!(
                        engine.line_debris.len(),
                        eq(0),
                        "if the ship is alive, no debris from it"
                    ),
                    false => {
                        assert_that!(
                            engine.line_debris.len(),
                            eq(engine.ship.polygon().len()),
                            "if the ship is destroyed, expect the polygon to create LineDebris"
                        )
                    }
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

        let (arbitrary_size, tests) = quickcheck_params();
        println!(
            "Running with arbitraries of size {}, {} tests",
            arbitrary_size, tests
        );
        QuickCheck::new()
            .rng(Gen::new(arbitrary_size))
            .tests(tests)
            .quickcheck(run_engine as fn(Vec<GameKeyInput>, u32, u64, u64) -> bool)
    }

    #[quickcheck]
    fn it_never_spawns_an_asteroid_on_the_ship(engine_seed: u64) {
        let mut engine = create_test_engine(BASE_DIFFICULTY, engine_seed, 0);
        engine.spawn_asteroid();
        engine.handle_ship_collision();
        assert_that!(engine.ship.alive(), is_true());
    }
}
