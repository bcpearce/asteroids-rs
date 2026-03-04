use yew::{html, Component, Context, Html};


pub struct Canvas {

}
impl Component for Canvas {
    type Message = ();
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        unimplemented!()
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        let view_box = format!("0 0 {} {}", 1600, 1000);

        html! {
            <svg class="canvas-window" viewBox={view_box}>
                <circle cx="50" cy="50" r="40" stroke="black" stroke-width="3" fill="red" />
            </svg>
        }
    }
}