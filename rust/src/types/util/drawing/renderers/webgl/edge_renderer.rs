use std::{
    collections::{HashMap, HashSet},
    iter::repeat,
};

use itertools::Itertools;
use multimap::MultiMap;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    types::util::drawing::{
        diagram_layout::{Point, Transition},
        renderer::GroupSelection,
        renderers::webgl::util::set_animated_data::{self, set_animated_data},
    },
    util::{logging::console, matrix4::Matrix4},
    wasm_interface::NodeGroupID,
};

use super::util::vertex_renderer::VertexRenderer;

pub struct EdgeRenderer {
    vertex_renderer: VertexRenderer,
    edge_types: Vec<EdgeRenderingType>,
    node_edge_indices: MultiMap<NodeGroupID, usize>,
}

pub struct Edge {
    pub start: Transition<Point>,
    pub start_node: NodeGroupID,
    pub points: Vec<Transition<Point>>,
    pub end: Transition<Point>,
    pub end_node: NodeGroupID,
    pub exists: Transition<f32>,
    pub edge_type: usize,
    pub shift: Transition<f32>, // Some sideways shift
}

#[derive(Clone)]
pub struct EdgeRenderingType {
    pub color: (f32, f32, f32),
    pub select_color: (f32, f32, f32),
    pub partial_select_color: (f32, f32, f32),
    pub hover_color: (f32, f32, f32),
    pub partial_hover_color: (f32, f32, f32),
    pub width: f32,
    pub dash_solid: f32, // The distance per period over which this dash should be solid
    pub dash_transparent: f32, // The distance per
}
type Segment = (
    Transition<Point>,
    Transition<Point>,
    f32,             /* type*/
    Transition<f32>, /* curvature */
    Transition<f32>, /* exists */
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
            node_edge_indices: MultiMap::new(),
        }
    }

    pub fn set_edges(&mut self, context: &WebGl2RenderingContext, edges: &Vec<Edge>) {
        let segments = edges
            .iter()
            .flat_map(|edge| {
                let mut points = edge.points.clone();
                let edge_type = edge.edge_type;
                let curve_offset = edge.shift;
                let exists = edge.exists;
                points.push(edge.end);
                let edge_segments = points
                    .iter()
                    .scan(edge.start, |prev, item| {
                        let out = (*prev, *item, edge_type as f32, curve_offset, exists);
                        *prev = *item;
                        Some((out, edge))
                    })
                    .collect::<Vec<_>>();
                edge_segments
            })
            .collect::<Vec<(Segment, &Edge)>>();

        self.node_edge_indices = segments
            .iter()
            .enumerate()
            .flat_map(|(index, (segment, edge))| [(edge.start_node, index), (edge.end_node, index)])
            .collect();

        let segments6 = segments.iter().flat_map(|(edge, _)| repeat(edge).take(6));
        set_animated_data(
            "start",
            segments6.clone().map(|(start, _, _, _, _)| start.clone()),
            |start| [start.x, start.y],
            context,
            &mut self.vertex_renderer,
        );
        set_animated_data(
            "end",
            segments6.clone().map(|(_, end, _, _, _)| end.clone()),
            |end| [end.x, end.y],
            context,
            &mut self.vertex_renderer,
        );
        set_animated_data(
            "curveOffset",
            segments6.clone().map(|(_, _, _, offset, _)| offset.clone()),
            |offset| [offset],
            context,
            &mut self.vertex_renderer,
        );
        set_animated_data(
            "exists",
            segments6.clone().map(|(_, _, _, _, exists)| exists.clone()),
            |exists| [exists],
            context,
            &mut self.vertex_renderer,
        );

        self.vertex_renderer.set_data(
            context,
            "type",
            &segments6
                .clone()
                .map(|(_, _, edge_type, _, _)| edge_type.clone())
                .collect::<Box<_>>(),
            1,
        );
        self.vertex_renderer.set_data(
            context,
            "state",
            &segments6.map(|_| 0.).collect::<Box<_>>(),
            1,
        );

        self.vertex_renderer.send_data(context);
    }

    pub fn set_transform(&mut self, context: &WebGl2RenderingContext, transform: &Matrix4) {
        self.vertex_renderer.set_uniform(context, "transform", |u| {
            context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
        });
    }

    pub fn update_selection(
        &mut self,
        context: &WebGl2RenderingContext,
        selection: &GroupSelection,
        old_selection: &GroupSelection,
    ) {
        let to_indices = |ids: &[NodeGroupID]| {
            ids.iter()
                .filter_map(|id| self.node_edge_indices.get_vec(&(*id as usize)))
                .flatten()
                .cloned()
                .collect::<HashSet<usize>>()
        };

        let new_selected_indices = to_indices(selection.0);
        let new_partially_selected_indices = to_indices(selection.1);
        let new_hover_indices = to_indices(selection.2);
        let new_partially_hover_indices = to_indices(selection.3);
        let old_selected_indices = to_indices(old_selection.0);
        let old_partially_selected_indices = to_indices(old_selection.1);
        let old_hover_indices = to_indices(old_selection.2);
        let old_partially_hover_indices = to_indices(old_selection.3);

        let indices = new_selected_indices
            .iter()
            .chain(old_selected_indices.iter())
            .chain(new_partially_selected_indices.iter())
            .chain(old_partially_selected_indices.iter())
            .chain(new_hover_indices.iter())
            .chain(old_hover_indices.iter())
            .chain(new_partially_hover_indices.iter())
            .chain(old_partially_hover_indices.iter());

        let state_updates = indices.filter_map(|index| {
            let new_state = if new_selected_indices.contains(&index) {
                4
            } else if new_partially_selected_indices.contains(&index) {
                3
            } else if new_hover_indices.contains(&index) {
                2
            } else if new_partially_hover_indices.contains(&index) {
                1
            } else {
                0
            };

            let old_state = if old_selected_indices.contains(&index) {
                4
            } else if old_partially_selected_indices.contains(&index) {
                3
            } else if old_hover_indices.contains(&index) {
                2
            } else if old_partially_hover_indices.contains(&index) {
                1
            } else {
                0
            };

            if new_state != old_state {
                Some((index, new_state))
            } else {
                None
            }
        });

        for (index, state) in state_updates {
            let data_index = index * 6;
            for i in 0..6 {
                self.vertex_renderer
                    .update_data(context, "state", data_index + i, [state as f32]);
            }
        }
        self.vertex_renderer.send_data(context);
    }

    pub fn render(&mut self, context: &WebGl2RenderingContext, time: u32) {
        self.vertex_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));
        for (index, edge_type) in self.edge_types.iter().enumerate() {
            let c = edge_type.color;
            self.vertex_renderer
                .set_uniform(context, &format!("edgeTypes[{index}].color"), |u| {
                    context.uniform3f(u, c.0, c.1, c.2)
                });
            let c = edge_type.hover_color;
            self.vertex_renderer.set_uniform(
                context,
                &format!("edgeTypes[{index}].hoverColor"),
                |u| context.uniform3f(u, c.0, c.1, c.2),
            );
            let c = edge_type.select_color;
            self.vertex_renderer.set_uniform(
                context,
                &format!("edgeTypes[{index}].selectColor"),
                |u| context.uniform3f(u, c.0, c.1, c.2),
            );
            let c = edge_type.partial_hover_color;
            self.vertex_renderer.set_uniform(
                context,
                &format!("edgeTypes[{index}].partialHoverColor"),
                |u| context.uniform3f(u, c.0, c.1, c.2),
            );
            let c = edge_type.partial_select_color;
            self.vertex_renderer.set_uniform(
                context,
                &format!("edgeTypes[{index}].partialSelectColor"),
                |u| context.uniform3f(u, c.0, c.1, c.2),
            );
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
