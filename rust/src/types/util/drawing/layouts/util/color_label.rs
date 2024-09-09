use std::ops::Add;

pub trait ColorLabel {
    fn get_color(&self) -> Color;
}
impl ColorLabel for (f32, f32, f32) {
    fn get_color(&self) -> Color {
        self.clone()
    }
}

pub type Color = (f32, f32, f32);

pub fn mix(c1: &Color, c2: &Color, per: f32) -> Color {
    let r = (c1.0 * c1.0 * (1.0 - per) + c2.0 * c2.0 * per).sqrt();
    let b = (c1.2 * c1.2 * (1.0 - per) + c2.2 * c2.2 * per).sqrt();
    let g = (c1.1 * c1.1 * (1.0 - per) + c2.1 * c2.1 * per).sqrt();
    (r, g, b)
}
