use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::diagram_layout::{Point, Transition},
    util::logging::console,
};

use super::{matrix4::Matrix4, vertex_renderer::VertexRenderer};

pub struct NodeRenderer {
    vertex_renderer: VertexRenderer,
}

pub struct Node {
    pub center_position: Transition<Point>,
    pub size: Transition<Point>,
    pub label: String,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
}

impl NodeRenderer {
    pub fn new(context: &WebGl2RenderingContext) -> NodeRenderer {
        let vertex_renderer = VertexRenderer::new(
            context,
            r##"#version 300 es
            in vec2 position;
            in vec2 positionOld;
            in float positionStartTime;
            in float positionDuration;

            in vec2 size;
                in vec2 sizeOld;
                in float sizeStartTime;
                in float sizeDuration;
                
                uniform mat4 transform;
                uniform float time;

                out vec2 cornerPos;
                out vec2 curSize;

                void main() {
                    float positionPer = min((time - positionStartTime) / positionDuration, 1.0);
                    vec2 curPosition = positionPer * position + (1.0 - positionPer) * positionOld;

                    float sizePer = min((time - sizeStartTime) / sizeDuration, 1.0);
                    curSize = sizePer * size + (1.0 - sizePer) * sizeOld;

                    int corner = gl_VertexID % 6; // two triangles
                    cornerPos = curSize * (
                        corner == 0  || corner == 3  ? vec2(0.5, 0.5)
                        : corner == 1                ? vec2(0.5, -0.5)
                        : corner == 2 || corner == 4 ? vec2(-0.5, -0.5)
                        :                              vec2(-0.5, 0.5)
                    );
                    gl_Position = transform * vec4(curPosition + cornerPos, 0.0, 1.0);
                }
                "##,
            r##"#version 300 es
                precision highp float;
                out vec4 outColor;
                in vec2 cornerPos;
                in vec2 curSize;

                float cornerSize = 0.3;
                float fuzziness = 0.03; // A form of anti-aliasing by making the circle border a slight gradient
                
                void main() {
                    float alpha = 1.0;
                    float cornerSize2 = cornerSize * cornerSize;

                    float xBoundary = curSize.x / 2.0 - cornerSize;
                    float yBoundary = curSize.y / 2.0 - cornerSize;
                    float absX = abs(cornerPos.x);
                    float absY = abs(cornerPos.y);
                    if (absX > xBoundary && absY > yBoundary) {
                        float dx = xBoundary - absX;
                        float dy = yBoundary - absY;
                        float distance2 = dx*dx + dy*dy;
                        if(distance2 >= cornerSize2)
                            alpha = 1.0 - max(0.0, (sqrt(distance2) - cornerSize) / fuzziness);
                    }

                    outColor = vec4(1, 1, 1, alpha);
                }
                "##,
        )
        .unwrap();
        NodeRenderer { vertex_renderer }
    }

    pub fn set_nodes(&mut self, context: &WebGl2RenderingContext, nodes: &Vec<Node>) {
        fn map<const LEN: usize>(
            nodes: &Vec<Node>,
            map: impl Fn(&Node) -> [f32; LEN],
        ) -> Box<[f32]> {
            nodes
                .iter()
                .flat_map(|node| map(node).repeat(6))
                // .flat_map(|node| [0.0, 0.0].repeat(4))
                .collect()
        }

        let old_positions = map(nodes, |node| {
            [node.center_position.old.x, node.center_position.old.y]
        });
        self.vertex_renderer
            .set_data(context, "positionOld", &old_positions, 2);

        let positions = map(nodes, |node| {
            [node.center_position.new.x, node.center_position.new.y]
        });
        self.vertex_renderer
            .set_data(context, "position", &positions, 2);

        let position_old_times = map(nodes, |node| [node.center_position.old_time as f32]);
        self.vertex_renderer
            .set_data(context, "positionStartTime", &position_old_times, 1);

        let position_durations = map(nodes, |node| [node.center_position.duration as f32]);
        self.vertex_renderer
            .set_data(context, "positionDuration", &position_durations, 1);

        let old_sizes = map(nodes, |node| [node.size.old.x, node.size.old.y]);
        self.vertex_renderer
            .set_data(context, "sizeOld", &old_sizes, 2);

        let sizes = map(nodes, |node| [node.size.new.x, node.size.new.y]);
        self.vertex_renderer.set_data(context, "size", &sizes, 2);

        let size_old_times = map(nodes, |node| [node.size.old_time as f32]);
        self.vertex_renderer
            .set_data(context, "sizeStartTime", &size_old_times, 1);

        let size_durations = map(nodes, |node| [node.size.duration as f32]);
        self.vertex_renderer
            .set_data(context, "sizeDuration", &size_durations, 1);

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
        self.vertex_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);
    }

    pub fn dispose(&self, context: &WebGl2RenderingContext) {
        self.vertex_renderer.dispose(context);
    }
}
