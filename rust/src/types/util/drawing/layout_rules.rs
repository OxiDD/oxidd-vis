use oxidd::{Function, Manager};
use oxidd_core::Tag;

use crate::{
    types::util::{
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
        group_manager::GroupManager,
    },
    util::transition::Interpolatable,
    wasm_interface::NodeGroupID,
};

use super::diagram_layout::{DiagramLayout, LayerStyle, NodeStyle};

pub trait LayoutRules<T: DrawTag, S: NodeStyle, LS: LayerStyle, G: GroupedGraphStructure<T, S, LS>>
{
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T, S, LS>,
        /* Sources for new nodes that did not yet exist in the previous layout iteration */
        new_sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T, S, LS>;
}
