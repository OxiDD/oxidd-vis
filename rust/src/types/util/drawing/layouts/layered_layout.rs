use oxidd_core::Tag;

use crate::{
    types::util::grouped_graph_structure::GroupedGraphStructure, wasm_interface::NodeGroupID,
};

/// The trait used to decide what ordering of nodes to use in the layout
pub trait LayerOrdering<T: Tag> {
    fn order_nodes(
        &self,
        graph: &dyn GroupedGraphStructure<T>,
        layers: &Vec<Vec<NodeGroupID>>,
    ) -> Vec<Vec<NodeGroupID>>;
}

pub struct LayeredLayout<T: Tag> {
    ordering: Box<dyn LayerOrdering<T>>,
}

impl<T: Tag> LayeredLayout<T> {
    pub fn new(ordering: Box<dyn LayerOrdering<T>>) -> LayeredLayout<T> {
        LayeredLayout { ordering }
    }
}
