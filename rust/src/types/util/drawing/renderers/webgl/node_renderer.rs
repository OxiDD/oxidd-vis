use std::iter::repeat;

use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::{
        diagram_layout::{Point, Transition},
        renderers::webgl::set_animated_data::set_animated_data,
    },
    util::{logging::console, matrix4::Matrix4},
};

use super::vertex_renderer::VertexRenderer;

pub struct NodeRenderer {
    vertex_renderer: VertexRenderer,
}

#[derive(Clone)]
pub struct Node {
    pub center_position: Transition<Point>,
    pub size: Transition<Point>,
    pub color: Transition<(f32, f32, f32)>,
    pub label: String,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
}

impl NodeRenderer {
    pub fn new(context: &WebGl2RenderingContext) -> NodeRenderer {
        let vertex_renderer = VertexRenderer::new(
            context,
            include_str!("node_renderer.vert"),
            include_str!("node_renderer.frag"),
        )
        .unwrap();
        NodeRenderer { vertex_renderer }
    }

    pub fn set_nodes(&mut self, context: &WebGl2RenderingContext, nodes: &Vec<Node>) {
        fn map<const LEN: usize>(
            nodes: &Vec<Node>,
            map: impl Fn(&Node) -> [f32; LEN],
        ) -> Box<[f32]> {
            nodes.iter().flat_map(|node| map(node).repeat(6)).collect()
        }

        let nodes6 = nodes.iter().flat_map(|node| repeat(node).take(6));
        set_animated_data(
            "position",
            nodes6.clone().map(|n| n.center_position),
            |v| [v.x, v.y],
            context,
            &mut self.vertex_renderer,
        );

        set_animated_data(
            "size",
            nodes6.clone().map(|n| n.size),
            |v| [v.x, v.y],
            context,
            &mut self.vertex_renderer,
        );

        set_animated_data(
            "color",
            nodes6.map(|n| n.color),
            |v| [v.0, v.1, v.2],
            context,
            &mut self.vertex_renderer,
        );

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

    pub fn dispose(&mut self, context: &WebGl2RenderingContext) {
        self.vertex_renderer.dispose(context);
    }
}
