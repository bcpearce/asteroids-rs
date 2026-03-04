use yew::{Html, html};

pub struct Ship {
    pub x: f32,
    pub y: f32,
    pub v: f32,
    pub theta_rad: f32,
    pub sz: f32,
}
impl Ship {
    pub fn render(&self) -> Html {
        let p1x = self.theta_rad.cos() * self.sz + self.x;
        let p1y = self.theta_rad.sin() * self.sz + self.y;
        let p2x = (self.theta_rad + 0.75 * std::f32::consts::PI).cos() * self.sz * 0.6 + self.x;
        let p2y = (self.theta_rad + 0.75 * std::f32::consts::PI).sin() * self.sz * 0.6 + self.y;
        let p3x = (self.theta_rad - 0.75 * std::f32::consts::PI).cos() * self.sz * 0.6 + self.x;
        let p3y = (self.theta_rad - 0.75 * std::f32::consts::PI).sin() * self.sz * 0.6 + self.y;

        let points = format!(
            "{:.0},{:.0} {:.0},{:.0} {:.0},{:.0}",
            p1x, p1y, p2x, p2y, p3x, p3y
        );

        html! { <polygon points={points} stroke="white" /> }
    }
}
