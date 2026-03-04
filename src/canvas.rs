use crate::ship::Ship;
use yew::{Component, Context, Html, html};

pub struct Canvas {
    w: i32,
    h: i32,
    ship: Ship,
}
impl Component for Canvas {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            w: 720,
            h: 480,
            ship: Ship {
                x: 720.0 / 2.0,
                y: 480.0 / 2.0,
                v: 0.0,
                theta_rad: 0.0,
                sz: 10.0,
            },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        unimplemented!()
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let view_box = format!("0 0 {} {}", self.w, self.h);

        html! {
            <svg class="canvas-window" viewBox={view_box}>
                {self.ship.render()}
            </svg>
        }
    }
}
