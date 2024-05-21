use oxidd::{Function, Manager};
use oxidd_core::Tag;

use crate::types::util::{
    group_manager::GroupManager, grouped_graph_structure::GroupedGraphStructure,
};

use super::diagram_layout::DiagramLayout;

pub trait LayoutRules<T: Tag> {
    fn layout(
        &mut self,
        graph: &GroupedGraphStructure<T>,
        old: &DiagramLayout<T>,
        time: u32,
    ) -> DiagramLayout<T>;
}
