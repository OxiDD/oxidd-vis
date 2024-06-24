use std::collections::HashMap;

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    types::util::drawing::{
        diagram_layout::{Point, Transition},
        renderers::webgl::render_texture::RenderTexture,
    },
    util::{logging::console, matrix4::Matrix4},
};

use super::vertex_renderer::VertexRenderer;

pub struct EdgeRenderer {
    vertex_renderer: VertexRenderer,
    edge_types: Vec<EdgeRenderingType>,
}

pub struct Edge {
    pub start: Transition<Point>,
    pub points: Vec<Transition<Point>>,
    pub end: Transition<Point>,
    pub edge_type: usize,
    pub shift: Transition<f32>, // Some sideways shift
}

#[derive(Clone)]
pub struct EdgeRenderingType {
    pub color: (f32, f32, f32),
    pub width: f32,
    pub dash_solid: f32, // The distance per period over which this dash should be solid
    pub dash_transparent: f32, // The distance per
}
type Segment = (
    Transition<Point>,
    Transition<Point>,
    f32,             /* type*/
    Transition<f32>, /* curvature */
);

impl EdgeRenderer {
    pub fn new(
        context: &WebGl2RenderingContext,
        edge_types: Vec<EdgeRenderingType>,
    ) -> EdgeRenderer {
        let type_count = edge_types.len();
        console::log!("uniform EdgeType types[{type_count}];");
        let vertex_renderer = VertexRenderer::new_advanced(
            context,
            &include_str!("edge_renderer.vert"),
            &include_str!("edge_renderer.frag"),
            Some(&HashMap::from([(
                "type_count",
                type_count.to_string().as_str(),
            )])),
        )
        .unwrap();

        EdgeRenderer {
            vertex_renderer,
            edge_types,
        }
    }

    pub fn set_edges(&mut self, context: &WebGl2RenderingContext, edges: &Vec<Edge>) {
        fn map<const LEN: usize>(
            edges: &Vec<Segment>,
            map: impl Fn(&Segment) -> [f32; LEN],
        ) -> Box<[f32]> {
            edges
                .iter()
                .flat_map(|edge| map(edge).repeat(6))
                // .flat_map(|node| [0.0, 0.0].repeat(4))
                .collect()
        }

        let segments = edges
            .iter()
            .flat_map(|edge| {
                let mut points = edge.points.clone();
                let edge_type = edge.edge_type;
                let curve_offset = edge.shift;
                points.push(edge.end);
                let edge_segments = points
                    .iter()
                    .scan(edge.start, |prev, item| {
                        let out = (*prev, *item, edge_type as f32, curve_offset);
                        *prev = *item;
                        Some(out)
                    })
                    .collect::<Vec<Segment>>();
                edge_segments
            })
            .collect::<Vec<Segment>>();

        // Start
        let old_starts = map(&segments, |(start, _, _, _)| [start.old.x, start.old.y]);
        self.vertex_renderer
            .set_data(context, "startOld", &old_starts, 2);

        let starts = map(&segments, |(start, _, _, _)| [start.new.x, start.new.y]);
        self.vertex_renderer.set_data(context, "start", &starts, 2);

        let start_old_times = map(&segments, |(start, _, _, _)| [start.old_time as f32]);
        self.vertex_renderer
            .set_data(context, "startStartTime", &start_old_times, 1);

        let start_durations = map(&segments, |(start, _, _, _)| [start.duration as f32]);
        self.vertex_renderer
            .set_data(context, "startDuration", &start_durations, 1);

        // End
        let old_ends = map(&segments, |(_, end, _, _)| [end.old.x, end.old.y]);
        self.vertex_renderer
            .set_data(context, "endOld", &old_ends, 2);

        let ends = map(&segments, |(_, end, _, _)| [end.new.x, end.new.y]);
        self.vertex_renderer.set_data(context, "end", &ends, 2);

        let end_old_times = map(&segments, |(_, end, _, _)| [end.old_time as f32]);
        self.vertex_renderer
            .set_data(context, "endStartTime", &end_old_times, 1);

        let end_durations = map(&segments, |(_, end, _, _)| [end.duration as f32]);
        self.vertex_renderer
            .set_data(context, "endDuration", &end_durations, 1);

        // Curve
        let old_curve_offsets = map(&segments, |(_, _, _, curve_offset)| [curve_offset.old]);
        self.vertex_renderer
            .set_data(context, "curveOffsetOld", &old_curve_offsets, 1);

        let curve_offsets = map(&segments, |(_, _, _, curve_offset)| [curve_offset.new]);
        self.vertex_renderer
            .set_data(context, "curveOffset", &curve_offsets, 1);

        let curve_offset_old_times = map(&segments, |(_, _, _, curve_offset)| {
            [curve_offset.old_time as f32]
        });
        self.vertex_renderer
            .set_data(context, "curveOffsetStartTime", &curve_offset_old_times, 1);

        let curve_offset_durations = map(&segments, |(_, _, _, curve_offset)| {
            [curve_offset.duration as f32]
        });
        self.vertex_renderer
            .set_data(context, "curveOffsetDuration", &curve_offset_durations, 1);

        // Type
        let edge_types = map(&segments, |&(_, _, edge_type, _)| [edge_type]);
        self.vertex_renderer
            .set_data(context, "type", &edge_types, 1);

        self.vertex_renderer.update_data(context);
    }

    pub fn set_transform(&mut self, context: &WebGl2RenderingContext, transform: &Matrix4) {
        self.vertex_renderer.set_uniform(context, "transform", |u| {
            context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
        });
    }

    pub fn render(
        &mut self,
        context: &WebGl2RenderingContext,
        time: u32,
        selected_ids: &[u32],
        hovered_ids: &[u32],
    ) {
        self.vertex_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));
        for (index, edge_type) in self.edge_types.iter().enumerate() {
            let c = edge_type.color;
            self.vertex_renderer
                .set_uniform(context, &format!("edgeTypes[{index}].color"), |u| {
                    context.uniform3f(u, c.0, c.1, c.2)
                });
            self.vertex_renderer
                .set_uniform(context, &format!("edgeTypes[{index}].width"), |u| {
                    context.uniform1f(u, edge_type.width)
                });
            self.vertex_renderer.set_uniform(
                context,
                &format!("edgeTypes[{index}].dashSolid"),
                |u| context.uniform1f(u, edge_type.dash_solid),
            );
            self.vertex_renderer.set_uniform(
                context,
                &format!("edgeTypes[{index}].dashTransparent"),
                |u| context.uniform1f(u, edge_type.dash_transparent),
            );
        }

        self.vertex_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);
    }

    pub fn dispose(&self, context: &WebGl2RenderingContext) {
        self.vertex_renderer.dispose(context);
    }
}
