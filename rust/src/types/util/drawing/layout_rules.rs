use oxidd::{Function, Manager};
use oxidd_core::Tag;

use crate::types::util::group_manager::GroupManager;

use super::diagram_layout::DiagramLayout;

pub trait LayoutRules<T: Tag, F: Function>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = T>,
{
    fn layout(&mut self, groups: &GroupManager<T, F>, old: &DiagramLayout<T>) -> DiagramLayout<T>;
}
