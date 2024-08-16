use std::collections::HashMap;

use oxidd::LevelNo;
use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::{
            diagram_layout::Point,
            layouts::{
                layered_layout::NodePositioning,
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

impl<T: DrawTag, GL, LL> NodePositioning<T, GL, LL> for DummyLayerPositioning {
    fn position_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
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
                    layer.iter().map(move |(&node, &node_index)| {
                        (
                            node,
                            Point {
                                x: (node_index as f32) * spacing,
                                y: -(layer_index as f32) * spacing,
                            },
                        )
                    })
                })
                .collect(),
            layers
                .iter()
                .enumerate()
                .map(|(level, _)| (level as u32, level as f32 * spacing))
                .collect(),
        )
    }
}
