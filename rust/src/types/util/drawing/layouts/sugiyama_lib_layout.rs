use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::{
            diagram_layout::{DiagramLayout, Point},
            layout_rules::LayoutRules,
        },
        grouped_graph_structure::GroupedGraphStructure,
    },
    util::logging::console,
    wasm_interface::NodeGroupID,
};
use rust_sugiyama::from_edges;
use std::convert::TryInto;

use super::{
    layer_orderings::dummy_layer_ordering::DummyLayerOrdering,
    layered_layout::{LayerOrdering, LayeredLayout, NodePositioning},
    util::layered::layer_orderer::{EdgeMap, Order},
};

pub struct SugiyamaLibLayout<T: Tag> {
    layout: LayeredLayout<T>,
}
impl<T: Tag> SugiyamaLibLayout<T> {
    pub fn new() -> SugiyamaLibLayout<T> {
        SugiyamaLibLayout {
            layout: LayeredLayout::new(
                Box::new(DummyLayerOrdering),
                Box::new(SugiyamaLibPositioning),
            ),
        }
    }
}

impl<T: Tag> LayoutRules<T> for SugiyamaLibLayout<T> {
    fn layout(
        &mut self,
        graph: &dyn GroupedGraphStructure<T>,
        old: &DiagramLayout<T>,
        time: u32,
    ) -> DiagramLayout<T> {
        self.layout.layout(graph, old, time)
    }
}
struct SugiyamaLibPositioning;
impl<T: Tag> NodePositioning<T> for SugiyamaLibPositioning {
    fn position_nodes(
        &self,
        graph: &dyn GroupedGraphStructure<T>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        dummy_start_id: NodeGroupID,
    ) -> std::collections::HashMap<NodeGroupID, Point> {
        // console::log!(
        //     "{}",
        //     edges
        //         .iter()
        //         .flat_map(|(from, tos)| tos
        //             .iter()
        //             .map(|to| format!("{}->{}", from, to))
        //             .collect::<Vec<String>>())
        //         .collect::<Vec<String>>()
        //         .join(", ")
        // );
        let layouts = from_edges(
            &edges
                .iter()
                .flat_map(|(from, to_set)| to_set.iter().map(move |to| (*from as u32, *to as u32)))
                .collect::<Vec<(u32, u32)>>()[..],
        )
        .vertex_spacing(2)
        .transpose(true)
        .build();

        // console::log!(
        //     "{}",
        //     layouts
        //         .iter()
        //         .flat_map(|(nodes, _, _)| nodes
        //             .iter()
        //             .map(|(id, (x, y))| format!("{}({}, {})", id, x, y))
        //             .collect::<Vec<String>>())
        //         .collect::<Vec<String>>()
        //         .join(", ")
        // );
        // console::log!(
        //     "{}",
        //     layouts
        //         .iter()
        //         .map(|(_, width, height)| format!("({}, {})", width, height))
        //         .collect::<Vec<String>>()
        //         .join(", ")
        // );

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for (nodes, _, _) in &layouts {
            for (_, (x, y)) in nodes {
                let x = *x as f32;
                let y = *y as f32;
                if x < min_x {
                    min_x = x
                }
                if y < min_y {
                    min_y = y
                }
                if x > max_x {
                    max_x = x
                }
                if y > max_y {
                    max_y = y
                }
            }
        }
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;

        layouts
            .iter()
            .flat_map(|(nodes, _, _)| {
                nodes
                    .iter()
                    .map(|(id, (x, y))| {
                        (
                            *id,
                            Point {
                                x: *x as f32 - center_x,
                                y: *y as f32 - center_y,
                            },
                        )
                    })
                    .collect::<Vec<(NodeGroupID, Point)>>()
            })
            .collect()
    }
}
