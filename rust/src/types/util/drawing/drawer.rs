use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use oxidd::{Function, Manager};
use oxidd_core::Tag;
use web_sys::WebGl2RenderingContext;

use crate::types::util::group_manager::GroupManager;

use super::{diagram_layout::DiagramLayout, layout_rules::LayoutRules, renderer::Renderer};

pub struct Drawer<T: Tag, F: Function>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = T>,
{
    renderer: Box<dyn Renderer<T>>,
    layout_rules: Box<dyn LayoutRules<T, F>>,
    layout: DiagramLayout<T>,
    groups: Rc<RefCell<GroupManager<T, F>>>,
}

impl<T: Tag, F: Function> Drawer<T, F>
where
    for<'id> F::Manager<'id>: Manager<EdgeTag = T>,
{
    pub fn new(
        renderer: Box<dyn Renderer<T>>,
        layout_rules: Box<dyn LayoutRules<T, F>>,
        groups: Rc<RefCell<GroupManager<T, F>>>,
    ) -> Drawer<T, F> {
        Drawer {
            renderer,
            layout_rules,
            groups,
            layout: DiagramLayout {
                groups: HashMap::new(),
                layers: HashMap::new(),
            },
        }
    }

    pub fn layout(&mut self) {
        self.layout_rules
            .layout(&(*self.groups.borrow()), &self.layout);
    }

    pub fn render(&self, time: i32, selected_ids: &[u32], hovered_ids: &[u32]) {
        self.renderer
            .render(&self.layout, time, selected_ids, hovered_ids);
    }
}
