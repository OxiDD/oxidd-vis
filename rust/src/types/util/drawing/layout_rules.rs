use oxidd::{Function, Manager};
use oxidd_core::Tag;

use crate::{
    types::util::{
        group_manager::GroupManager,
        grouped_graph_structure::{GroupedGraphStructure, SourceReader},
    },
    wasm_interface::NodeGroupID,
};

use super::diagram_layout::DiagramLayout;

pub trait LayoutRules<T: Tag, G: GroupedGraphStructure<T>> {
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T>,
        /* Sources for new nodes that did not yet exist in the previous layout iteration */
        new_sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T>;
}
