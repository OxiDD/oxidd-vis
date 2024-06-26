use itertools::Itertools;
use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::{
        diagram_layout::{Point, Transition},
        renderers::webgl::text::text_renderer::Text,
    },
    util::{logging::console, matrix4::Matrix4},
};

use super::{
    super::{
        text::text_renderer::{TextRenderer, TextRendererSettings},
        vertex_renderer::VertexRenderer,
    },
    layer_renderer::{Layer, LayerDivisionRenderer},
};

pub struct LayerBgRenderer {
    bg_renderer: VertexRenderer,
    bg_color1: (f32, f32, f32, f32),
    bg_color2: (f32, f32, f32, f32),
}

impl LayerBgRenderer {
    pub fn new(context: &WebGl2RenderingContext) -> LayerBgRenderer {
        let vertex_renderer = VertexRenderer::new(
            context,
            include_str!("layer_bg_renderer.vert"),
            include_str!("layer_bg_renderer.frag"),
        )
        .unwrap();
        LayerBgRenderer {
            bg_renderer: vertex_renderer,
            bg_color1: (0.9, 0.9, 0.9, 1.),
            bg_color2: (0.98, 0.98, 0.98, 1.),
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

        // TODO: write more helpers to clean this up

        let y_positions = map(&layers, |t| t.new);
        self.bg_renderer
            .set_data(context, "yPosition", &y_positions, 1);

        let y_positions_old = map(&layers, |t| t.old);
        self.bg_renderer
            .set_data(context, "yPositionOld", &y_positions_old, 1);

        let position_old_times = map(&layers, |t| t.old_time as f32);
        self.bg_renderer
            .set_data(context, "positionStartTime", &position_old_times, 1);

        let position_durations = map(&layers, |t| t.duration as f32);
        self.bg_renderer
            .set_data(context, "positionDuration", &position_durations, 1);

        let types = layers
            .iter()
            .flat_map(|layer| [(layer.index.new % 2.); 6])
            .collect::<Box<[f32]>>();
        self.bg_renderer.set_data(context, "type", &types, 1);

        let types_old = layers
            .iter()
            .flat_map(|layer| [(layer.index.old % 2.); 6])
            .collect::<Box<[f32]>>();
        self.bg_renderer.set_data(context, "typeOld", &types_old, 1);

        let types_old_times = layers
            .iter()
            .flat_map(|layer| [layer.index.old_time as f32; 6])
            .collect::<Box<[f32]>>();
        self.bg_renderer
            .set_data(context, "typeStartTime", &types_old_times, 1);

        let types_durations = layers
            .iter()
            .flat_map(|layer| [layer.index.duration as f32; 6])
            .collect::<Box<[f32]>>();
        self.bg_renderer
            .set_data(context, "typeDuration", &types_durations, 1);

        self.bg_renderer.update_data(context);
    }

    fn set_transform(&mut self, context: &WebGl2RenderingContext, transform: &Matrix4) {
        self.bg_renderer.set_uniform(context, "transform", |u| {
            context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
        });
    }

    fn render(
        &mut self,
        context: &WebGl2RenderingContext,
        time: u32,
        selected_ids: &[u32],
        hovered_ids: &[u32],
    ) {
        self.bg_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));

        let (r1, g1, b1, a1) = self.bg_color1;
        self.bg_renderer
            .set_uniform(context, "color1", |u| context.uniform4f(u, r1, g1, b1, a1));

        let (r2, g2, b2, a2) = self.bg_color2;
        self.bg_renderer
            .set_uniform(context, "color2", |u| context.uniform4f(u, r2, g2, b2, a2));

        self.bg_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);
    }

    fn dispose(&mut self, context: &WebGl2RenderingContext) {
        self.bg_renderer.dispose(context);
    }
}
