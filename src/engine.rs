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

pub struct Engine {
    w: u32,
    h: u32,
    ship: Ship,
    interval: Interval,
    keydown: EventListener,
    keyup: EventListener,
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
            ship: Ship::create(WIDTH as f32, HEIGHT as f32),
            interval: interval,
            keydown: keydown,
            keyup: keyup,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                self.ship.update(INTERVAL_DURATION_MILLIS as f32);
                true
            }
            Msg::Keydown(e) => {
                match e.key().as_str() {
                    "w" | "W" => self.ship.thrust(),
                    "a" | "A" => self.ship.rotate_left(),
                    "d" | "D" => self.ship.rotate_right(),
                    "." | ">" => self.ship.shoot(),
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
            </svg>
        }
    }
}
