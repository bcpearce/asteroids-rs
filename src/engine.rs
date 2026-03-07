use crate::asteroid::Asteroid;
use crate::ship::Ship;
use crate::shot::Shot;
use gloo::events::EventListener;
use gloo::timers::callback::Interval;
use web_sys::wasm_bindgen::JsCast;
use yew::{Component, Context, Html, html};

const INTERVAL_DURATION_MILLIS: u32 = 33;
const WIDTH: u32 = 480;
const HEIGHT: u32 = 240;

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
    fn render(&self) -> Html;
}

pub struct Engine {
    pub w: u32,
    pub h: u32,
    pub t: f32,
    ship: Ship,
    shots: Vec<Shot>,
    asteroids: Vec<Asteroid>,
    _interval: Interval,
    _keydown: EventListener,
    _keyup: EventListener,
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
        self.ship.update(&ctx);
        for a in self.asteroids.iter_mut() {
            a.update(&ctx);
        }
        for s in self.shots.iter_mut() {
            s.update(&ctx);
        }
        self.shots.retain(|s| s.alive());
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
        let keydown = {
            let window = gloo::utils::window();
            let link = ctx.link().clone();
            EventListener::new(&window, "keydown", move |e| {
                e.prevent_default();
                let event = e.dyn_ref::<web_sys::KeyboardEvent>().unwrap().clone();
                link.send_message(Msg::Keydown(event));
            })
        };
        let keyup = {
            let window = gloo::utils::window();
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
            asteroids: vec![Asteroid::spawn(WIDTH as f32, HEIGHT as f32)],
            _interval: interval,
            _keydown: keydown,
            _keyup: keyup,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                self.handle_loop_update();
                true
            }
            Msg::Keydown(e) => {
                match e.key().as_str() {
                    "w" | "W" => self.ship.thrust(),
                    "a" | "A" => self.ship.rotate_left(),
                    "d" | "D" => self.ship.rotate_right(),
                    "." | ">" | "+" => self.ship.hyperspace(),
                    "Spacebar" | " " => self.shots.push(self.ship.shoot()),
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
                {self.ship.render()}
                {self.shots.iter().map(|s| s.render()).collect::<Html>()}
                {self.asteroids.iter().map(|a| a.render()).collect::<Html>()}
            </svg>
        }
    }
}
