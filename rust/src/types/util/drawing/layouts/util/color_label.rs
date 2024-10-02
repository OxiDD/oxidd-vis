use std::ops::Add;

pub trait ColorLabel {
    fn get_color(&self) -> Color;
    fn get_outline_color(&self) -> TransparentColor;
}
impl ColorLabel for (f32, f32, f32) {
    fn get_color(&self) -> Color {
        self.clone()
    }
    fn get_outline_color(&self) -> TransparentColor {
        transparent_color(&self, 0.0)
    }
}
impl ColorLabel for ((f32, f32, f32), (f32, f32, f32, f32)) {
    fn get_color(&self) -> Color {
        self.0.clone()
    }
    fn get_outline_color(&self) -> TransparentColor {
        self.1.clone()
    }
}

pub type Color = (f32, f32, f32);

pub fn mix(c1: &Color, c2: &Color, per: f32) -> Color {
    let r = (c1.0 * c1.0 * (1.0 - per) + c2.0 * c2.0 * per).sqrt();
    let b = (c1.2 * c1.2 * (1.0 - per) + c2.2 * c2.2 * per).sqrt();
    let g = (c1.1 * c1.1 * (1.0 - per) + c2.1 * c2.1 * per).sqrt();
    (r, g, b)
}

pub type TransparentColor = (f32, f32, f32, f32);
pub fn transparent_color(color: &Color, alpha: f32) -> TransparentColor {
    (color.0, color.1, color.2, alpha)
}
pub fn mix_transparent(c1: &TransparentColor, c2: &TransparentColor, per: f32) -> TransparentColor {
    transparent_color(
        &mix(&(c1.0, c1.1, c1.2), &(c2.0, c2.1, c2.2), per),
        c1.3 * (1.0 - per) + c2.3 * per,
    )
}
