use std::ops::Deref;

use swash::{
    scale::{ScaleContext, Scaler},
    shape::ShapeContext,
    FontRef,
};

use crate::util::logging::console;

const SCALERES: f32 = 200.;
pub struct Font {
    _font_data: Box<[u8]>,
    font: FontRef<'static>,
    text_size: f32,
}

impl Font {
    pub fn new(font_data: Vec<u8>, text_size: f32) -> Font {
        let font_data: Box<[u8]> = font_data.into_boxed_slice();
        let font_data_ref = unsafe { std::mem::transmute::<&[u8], &'static [u8]>(&*font_data) };
        let font = FontRef::from_index(&font_data_ref[..], 0).unwrap();

        Font {
            _font_data: font_data,
            font,
            text_size,
        }
    }

    pub fn measure_width(&self, text: &str) -> f32 {
        let mut shaper_context = Box::new(ShapeContext::new());
        let mut shaper = shaper_context
            .builder(self.font)
            .size(self.text_size)
            .build();
        shaper.add_str(text);
        let mut res: f32 = 0.;
        shaper.shape_with(|cluster| {
            res += cluster.advance();
        });
        res
    }

    pub fn measure_height(&self, text: &str) -> f32 {
        let mut scaler_context = Box::new(ScaleContext::new());
        let scaler_context_ref = unsafe {
            std::mem::transmute::<&mut ScaleContext, &'static mut ScaleContext>(
                scaler_context.as_mut(),
            )
        };
        let mut scaler = scaler_context_ref
            .builder(self.font)
            .size(self.text_size * SCALERES)
            .build();

        let charmap = self.font.charmap();
        let glyphs = text.chars().map(|char| charmap.map(char));
        let max_height = glyphs
            .filter_map(|glyph| Some(scaler.scale_outline(glyph)?.bounds().height()))
            .reduce(|a, b| a.max(b));
        max_height
            .map(|max| max / SCALERES)
            .unwrap_or(self.text_size)
    }

    pub fn text_size(&self) -> f32 {
        self.text_size
    }

    pub fn with_text_size(&self, size: f32) -> Self {
        Self::new(self._font_data.clone().into(), size)
    }
}

impl AsRef<FontRef<'static>> for Font {
    fn as_ref(&self) -> &FontRef<'static> {
        &self.font
    }
}
