use oxidd::{Function, Manager};
use oxidd_core::Tag;

use crate::{
    types::util::{
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
        group_manager::GroupManager,
    },
    wasm_interface::NodeGroupID,
};

use super::diagram_layout::DiagramLayout;

pub trait LayoutRules<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>> {
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T>,
        /* Sources for new nodes that did not yet exist in the previous layout iteration */
        new_sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T>;
}
