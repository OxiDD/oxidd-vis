use std::collections::HashMap;

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
        grouped_graph_structure::GroupedGraphStructure,
    },
    wasm_interface::NodeGroupID,
};

pub struct DummyLayerPositioning;

impl<T: Tag> NodePositioning<T> for DummyLayerPositioning {
    fn position_nodes(
        &self,
        graph: &dyn GroupedGraphStructure<T>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        dummy_start_id: NodeGroupID,
    ) -> HashMap<NodeGroupID, Point> {
        layers
            .iter()
            .enumerate()
            .flat_map(|(layer_index, layer)| {
                layer.iter().map(move |(&node, &node_index)| {
                    (
                        node,
                        Point {
                            x: (node_index as f32) * 2.,
                            y: -(layer_index as f32) * 2.,
                        },
                    )
                })
            })
            .collect()
    }
}
