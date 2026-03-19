#![deny(warnings)]

mod asteroid;
mod collisions;
mod common;
mod debris;
mod engine;
mod ferris;
mod math;
mod ship;
mod shot;
mod ufo;

use engine::Engine;
use yew::prelude::*;

#[component]
fn App() -> Html {
    html! {
        <div class="engine-window">
            <Engine />
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
