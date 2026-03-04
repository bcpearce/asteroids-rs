mod canvas;
mod ship;

use canvas::Canvas;
use yew::prelude::*;

#[component]
fn App() -> Html {
    html! {
        <div>
            <Canvas />
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
