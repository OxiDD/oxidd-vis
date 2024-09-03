use std::collections::{HashMap, HashSet};

use itertools::{Either, Itertools};
use ordered_float::OrderedFloat;
use web_sys::WebGl2RenderingContext as Gl;

use crate::{
    types::util::drawing::{
        diagram_layout::{Point, Transition},
        renderers::webgl::{text::text_renderer::Text, util::vertex_renderer::VertexRenderer},
    },
    util::{logging::console, matrix4::Matrix4},
};

use super::{
    super::text::text_renderer::{TextRenderer, TextRendererSettings},
    layer_renderer::{Layer, LayerDivisionRenderer},
};

pub struct LayerLinesRenderer {
    vertex_renderer: VertexRenderer,
    color: (f32, f32, f32, f32),
}

impl LayerLinesRenderer {
    pub fn new(context: &Gl) -> LayerLinesRenderer {
        let vertex_renderer = VertexRenderer::new(
            context,
            include_str!("layer_lines_renderer.vert"),
            include_str!("layer_lines_renderer.frag"),
        )
        .unwrap();
        LayerLinesRenderer {
            vertex_renderer,
            color: (0.9, 0.9, 0.9, 1.),
        }
    }
}
impl LayerDivisionRenderer for LayerLinesRenderer {
    fn set_layers(&mut self, context: &Gl, layers: &Vec<Layer>) {
        let (old_lines, new_lines): (HashMap<_, _>, HashMap<_, _>) = layers
            .iter()
            .flat_map(|layer| {
                vec![
                    (OrderedFloat(layer.top.new), layer),
                    (OrderedFloat(layer.bottom.new), layer),
                ]
            })
            .partition_map(|(line, layer)| {
                if layer.exists.new > 0.5 {
                    Either::Right((line, layer))
                } else {
                    Either::Left((line, layer))
                }
            });

        let lines = layers
            .iter()
            .flat_map(|layer| vec![OrderedFloat(layer.top.new), OrderedFloat(layer.bottom.new)])
            .sorted()
            .dedup()
            .map(|line| {
                (
                    line.0,
                    match (old_lines.get(&line), new_lines.get(&line)) {
                        (Some(old_layer), Some(new_layer)) => Transition::plain(1.0),
                        (Some(old_layer), None) => old_layer.exists,
                        (None, Some(new_layer)) => new_layer.exists,
                        _ => panic!("not in old nor new lines, impossible?"),
                    },
                )
            })
            .collect_vec();
        console::log!(
            "Lines: {}",
            lines
                .iter()
                .map(|line| format!("[{}: {}]", line.0, line.1))
                .join(", \n")
        );

        const COUNT: usize = 2;
        let y_positions = lines
            .iter()
            .flat_map(|&(line, _)| vec![line; COUNT])
            .collect_vec();
        self.vertex_renderer
            .set_data(context, "yPosition", &y_positions, 1);

        let exists = lines
            .iter()
            .flat_map(|(_, exists)| [exists.new; COUNT])
            .collect::<Box<[f32]>>();
        self.vertex_renderer.set_data(context, "exists", &exists, 1);

        let exists_old = lines
            .iter()
            .flat_map(|(_, exists)| [exists.old; COUNT])
            .collect::<Box<[f32]>>();
        self.vertex_renderer
            .set_data(context, "existsOld", &exists_old, 1);

        let exists_old_times = lines
            .iter()
            .flat_map(|(_, exists)| [exists.old_time as f32; COUNT])
            .collect::<Box<[f32]>>();
        self.vertex_renderer
            .set_data(context, "existsStartTime", &exists_old_times, 1);

        let exists_durations = lines
            .iter()
            .flat_map(|(_, exists)| [exists.duration as f32; COUNT])
            .collect::<Box<[f32]>>();
        self.vertex_renderer
            .set_data(context, "existsDuration", &exists_durations, 1);

        self.vertex_renderer.send_data(context);
    }

    fn set_transform(&mut self, context: &Gl, transform: &Matrix4) {
        self.vertex_renderer.set_uniform(context, "transform", |u| {
            context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
        });
    }

    fn render(&mut self, context: &Gl, time: u32) {
        self.vertex_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));

        let (r1, g1, b1, a1) = self.color;
        self.vertex_renderer
            .set_uniform(context, "color", |u| context.uniform4f(u, r1, g1, b1, a1));

        self.vertex_renderer.render(context, Gl::LINES);
    }

    fn dispose(&mut self, context: &Gl) {
        self.vertex_renderer.dispose(context);
    }
}
