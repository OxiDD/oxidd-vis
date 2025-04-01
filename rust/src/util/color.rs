use super::transition::Interpolatable;

// pub type Color = (f32, f32, f32);
#[derive(Clone, PartialEq, Copy)]
pub struct Color(pub f32, pub f32, pub f32);
impl Interpolatable for Color {
    fn mix(&self, c2: &Self, per: f32) -> Self {
        let r = (self.0 * self.0 * (1.0 - per) + c2.0 * c2.0 * per).sqrt();
        let b = (self.2 * self.2 * (1.0 - per) + c2.2 * c2.2 * per).sqrt();
        let g = (self.1 * self.1 * (1.0 - per) + c2.1 * c2.1 * per).sqrt();
        Color(r, g, b)
    }
}

impl Color {
    pub fn mix_transparent(&self, c2: &TransparentColor) -> Self {
        self.mix(&Color(c2.0, c2.1, c2.2), c2.3)
    }
}

impl Into<TransparentColor> for Color {
    fn into(self) -> TransparentColor {
        TransparentColor(self.0, self.1, self.2, 1.0)
    }
}

#[derive(Clone, PartialEq, Copy)]
pub struct TransparentColor(pub f32, pub f32, pub f32, pub f32);
impl Interpolatable for TransparentColor {
    fn mix(&self, c2: &Self, per: f32) -> Self {
        let r = (self.0 * self.0 * (1.0 - per) + c2.0 * c2.0 * per).sqrt();
        let b = (self.2 * self.2 * (1.0 - per) + c2.2 * c2.2 * per).sqrt();
        let g = (self.1 * self.1 * (1.0 - per) + c2.1 * c2.1 * per).sqrt();
        let a = self.3 * (1.0 - per) + c2.3 * per;
        TransparentColor(r, g, b, a)
    }
}
