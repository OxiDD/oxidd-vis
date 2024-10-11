use std::collections::HashMap;

use itertools::Itertools;
use oxidd::LevelNo;
use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::{
            diagram_layout::Point,
            layouts::{
                layered_layout_traits::{NodePositioning, WidthLabel},
                util::layered::layer_orderer::{EdgeMap, Order},
            },
        },
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
    },
    wasm_interface::NodeGroupID,
};

pub struct DummyLayerPositioning;

impl<T: DrawTag, GL: WidthLabel, LL> NodePositioning<T, GL, LL> for DummyLayerPositioning {
    fn position_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        node_widths: &HashMap<NodeGroupID, f32>,
        dummy_group_start_id: NodeGroupID,
        dummy_edge_start_id: NodeGroupID,
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> (HashMap<NodeGroupID, Point>, HashMap<LevelNo, f32>) {
        let spacing = 2.;

        (
            layers
                .iter()
                .enumerate()
                .flat_map(|(layer_index, layer)| {
                    let mut points = Vec::<(NodeGroupID, Point)>::new();
                    let mut x = 0.0;
                    for (&node, _) in layer.iter().sorted_by_key(|&(_, i)| i) {
                        let width = node_widths[&node];
                        points.push((
                            node,
                            Point {
                                x: x,
                                y: -(layer_index as f32) * spacing,
                            },
                        ));
                        x += (spacing - 1.) + width;
                    }
                    points
                })
                .collect(),
            layers
                .iter()
                .enumerate()
                .map(|(level, _)| (level as u32, -(level as f32 * spacing)))
                .collect(),
        )
    }
}
