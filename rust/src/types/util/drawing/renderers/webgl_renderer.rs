use oxidd_core::Tag;
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader,
    WebGlVertexArrayObject,
};

use crate::{
    types::util::drawing::{
        diagram_layout::{DiagramLayout, Point},
        renderer::Renderer,
        renderers::webgl::{
            setup::{compile_shader, link_program},
            vertex_renderer::VertexRenderer,
        },
    },
    util::transformation::Transformation,
};

use super::webgl::{
    edge_renderer::{Edge, EdgeRenderer},
    node_renderer::{Node, NodeRenderer},
};

/// A simple renderer that uses webgl to draw nodes and edges
pub struct WebglRenderer {
    webgl_context: WebGl2RenderingContext,
    node_renderer: NodeRenderer,
    edge_renderer: EdgeRenderer,
}

impl WebglRenderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<WebglRenderer, JsValue> {
        Ok(WebglRenderer {
            node_renderer: NodeRenderer::new(&context),
            edge_renderer: EdgeRenderer::new(&context),
            webgl_context: context,
        })
    }
    pub fn from_canvas(canvas: HtmlCanvasElement) -> Result<WebglRenderer, JsValue> {
        let context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();
        context.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        context.enable(WebGl2RenderingContext::BLEND);
        WebglRenderer::new(context)
    }
}

impl<T: Tag> Renderer<T> for WebglRenderer {
    fn set_transform(&mut self, transform: Transformation) {
        let matrix = transform.get_matrix();
        self.node_renderer
            .set_transform(&self.webgl_context, &matrix);
        self.edge_renderer
            .set_transform(&self.webgl_context, &matrix);
    }
    fn update_layout(&mut self, layout: &DiagramLayout<T>) {
        self.node_renderer.set_nodes(
            &self.webgl_context,
            &layout
                .groups
                .values()
                .map(|group| Node {
                    center_position: group.center_position,
                    size: group.size,
                    label: group.label.clone(),
                    exists: group.exists,
                })
                .collect(),
        );
        self.edge_renderer.set_edges(
            &self.webgl_context,
            &layout
                .groups
                .values()
                .flat_map(|group| {
                    let start = group.center_position;
                    group.edges.iter().flat_map(move |(to_id, edges)| {
                        edges.iter().map(move |(edge_type, edge)| Edge {
                            start: start + edge.start_offset,
                            points: edge.points.iter().map(|point| point.point).collect(),
                            end: layout.groups.get(to_id).unwrap().center_position
                                + edge.end_offset,
                        })
                    })
                })
                .collect(),
        );
    }
    fn render(&mut self, time: u32, selected_ids: &[u32], hovered_ids: &[u32]) {
        self.webgl_context.clear_color(1.0, 1.0, 1.0, 1.0);
        self.webgl_context
            .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        self.node_renderer
            .render(&self.webgl_context, time, selected_ids, hovered_ids);
        self.edge_renderer
            .render(&self.webgl_context, time, selected_ids, hovered_ids);
    }
}

impl Drop for WebglRenderer {
    fn drop(&mut self) {
        self.node_renderer.dispose(&self.webgl_context);
        self.edge_renderer.dispose(&self.webgl_context);
    }
}
