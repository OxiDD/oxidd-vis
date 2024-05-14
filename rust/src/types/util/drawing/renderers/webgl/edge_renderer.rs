use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::diagram_layout::{Point, Transition},
    util::logging::console,
};

use super::{matrix4::Matrix4, vertex_renderer::VertexRenderer};

pub struct EdgeRenderer {
    vertex_renderer: VertexRenderer,
    width: f32,
}

pub struct Edge {
    pub start: Transition<Point>,
    pub points: Vec<Transition<Point>>,
    pub end: Transition<Point>,
}

impl EdgeRenderer {
    pub fn new(context: &WebGl2RenderingContext) -> EdgeRenderer {
        let vertex_renderer = VertexRenderer::new(
            context,
            r##"#version 300 es
                in vec2 start;
                in vec2 startOld;
                in float startStartTime;
                in float startDuration;

                in vec2 end;
                in vec2 endOld;
                in float endStartTime;
                in float endDuration;

                uniform float width;
                uniform mat4 transform;
                uniform float time;

                void main() {
                    float startPer = min((time - startStartTime) / startDuration, 1.0);
                    vec2 curStart = startPer * start + (1.0 - startPer) * startOld;

                    float endPer = min((time - endStartTime) / endDuration, 1.0);
                    vec2 curEnd = endPer * end + (1.0 - endPer) * endOld;

                    vec2 delta = curEnd - curStart;
                    vec2 dir = normalize(delta);
                    vec2 dirOrth = vec2(-dir.y, dir.x);

                    int corner = gl_VertexID % 6; // two triangles
                    vec2 pos = (
                        corner == 0                  ? curStart + (-dir - dirOrth) * width
                        : corner == 1 || corner == 3 ? curStart + (-dir + dirOrth) * width
                        : corner == 2 || corner == 4 ? curEnd + (dir - dirOrth) * width
                        :                              curEnd + (dir + dirOrth) * width
                    );
                    gl_Position = transform * vec4(pos, 0.0, 1.0);
                }
                "##,
            r##"#version 300 es
                precision highp float;
                out vec4 outColor;
                // in vec2 cornerPos;
                // in vec2 curSize;

                // float cornerSize = 0.3;
                // float fuzziness = 0.03; // A form of anti-aliasing by making the circle border a slight gradient
                
                void main() {
                    float alpha = 1.0;
                    // float cornerSize2 = cornerSize * cornerSize;

                    // float xBoundary = curSize.x / 2.0 - cornerSize;
                    // float yBoundary = curSize.y / 2.0 - cornerSize;
                    // float absX = abs(cornerPos.x);
                    // float absY = abs(cornerPos.y);
                    // if (absX > xBoundary && absY > yBoundary) {
                    //     float dx = xBoundary - absX;
                    //     float dy = yBoundary - absY;
                    //     float distance2 = dx*dx + dy*dy;
                    //     if(distance2 >= cornerSize2)
                    //         alpha = 1.0 - max(0.0, (sqrt(distance2) - cornerSize) / fuzziness);
                    // }

                    outColor = vec4(1, 1, 1, alpha);
                }
                "##,
        )
        .unwrap();
        EdgeRenderer {
            vertex_renderer,
            width: 0.1,
        }
    }

    pub fn set_edges(&mut self, context: &WebGl2RenderingContext, edges: &Vec<Edge>) {
        fn map<const LEN: usize>(
            edges: &Vec<(Transition<Point>, Transition<Point>)>,
            map: impl Fn(&(Transition<Point>, Transition<Point>)) -> [f32; LEN],
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
                points.push(edge.end);
                let edge_segments = points
                    .iter()
                    .scan(edge.start, |prev, item| {
                        let out = (*prev, *item);
                        *prev = *item;
                        Some(out)
                    })
                    .collect::<Vec<(Transition<Point>, Transition<Point>)>>();
                edge_segments
            })
            .collect::<Vec<(Transition<Point>, Transition<Point>)>>();

        // console::log!(
        //     "{}",
        //     segments
        //         .iter()
        //         .map(|(start, end)| format!(
        //             "({}, {}; {}, {})",
        //             start.new.x, start.new.y, end.new.x, end.new.y
        //         ))
        //         .collect::<Vec<String>>()
        //         .join(", ")
        // );

        let old_starts = map(&segments, |segment| [segment.0.old.x, segment.0.old.y]);
        self.vertex_renderer
            .set_data(context, "startOld", &old_starts, 2);

        let starts = map(&segments, |segment| [segment.0.new.x, segment.0.new.y]);
        self.vertex_renderer.set_data(context, "start", &starts, 2);

        let start_old_times = map(&segments, |segment| [segment.0.old_time as f32]);
        self.vertex_renderer
            .set_data(context, "startStartTime", &start_old_times, 1);

        let start_durations = map(&segments, |segment| [segment.0.duration as f32]);
        self.vertex_renderer
            .set_data(context, "startDuration", &start_durations, 1);

        let old_ends = map(&segments, |segment| [segment.1.old.x, segment.1.old.y]);
        self.vertex_renderer
            .set_data(context, "endOld", &old_ends, 2);

        let ends = map(&segments, |segment| [segment.1.new.x, segment.1.new.y]);
        self.vertex_renderer.set_data(context, "end", &ends, 2);

        let end_old_times = map(&segments, |segment| [segment.1.old_time as f32]);
        self.vertex_renderer
            .set_data(context, "endStartTime", &end_old_times, 1);

        let end_durations = map(&segments, |segment| [segment.1.duration as f32]);
        self.vertex_renderer
            .set_data(context, "endDuration", &end_durations, 1);

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
        let width = self.width;
        self.vertex_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));
        self.vertex_renderer
            .set_uniform(context, "width", |u| context.uniform1f(u, width as f32));
        self.vertex_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);
    }

    pub fn dispose(&self, context: &WebGl2RenderingContext) {
        self.vertex_renderer.dispose(context);
    }
}
