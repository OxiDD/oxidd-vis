use std::{
    collections::{HashMap, HashSet},
    iter::{repeat, FromIterator, Repeat},
    rc::Rc,
};

use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::{
        renderer::GroupSelection,
        renderers::{util::Font::Font, webgl::util::set_animated_data::set_animated_data},
    },
    util::{
        color::{Color, TransparentColor},
        logging::console,
        matrix4::Matrix4,
        point::Point,
        transition::{Interpolatable, Transition},
    },
    wasm_interface::NodeGroupID,
};

use super::{
    text::text_renderer::{Text, TextRenderer, TextRendererSettings},
    util::vertex_renderer::VertexRenderer,
};

pub struct NodeRenderer {
    vertex_renderer: VertexRenderer,
    outline_vertex_renderer: VertexRenderer,
    text_renderer: TextRenderer,
    font: Rc<Font>,
    node_indices: HashMap<NodeGroupID, NodeData>,
    colors: NodeRenderingColorConfig,
}
pub struct NodeData {
    index: usize,
    color: Color,
}

#[derive(Clone)]
pub struct Node {
    pub ID: NodeGroupID,
    pub center_position: Transition<Point>,
    pub size: Transition<Point>,
    pub color: Transition<Color>,
    pub outline_color: Transition<TransparentColor>,
    pub label: Option<String>,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
}

pub struct NodeRenderingColorConfig {
    pub select: (Color, f32),
    pub partial_select: (Color, f32),
    pub hover: (Color, f32),
    pub partial_hover: (Color, f32),
}

pub struct TextRenderingConfig {
    pub screen_height: usize,
    pub font: Rc<Font>,
    pub font_settings: TextRendererSettings,
}

impl NodeRenderer {
    pub fn new(
        context: &WebGl2RenderingContext,
        colors: NodeRenderingColorConfig,
        text: TextRenderingConfig,
    ) -> NodeRenderer {
        let vertex_renderer = VertexRenderer::new(
            context,
            include_str!("node_renderer.vert"),
            include_str!("node_renderer.frag"),
        )
        .unwrap();
        let outline_vertex_renderer = VertexRenderer::new(
            context,
            include_str!("node_outline.vert"),
            include_str!("node_outline.frag"),
        )
        .unwrap();
        NodeRenderer {
            vertex_renderer,
            outline_vertex_renderer,
            node_indices: HashMap::new(),
            colors,
            font: text.font.clone(),
            text_renderer: TextRenderer::new(
                context,
                text.font,
                text.font_settings,
                text.screen_height,
            ),
        }
    }

    pub fn set_nodes(&mut self, context: &WebGl2RenderingContext, nodes: &Vec<Node>) {
        self.node_indices = nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                (
                    node.ID,
                    NodeData {
                        index: index,
                        color: node.color.new.clone(),
                    },
                )
            })
            .collect();

        // Node shape
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
            "exists",
            nodes6.clone().map(|n| n.exists),
            |v| [v],
            context,
            &mut self.vertex_renderer,
        );
        set_animated_data(
            "color",
            nodes6.map(|n| n.color.clone()),
            |v| [v.0, v.1, v.2],
            context,
            &mut self.vertex_renderer,
        );
        self.vertex_renderer.send_data(context);

        // Outline shape
        let outline_nodes = nodes
            .iter()
            .filter(|node| node.outline_color.new.3 != 0. || node.outline_color.old.3 != 0.);
        let outline_nodes6 = outline_nodes.flat_map(|node| repeat(node).take(6));
        set_animated_data(
            "position",
            outline_nodes6.clone().map(|n| n.center_position),
            |v| [v.x, v.y],
            context,
            &mut self.outline_vertex_renderer,
        );
        set_animated_data(
            "size",
            outline_nodes6.clone().map(|n| n.size),
            |v| [v.x, v.y],
            context,
            &mut self.outline_vertex_renderer,
        );
        set_animated_data(
            "exists",
            outline_nodes6.clone().map(|n| n.exists),
            |v| [v],
            context,
            &mut self.outline_vertex_renderer,
        );
        set_animated_data(
            "color",
            outline_nodes6.map(|n| n.outline_color.clone()),
            |v| [v.0, v.1, v.2, v.3],
            context,
            &mut self.outline_vertex_renderer,
        );
        self.outline_vertex_renderer.send_data(context);

        // Text
        self.text_renderer.set_texts(
            context,
            &nodes
                .iter()
                .filter_map(|node| {
                    node.label.clone().map(|text| {
                        let text_width = self.font.measure_width(&text);
                        let text_height = self.font.measure_height(&text);
                        Text {
                            text,
                            position: &node.center_position
                                + &Transition {
                                    old_time: node.size.old_time,
                                    duration: node.size.duration,
                                    old: Point {
                                        x: -0.5 * text_width,
                                        y: -0.5 * text_height,
                                    },
                                    new: Point {
                                        x: -0.5 * text_width,
                                        y: -0.5 * text_height,
                                    },
                                },
                            exists: node.exists,
                        }
                    })
                })
                .collect(),
        );
    }

    pub fn update_selection(
        &mut self,
        context: &WebGl2RenderingContext,
        selection: &GroupSelection,
        old_selection: &GroupSelection,
    ) {
        let select_color = self.colors.select.clone();
        let partial_select_color = self.colors.partial_select.clone();
        let hover_color = self.colors.hover.clone();
        let partial_hover_color = self.colors.partial_hover.clone();

        let ids = selection
            .0
            .iter()
            .chain(selection.1.iter())
            .chain(selection.2.iter())
            .chain(selection.3.iter())
            .chain(old_selection.0.iter())
            .chain(old_selection.1.iter())
            .chain(old_selection.2.iter())
            .chain(old_selection.3.iter());

        let new_select: HashSet<NodeGroupID> = selection.0.iter().cloned().collect();
        let new_partial_select: HashSet<NodeGroupID> = selection.1.iter().cloned().collect();
        let new_hover: HashSet<NodeGroupID> = selection.2.iter().cloned().collect();
        let new_partial_hover: HashSet<NodeGroupID> = selection.3.iter().cloned().collect();
        let old_select: HashSet<NodeGroupID> = old_selection.0.iter().cloned().collect();
        let old_partial_select: HashSet<NodeGroupID> = old_selection.1.iter().cloned().collect();
        let old_hover: HashSet<NodeGroupID> = old_selection.2.iter().cloned().collect();
        let old_partial_hover: HashSet<NodeGroupID> = old_selection.3.iter().cloned().collect();

        let color_updates = ids.filter_map(|id| {
            let new_color = if new_select.contains(&id) {
                Some(select_color.clone())
            } else if new_partial_select.contains(&id) {
                Some(partial_select_color.clone())
            } else if new_hover.contains(&id) {
                Some(hover_color.clone())
            } else if new_partial_hover.contains(&id) {
                Some(partial_hover_color.clone())
            } else {
                None
            };

            let old_color = if old_select.contains(&id) {
                Some(select_color.clone())
            } else if old_partial_select.contains(&id) {
                Some(partial_select_color.clone())
            } else if old_hover.contains(&id) {
                Some(hover_color.clone())
            } else if old_partial_hover.contains(&id) {
                Some(partial_hover_color.clone())
            } else {
                None
            };

            if new_color != old_color {
                Some((id, new_color))
            } else {
                None
            }
        });

        for (id, maybe_color) in color_updates {
            if let Some(node_data) = self.node_indices.get(id) {
                let data_index = node_data.index * 6;
                let node_color = maybe_color
                    .clone()
                    .map(|(color, per)| node_data.color.mix(&color, per))
                    .unwrap_or_else(|| node_data.color.clone());
                let old_node_color = maybe_color
                    .map(|(color, per)| node_data.color.mix(&color, per))
                    .unwrap_or_else(|| node_data.color.clone());
                for i in 0..6 {
                    self.vertex_renderer.update_data(
                        context,
                        "color",
                        data_index + i,
                        [node_color.0, node_color.1, node_color.2],
                    );
                    self.vertex_renderer.update_data(
                        context,
                        "colorOld",
                        data_index + i,
                        [old_node_color.0, old_node_color.1, old_node_color.2],
                    );
                }
            }
        }
        self.vertex_renderer.send_data(context);
    }

    pub fn set_transform_and_screen_height(
        &mut self,
        context: &WebGl2RenderingContext,
        transform: &Matrix4,
        screen_height: usize,
    ) {
        self.vertex_renderer.set_uniform(context, "transform", |u| {
            context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
        });
        self.outline_vertex_renderer
            .set_uniform(context, "transform", |u| {
                context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
            });

        self.text_renderer
            .set_transform_and_screen_height(context, transform, screen_height);
    }

    pub fn render(&mut self, context: &WebGl2RenderingContext, time: u32) {
        // TODO: add configuration
        let corner_radius = 0.3;
        let border_offset = 0.3;
        let border_width = 0.2;

        self.vertex_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));
        self.vertex_renderer
            .set_uniform(context, "selection", |u| context.uniform1f(u, time as f32));
        self.vertex_renderer
            .set_uniform(context, "cornerSize", |u| {
                context.uniform1f(u, corner_radius)
            });
        self.vertex_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);

        self.outline_vertex_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));
        self.outline_vertex_renderer
            .set_uniform(context, "selection", |u| context.uniform1f(u, time as f32));
        self.outline_vertex_renderer
            .set_uniform(context, "cornerSize", |u| {
                context.uniform1f(u, corner_radius)
            });
        self.outline_vertex_renderer
            .set_uniform(context, "width", |u| context.uniform1f(u, border_width));
        self.outline_vertex_renderer
            .set_uniform(context, "offset", |u| context.uniform1f(u, border_offset));
        self.outline_vertex_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);

        self.text_renderer.render(context, time);
    }

    pub fn dispose(&mut self, context: &WebGl2RenderingContext) {
        self.vertex_renderer.dispose(context);
        self.outline_vertex_renderer.dispose(context);
        self.text_renderer.dispose(context);
    }
}
