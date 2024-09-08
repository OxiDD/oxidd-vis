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
