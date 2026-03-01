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

pub trait LayoutRules {
    type T: DrawTag;
    type NS: NodeStyle;
    type LS: LayerStyle;
    type Tracker;
    type G: GroupedGraphStructure<
        GL = Self::NS,
        LL = Self::LS,
        T = Self::T,
        Tracker = Self::Tracker,
    >;
    fn layout(
        &mut self,
        graph: &Self::G,
        old: &DiagramLayout<Self::T, Self::NS, Self::LS>,
        /* Sources for new nodes that did not yet exist in the previous layout iteration */
        new_sources: &Self::Tracker,
        time: u32,
    ) -> DiagramLayout<Self::T, Self::NS, Self::LS>;
}
