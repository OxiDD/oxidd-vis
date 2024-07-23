use std::collections::HashMap;

use oxidd_core::Tag;
use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader,
    WebGlVertexArrayObject,
};

use crate::{
    types::util::{
        drawing::{
            diagram_layout::{DiagramLayout, Point, Transition},
            renderer::Renderer,
            renderers::webgl::{
                setup::{compile_shader, link_program},
                vertex_renderer::VertexRenderer,
            },
        },
        edge_type::EdgeType,
    },
    util::{logging::console, transformation::Transformation},
};

use super::webgl::{
    edge_renderer::{Edge, EdgeRenderer, EdgeRenderingType},
    layers::{
        layer_bg_renderer::LayerBgRenderer,
        layer_lines_renderer::LayerLinesRenderer,
        layer_renderer::{Layer, LayerRenderer},
    },
    node_renderer::{Node, NodeRenderer},
    render_texture::{RenderTarget, ScreenTexture},
    text::text_renderer::{Text, TextRenderer, TextRendererSettings},
};

/// A simple renderer that uses webgl to draw nodes and edges
pub struct WebglRenderer<T: Tag> {
    webgl_context: WebGl2RenderingContext,
    node_renderer: NodeRenderer,
    edge_renderer: EdgeRenderer,
    layer_renderer: LayerRenderer,
    edge_type_ids: HashMap<EdgeType<T>, usize>,
    screen_texture: ScreenTexture,
}

impl<T: Tag> WebglRenderer<T> {
    pub fn new(
        context: WebGl2RenderingContext,
        screen_texture: ScreenTexture,
        edge_types: HashMap<EdgeType<T>, EdgeRenderingType>,
    ) -> Result<WebglRenderer<T>, JsValue> {
        let (edge_type_ids, edge_rendering_types): (
            HashMap<EdgeType<T>, usize>,
            Vec<EdgeRenderingType>,
        ) = edge_types
            .iter()
            .enumerate()
            .map(|(index, (edge_type, edge_rendering))| {
                ((edge_type.clone(), index), edge_rendering.clone())
            })
            .unzip();
        Ok(WebglRenderer {
            node_renderer: NodeRenderer::new(&context),
            edge_renderer: EdgeRenderer::new(&context, edge_rendering_types),
            layer_renderer: LayerRenderer::new(
                &context,
                LayerBgRenderer::new(&context),
                // LayerLinesRenderer::new(&context),
                screen_texture.get_size().1,
                include_bytes!("../../../../../resources/Roboto-Bold.ttf").to_vec(),
                TextRendererSettings::new()
                    .resolution(1.5)
                    .sample_distance(35.)
                    .scale_factor_group_size(3.0)
                    .scale_cache_size(10) // Very large, mostly for testing
                    .max_scale(1.5),
            ),
            webgl_context: context,
            screen_texture,
            edge_type_ids,
        })
    }
    pub fn from_canvas(
        canvas: HtmlCanvasElement,
        edge_types: HashMap<EdgeType<T>, EdgeRenderingType>,
    ) -> Result<WebglRenderer<T>, JsValue> {
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
        WebglRenderer::new(
            context,
            ScreenTexture::new(
                canvas.width() as usize,
                canvas.height() as usize,
                (1.0, 1.0, 1.0, 1.0),
            ),
            edge_types,
        )
    }
}

impl<T: Tag> Renderer<T> for WebglRenderer<T> {
    fn set_transform(&mut self, transform: Transformation) {
        let height = transform.height as usize;
        // if self.screen_texture.get_size().1 != height {
        //     self.layer_renderer
        //         .set_screen_height(&self.webgl_context, height);
        // }

        self.screen_texture
            .setSize(transform.width as usize, height);
        let matrix = transform.get_matrix();
        self.node_renderer
            .set_transform(&self.webgl_context, &matrix);
        self.edge_renderer
            .set_transform(&self.webgl_context, &matrix);
        self.layer_renderer
            .set_transform_and_screen_height(&self.webgl_context, &matrix, height);
    }
    fn update_layout(&mut self, layout: &DiagramLayout<T>) {
        self.node_renderer.set_nodes(
            &self.webgl_context,
            &layout
                .groups
                .values()
                .map(|group| Node {
                    center_position: group.position + group.size * 0.5,
                    size: group.size,
                    label: group.label.clone(),
                    exists: group.exists,
                })
                .collect(),
        );
        let edge_type_ids = self.edge_type_ids.clone();
        self.edge_renderer.set_edges(
            &self.webgl_context,
            &layout
                .groups
                .values()
                .flat_map(|group| {
                    let start = group.position;
                    let edge_type_ids = &edge_type_ids;
                    group.edges.iter().map(move |(edge_data, edge)| Edge {
                        start: start + edge.start_offset,
                        points: edge.points.iter().map(|point| point.point).collect(),
                        end: layout.groups.get(&edge_data.to).unwrap().position + edge.end_offset,
                        edge_type: *edge_type_ids.get(&edge_data.edge_type).unwrap(),
                        shift: edge.curve_offset,
                    })
                })
                .collect(),
        );
        self.layer_renderer.set_layers(
            &self.webgl_context,
            &layout
                .layers
                .iter()
                .map(|layer| Layer {
                    top: layer.top,
                    bottom: layer.bottom,
                    label: layer.label.clone(),
                    index: layer.index,
                    exists: layer.exists,
                })
                .collect(),
        );
    }
    fn render(&mut self, time: u32, selected_ids: &[u32], hovered_ids: &[u32]) {
        self.screen_texture.clear(&self.webgl_context);
        self.layer_renderer
            .render(&self.webgl_context, time, selected_ids, hovered_ids);
        self.edge_renderer
            .render(&self.webgl_context, time, selected_ids, hovered_ids);
        self.node_renderer
            .render(&self.webgl_context, time, selected_ids, hovered_ids);
    }
}

impl<T: Tag> Drop for WebglRenderer<T> {
    fn drop(&mut self) {
        self.node_renderer.dispose(&self.webgl_context);
        self.edge_renderer.dispose(&self.webgl_context);
        self.layer_renderer.dispose(&self.webgl_context);
    }
}
