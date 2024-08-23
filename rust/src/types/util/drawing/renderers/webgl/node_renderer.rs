use std::{
    collections::{HashMap, HashSet},
    iter::{repeat, FromIterator, Repeat},
};

use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::{
        diagram_layout::{Point, Transition},
        layouts::util::color_label::Color,
        renderers::webgl::util::set_animated_data::set_animated_data,
    },
    util::{logging::console, matrix4::Matrix4},
    wasm_interface::NodeGroupID,
};

use super::util::{mix_color::mix_color, vertex_renderer::VertexRenderer};

pub struct NodeRenderer {
    vertex_renderer: VertexRenderer,
    node_indices: HashMap<u32, NodeData>,
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
        NodeRenderer {
            vertex_renderer,
            node_indices: HashMap::new(),
        }
    }

    pub fn set_nodes(&mut self, context: &WebGl2RenderingContext, nodes: &Vec<Node>) {
        self.node_indices = nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                (
                    node.ID as u32,
                    NodeData {
                        index: index,
                        color: node.color.new.clone(),
                    },
                )
            })
            .collect();

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

        self.vertex_renderer.send_data(context);
    }

    pub fn update_selection(
        &mut self,
        context: &WebGl2RenderingContext,
        selected_ids: &[u32],
        prev_selected_ids: &[u32],
        hover_ids: &[u32],
        prev_hover_ids: &[u32],
    ) {
        let select_color = ((0.0, 0.0, 1.0), 0.8);
        let hover_color = ((0.0, 0.0, 1.0), 0.3);

        let ids = selected_ids
            .iter()
            .chain(prev_selected_ids.iter())
            .chain(hover_ids.iter())
            .chain(prev_hover_ids.iter());

        let new_select: HashSet<u32> = selected_ids.iter().cloned().collect();
        let new_hover: HashSet<u32> = hover_ids.iter().cloned().collect();
        let old_select: HashSet<u32> = prev_selected_ids.iter().cloned().collect();
        let old_hover: HashSet<u32> = prev_hover_ids.iter().cloned().collect();

        let color_updates = ids.filter_map(|id| {
            let new_color = if new_select.contains(&id) {
                Some(select_color)
            } else if new_hover.contains(&id) {
                Some(hover_color)
            } else {
                None
            };

            let old_color = if old_select.contains(&id) {
                Some(select_color)
            } else if old_hover.contains(&id) {
                Some(hover_color)
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
                for i in 0..6 {
                    self.vertex_renderer.update_data(
                        context,
                        "color",
                        data_index + i,
                        [node_color.0, node_color.1, node_color.2],
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
    }

    pub fn render(&mut self, context: &WebGl2RenderingContext, time: u32) {
        self.vertex_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));
        self.vertex_renderer
            .set_uniform(context, "selection", |u| context.uniform1f(u, time as f32));
        self.vertex_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);
    }

    pub fn dispose(&mut self, context: &WebGl2RenderingContext) {
        self.vertex_renderer.dispose(context);
    }
}
