use oxidd::{Function, Manager};
use oxidd_core::Tag;

use crate::types::util::drawing::layout_rules::LayoutRules;

///
/// A layout builder that takes another layout approach, and applies transitioning to it.
/// This will make layout changes smoothly transition from the previous state to the new state.
///
pub struct TransitionLayout<T: Tag, F: Function>(Box<dyn LayoutRules<T, F>>)
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = T>;

impl<T: Tag, F: Function> TransitionLayout<T, F>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = T>,
{
    pub fn new(layout: Box<dyn LayoutRules<T, F>>) -> TransitionLayout<T, F> {
        TransitionLayout(layout)
    }
}

impl<T: Tag, F: Function> LayoutRules<T, F> for TransitionLayout<T, F>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = T>,
{
    fn layout(
        &mut self,
        groups: &crate::types::util::group_manager::GroupManager<T, F>,
        old: &crate::types::util::drawing::diagram_layout::DiagramLayout<T>,
    ) -> crate::types::util::drawing::diagram_layout::DiagramLayout<T> {
        // TODO:
        self.0.layout(groups, old)
    }
}
