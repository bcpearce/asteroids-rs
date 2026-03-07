mod asteroid;
mod engine;
mod math;
mod ship;
mod shot;

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
