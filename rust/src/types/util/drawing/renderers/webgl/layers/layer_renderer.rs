use itertools::Itertools;
use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::{
        diagram_layout::{Point, Transition},
        renderers::webgl::text::text_renderer::Text,
    },
    util::{logging::console, matrix4::Matrix4},
};

use super::super::{
    text::text_renderer::{TextRenderer, TextRendererSettings},
    vertex_renderer::VertexRenderer,
};

pub struct LayerRenderer {
    division_renderer: Box<dyn LayerDivisionRenderer>,
    text_renderer: TextRenderer,
    text_size: f32,
}

#[derive(Clone)]
pub struct Layer {
    pub top: Transition<f32>,
    pub bottom: Transition<f32>,
    pub label: String,
    pub index: Transition<f32>,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this layer is visible (0-1)
}

impl LayerRenderer {
    pub fn new<D: LayerDivisionRenderer + 'static>(
        context: &WebGl2RenderingContext,
        layer_divider: D,
        screen_height: usize,
        font_data: Vec<u8>,
        font_settings: TextRendererSettings,
    ) -> LayerRenderer {
        LayerRenderer {
            division_renderer: Box::new(layer_divider),
            text_size: font_settings.text_size,
            text_renderer: TextRenderer::new(context, font_data, font_settings, screen_height),
        }
    }

    pub fn set_layers(&mut self, context: &WebGl2RenderingContext, layers: &Vec<Layer>) {
        self.division_renderer.set_layers(context, layers);

        self.text_renderer.set_texts(
            context,
            &layers
                .iter()
                .map(|layer| {
                    let b = layer.bottom;
                    Text {
                        text: layer.label.clone(),
                        position: Transition {
                            old_time: b.old_time,
                            duration: b.duration,
                            old: Point { x: 0., y: b.old },
                            new: Point { x: 0., y: b.new },
                        },
                        exists: layer.exists,
                    }
                })
                .collect(),
        );
    }

    pub fn set_transform_and_screen_height(
        &mut self,
        context: &WebGl2RenderingContext,
        transform: &Matrix4,
        screen_height: usize,
    ) {
        self.division_renderer.set_transform(context, transform);

        let margin = 0.5 * self.text_size;
        let modified_transform = &mut transform.clone();
        modified_transform.0[3] = -0.5 + margin * transform.0[0]; // Shift by -1 on the x-axis + margin
        modified_transform.0[7] += margin * transform.0[5]; // Shift up by margin
        self.text_renderer.set_transform_and_screen_height(
            context,
            modified_transform,
            screen_height,
        );
    }

    // pub fn set_screen_height(&mut self, context: &WebGl2RenderingContext, height: usize) {
    //     self.text_renderer.set_screen_height(context, height);
    // }

    pub fn render(
        &mut self,
        context: &WebGl2RenderingContext,
        time: u32,
        selected_ids: &[u32],
        hovered_ids: &[u32],
    ) {
        self.division_renderer
            .render(context, time, selected_ids, hovered_ids);

        self.text_renderer.render(context, time);
    }

    pub fn dispose(&mut self, context: &WebGl2RenderingContext) {
        self.division_renderer.dispose(context);
        self.text_renderer.dispose(context);
    }
}

pub trait LayerDivisionRenderer {
    fn set_layers(&mut self, context: &WebGl2RenderingContext, layers: &Vec<Layer>);
    fn set_transform(&mut self, context: &WebGl2RenderingContext, transform: &Matrix4);
    fn render(
        &mut self,
        context: &WebGl2RenderingContext,
        time: u32,
        selected_ids: &[u32],
        hovered_ids: &[u32],
    );
    fn dispose(&mut self, context: &WebGl2RenderingContext);
}
