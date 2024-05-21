use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Deref,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use js_sys::Date;
use oxidd::{Function, Manager};
use oxidd_core::Tag;
use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::{group_manager::GroupManager, grouped_graph_structure::GroupedGraphStructure},
    util::logging::console,
};

use super::{diagram_layout::DiagramLayout, layout_rules::LayoutRules, renderer::Renderer};

pub struct Drawer<T: Tag> {
    renderer: Box<dyn Renderer<T>>,
    layout_rules: Box<dyn LayoutRules<T>>,
    layout: DiagramLayout<T>,
    graph: Rc<RefCell<dyn GroupedGraphStructure<T>>>,
}

impl<T: Tag> Drawer<T> {
    pub fn new(
        renderer: Box<dyn Renderer<T>>,
        layout_rules: Box<dyn LayoutRules<T>>,
        graph: Rc<RefCell<dyn GroupedGraphStructure<T>>>,
    ) -> Drawer<T> {
        Drawer {
            renderer,
            layout_rules,
            graph,
            layout: DiagramLayout {
                groups: HashMap::new(),
                layers: HashMap::new(),
            },
        }
    }

    pub fn layout(&mut self, time: u32) {
        self.layout = self
            .layout_rules
            .layout(&(*self.graph.borrow()), &self.layout, time);
        self.renderer.update_layout(&self.layout);
    }
    pub fn set_transform(&mut self, x: f32, y: f32, scale: f32) {
        self.renderer.set_transform(x, y, scale);
    }
    pub fn render(&mut self, time: u32, selected_ids: &[u32], hovered_ids: &[u32]) {
        self.renderer.render(time, selected_ids, hovered_ids);
    }
}
