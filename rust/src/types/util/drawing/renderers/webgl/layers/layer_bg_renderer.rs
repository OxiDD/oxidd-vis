use itertools::Itertools;
use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::renderers::webgl::{
        text::text_renderer::Text,
        util::{set_animated_data::set_animated_data, vertex_renderer::VertexRenderer},
    },
    util::{color::TransparentColor, logging::console, matrix4::Matrix4, transition::Transition},
};

use super::{
    super::text::text_renderer::{TextRenderer, TextRendererSettings},
    layer_renderer::{Layer, LayerDivisionRenderer},
};

pub struct LayerBgRenderer {
    bg_renderer: VertexRenderer,
    bg_color1: TransparentColor,
    bg_color2: TransparentColor,
}

impl LayerBgRenderer {
    pub fn new(
        context: &WebGl2RenderingContext,
        color1: TransparentColor,
        color2: TransparentColor,
    ) -> LayerBgRenderer {
        let vertex_renderer = VertexRenderer::new(
            context,
            include_str!("layer_bg_renderer.vert"),
            include_str!("layer_bg_renderer.frag"),
        )
        .unwrap();
        LayerBgRenderer {
            bg_renderer: vertex_renderer,
            bg_color1: color1,
            bg_color2: color2,
        }
    }
}
impl LayerDivisionRenderer for LayerBgRenderer {
    fn set_layers(&mut self, context: &WebGl2RenderingContext, layers: &Vec<Layer>) {
        fn map(layers: &Vec<Layer>, map: impl Fn(&Transition<f32>) -> f32) -> Box<[f32]> {
            layers
                .iter()
                .flat_map(|layer| {
                    let top = map(&layer.top);
                    let bottom = map(&layer.bottom);
                    [top, bottom, bottom, top, bottom, top]
                })
                .collect()
        }

        let mut layers = layers.clone();
        layers.sort_by_key(|layer| layer.exists.new > 0.5);

        let layer_vertices = layers.iter().flat_map(|layer| {
            [
                layer.top,
                layer.bottom,
                layer.bottom,
                layer.top,
                layer.bottom,
                layer.top,
            ]
        });
        set_animated_data(
            "yPosition",
            layer_vertices.clone(),
            |v| [v],
            context,
            &mut self.bg_renderer,
        );

        let layers6 = layers.iter().flat_map(|layer| [layer; 6]);
        set_animated_data(
            "type",
            layers6.clone().map(|l| l.index),
            |v| [v % 2.],
            context,
            &mut self.bg_renderer,
        );

        set_animated_data(
            "exists",
            layers6.clone().map(|l| l.exists),
            |v| [v],
            context,
            &mut self.bg_renderer,
        );

        self.bg_renderer.send_data(context);
    }

    fn set_transform(&mut self, context: &WebGl2RenderingContext, transform: &Matrix4) {
        self.bg_renderer.set_uniform(context, "transform", |u| {
            context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
        });
    }

    fn render(&mut self, context: &WebGl2RenderingContext, time: u32) {
        self.bg_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));

        let TransparentColor(r1, g1, b1, a1) = self.bg_color1;
        self.bg_renderer
            .set_uniform(context, "color1", |u| context.uniform4f(u, r1, g1, b1, a1));

        let TransparentColor(r2, g2, b2, a2) = self.bg_color2;
        self.bg_renderer
            .set_uniform(context, "color2", |u| context.uniform4f(u, r2, g2, b2, a2));

        self.bg_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);
    }

    fn dispose(&mut self, context: &WebGl2RenderingContext) {
        self.bg_renderer.dispose(context);
    }
}
