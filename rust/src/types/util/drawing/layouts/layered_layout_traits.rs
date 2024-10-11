use std::collections::HashMap;

use oxidd::LevelNo;

use crate::{
    types::util::{
        drawing::diagram_layout::Point,
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
    },
    wasm_interface::NodeGroupID,
};

use super::util::layered::layer_orderer::{EdgeMap, Order};

/// The trait used to decide what ordering of nodes to use in the layout, including dummy nodes
pub trait LayerOrdering<T: DrawTag, GL, LL> {
    fn order_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        // The ID such that any ID in the range [dummy_group_start_id, dummy_edge_start_id) represents a dummy node of a group
        dummy_group_start_id: NodeGroupID,
        // The ID such that any ID greater or equal represents a dummy node of an edge
        dummy_edge_start_id: NodeGroupID,
        // The owner of a given dummy node, such that multiple nodes derived from the same data can be considered as a group
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order>;
}

/// The trait used to decide what positioning of nodes to use in the layout for the given node orders, including dummy nodes
pub trait NodePositioning<T: DrawTag, GL, LL> {
    // TODO: change interface to provide node widths instead of having to get them from the graph
    fn position_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        node_widths: &HashMap<NodeGroupID, f32>,
        // The ID such that any ID in the range [dummy_group_start_id, dummy_edge_start_id) represents a dummy node of a group
        dummy_group_start_id: NodeGroupID,
        // The ID such that any ID greater or equal represents a dummy node of an edge
        dummy_edge_start_id: NodeGroupID,
        // The owner of a given dummy node, such that multiple nodes derived from the same data can be considered as a group
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> (HashMap<NodeGroupID, Point>, HashMap<LevelNo, f32>);
}

/// The trait used to decide what positioning of nodes to use in the layout for the given node orders, including dummy nodes
pub trait LayerGroupSorting<T: DrawTag, GL, LL> {
    fn align_cross_layer_nodes(
        &self,
        graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        edges: &EdgeMap,
        // The ID such that any ID in the range [dummy_group_start_id, dummy_edge_start_id) represents a dummy node of a group
        dummy_group_start_id: NodeGroupID,
        // The ID such that any ID greater or equal represents a dummy node of an edge
        dummy_edge_start_id: NodeGroupID,
        // The owner of a given dummy node, such that multiple nodes derived from the same data can be considered as a group
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order>;
}

/// A trait for node tags to specify the width of the node, where 1 is the unit size
pub trait WidthLabel {
    fn get_width(&self) -> f32;
}
