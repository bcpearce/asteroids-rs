use crate::asteroid::Asteroid;
use crate::ship::Ship;
use gloo::events::EventListener;
use gloo::timers::callback::Interval;
use web_sys::{KeyboardEvent, wasm_bindgen::JsCast};
use yew::{Component, Context, Html, Properties, html};

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
pub struct Engine {
    pub w: u32,
    pub h: u32,
    pub t: f32,
    ship: Ship,
    asteroids: Vec<Asteroid>,
    interval: Interval,
    keydown: EventListener,
    keyup: EventListener,
}
impl Engine {
    fn get_context(&self) -> GameContext {
        GameContext {
            w: self.w as f32,
            h: self.h as f32,
            t: self.t,
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
            asteroids: vec![Asteroid::spawn(WIDTH as f32, HEIGHT as f32)],
            interval: interval,
            keydown: keydown,
            keyup: keyup,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                let ctx = self.get_context();
                self.ship.update(&ctx);
                for a in self.asteroids.iter_mut() {
                    a.update(&ctx);
                }
                true
            }
            Msg::Keydown(e) => {
                match e.key().as_str() {
                    "w" | "W" => self.ship.thrust(),
                    "a" | "A" => self.ship.rotate_left(),
                    "d" | "D" => self.ship.rotate_right(),
                    "." | ">" | "+" => self.ship.shoot(),
                    "Spacebar" | " " => self.ship.hyperspace(),
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
                {self.asteroids.iter().map(|a| a.render()).collect::<Html>()}
            </svg>
        }
    }
}
