use std::ops::Deref;

use swash::{
    scale::{ScaleContext, Scaler},
    shape::ShapeContext,
    FontRef,
};

use crate::util::logging::console;

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

    pub fn text_size(&self) -> f32 {
        self.text_size
    }
}

impl AsRef<FontRef<'static>> for Font {
    fn as_ref(&self) -> &FontRef<'static> {
        &self.font
    }
}
