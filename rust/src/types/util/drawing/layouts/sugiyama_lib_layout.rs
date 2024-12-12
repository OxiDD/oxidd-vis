use oxidd::LevelNo;
use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::{
            diagram_layout::{DiagramLayout, LayerStyle, NodeStyle},
            layout_rules::LayoutRules,
        },
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
    },
    util::{logging::console, point::Point, transition::Interpolatable},
    wasm_interface::NodeGroupID,
};
use rust_sugiyama::from_edges;
use std::{collections::HashMap, convert::TryInto};

use super::{
    layer_group_sorting::average_group_alignment::AverageGroupAlignment,
    layer_orderings::dummy_layer_ordering::DummyLayerOrdering,
    layered_layout::LayeredLayout,
    layered_layout_traits::{NodePositioning, WidthLabel},
    util::layered::layer_orderer::{EdgeMap, Order},
};

pub struct SugiyamaLibLayout<T: DrawTag, S, LS> {
    layout:
        LayeredLayout<T, S, LS, DummyLayerOrdering, AverageGroupAlignment, SugiyamaLibPositioning>,
}
impl<T: DrawTag, S, LS> SugiyamaLibLayout<T, S, LS> {
    pub fn new(max_curve_offset: f32) -> SugiyamaLibLayout<T, S, LS> {
        SugiyamaLibLayout {
            layout: LayeredLayout::new(
                DummyLayerOrdering,
                AverageGroupAlignment,
                SugiyamaLibPositioning,
                max_curve_offset,
            ),
        }
    }
}

impl<T: DrawTag, S: NodeStyle + WidthLabel, LS: LayerStyle, G: GroupedGraphStructure<T, S, LS>>
    LayoutRules<T, S, LS, G> for SugiyamaLibLayout<T, S, LS>
{
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T, S, LS>,
        sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T, S, LS> {
        self.layout.layout(graph, old, sources, time)
    }
}
struct SugiyamaLibPositioning;
impl<T: DrawTag, S, LS> NodePositioning<T, S, LS> for SugiyamaLibPositioning {
    fn position_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T, S, LS>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        node_widths: &HashMap<NodeGroupID, f32>,
        dummy_group_start_id: NodeGroupID,
        dummy_edge_start_id: NodeGroupID,
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> (HashMap<NodeGroupID, Point>, HashMap<LevelNo, f32>) {
        let spacing = 2;
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
                .flat_map(|(from, to_set)| to_set.keys().map(move |to| (*from as u32, *to as u32)))
                .collect::<Vec<(u32, u32)>>()[..],
        )
        .vertex_spacing(spacing)
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

        (
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
                .collect(),
            layers
                .iter()
                .enumerate()
                .map(|(level, _)| (level as u32, (level * spacing) as f32))
                .collect(),
        )
    }
}
