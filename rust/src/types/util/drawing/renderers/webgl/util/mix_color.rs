use crate::types::util::drawing::layouts::util::color_label::Color;

/// Performs interpolation between color1 and color2, such that frac=1 outputs color2
pub fn mix_color(color1: Color, color2: Color, frac: f32) -> Color {
    let inv_frac = (1. - frac);
    (
        (color1.0 * color1.0 * inv_frac + color2.0 * color2.0 * frac).sqrt(),
        (color1.1 * color1.1 * inv_frac + color2.1 * color2.1 * frac).sqrt(),
        (color1.2 * color1.2 * inv_frac + color2.2 * color2.2 * frac).sqrt(),
    )
}
