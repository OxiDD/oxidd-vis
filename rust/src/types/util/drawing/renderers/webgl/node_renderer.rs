use std::{
    collections::{HashMap, HashSet},
    iter::{repeat, FromIterator, Repeat},
};

use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::{
        diagram_layout::{Point, Transition},
        layouts::util::color_label::{Color, TransparentColor},
        renderer::GroupSelection,
        renderers::webgl::util::set_animated_data::set_animated_data,
    },
    util::{logging::console, matrix4::Matrix4},
    wasm_interface::NodeGroupID,
};

use super::util::{mix_color::mix_color, vertex_renderer::VertexRenderer};

pub struct NodeRenderer {
    vertex_renderer: VertexRenderer,
    outline_vertex_renderer: VertexRenderer,
    node_indices: HashMap<NodeGroupID, NodeData>,
    hover_color: (Color, f32),
    select_color: (Color, f32),
    partial_hover_color: (Color, f32),
    partial_select_color: (Color, f32),
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
    pub label: String,
    pub exists: Transition<f32>, // A number between 0 and 1 of whether this node is visible (0-1)
}

impl NodeRenderer {
    pub fn new(
        context: &WebGl2RenderingContext,
        select_color: (Color, f32),
        partial_select_color: (Color, f32),
        hover_color: (Color, f32),
        partial_hover_color: (Color, f32),
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
            hover_color,
            select_color,
            partial_hover_color,
            partial_select_color,
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
            nodes6.map(|n| n.color),
            |v| [v.0, v.1, v.2],
            context,
            &mut self.vertex_renderer,
        );
        self.vertex_renderer.send_data(context);

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
            outline_nodes6.map(|n| n.outline_color),
            |v| [v.0, v.1, v.2, v.3],
            context,
            &mut self.outline_vertex_renderer,
        );
        self.outline_vertex_renderer.send_data(context);
    }

    pub fn update_selection(
        &mut self,
        context: &WebGl2RenderingContext,
        selection: &GroupSelection,
        old_selection: &GroupSelection,
    ) {
        let select_color = self.select_color.clone();
        let partial_select_color = self.partial_select_color.clone();
        let hover_color = self.hover_color.clone();
        let partial_hover_color = self.partial_hover_color.clone();

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
                Some(select_color)
            } else if new_partial_select.contains(&id) {
                Some(partial_select_color)
            } else if new_hover.contains(&id) {
                Some(hover_color)
            } else if new_partial_hover.contains(&id) {
                Some(partial_hover_color)
            } else {
                None
            };

            let old_color = if old_select.contains(&id) {
                Some(select_color)
            } else if old_partial_select.contains(&id) {
                Some(partial_select_color)
            } else if old_hover.contains(&id) {
                Some(hover_color)
            } else if old_partial_hover.contains(&id) {
                Some(partial_hover_color)
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
                    .map(|(color, per)| mix_color(node_data.color, color, per))
                    .unwrap_or(node_data.color);
                let old_node_color = maybe_color
                    .map(|(color, per)| mix_color(node_data.color, color, per))
                    .unwrap_or(node_data.color);
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

    pub fn set_transform(&mut self, context: &WebGl2RenderingContext, transform: &Matrix4) {
        self.vertex_renderer.set_uniform(context, "transform", |u| {
            context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
        });
        self.outline_vertex_renderer
            .set_uniform(context, "transform", |u| {
                context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
            });
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
    }

    pub fn dispose(&mut self, context: &WebGl2RenderingContext) {
        self.vertex_renderer.dispose(context);
    }
}
