use oxidd::{Function, Manager};
use oxidd_core::Tag;

use crate::types::util::group_manager::GroupManager;

use super::diagram_layout::DiagramLayout;

pub trait LayoutRules<T: Tag> {
    fn layout(
        &mut self,
        groups: &GroupManager<T>,
        old: &DiagramLayout<T>,
        time: u32,
    ) -> DiagramLayout<T>;
}
