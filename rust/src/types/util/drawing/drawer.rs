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
    util::{logging::console, rectangle::Rectangle, transformation::Transformation},
};

use super::{
    diagram_layout::{DiagramLayout, Point},
    layout_rules::LayoutRules,
    renderer::Renderer,
};

pub struct Drawer<T: Tag> {
    renderer: Box<dyn Renderer<T>>,
    layout_rules: Box<dyn LayoutRules<T>>,
    layout: DiagramLayout<T>,
    graph: Rc<RefCell<dyn GroupedGraphStructure<T>>>,
    transform: Transformation,
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
            transform: Transformation::default(),
        }
    }

    pub fn layout(&mut self, time: u32) {
        self.layout = self
            .layout_rules
            .layout(&(*self.graph.borrow()), &self.layout, time);
        self.renderer.update_layout(&self.layout);
    }
    pub fn set_transform(&mut self, width: u32, height: u32, x: f32, y: f32, scale: f32) {
        let transform = Transformation {
            width: width as f32,
            height: height as f32,
            scale,
            position: Point { x, y },
            angle: 0.0,
        };
        self.transform = transform.clone();
        self.renderer.set_transform(transform);
    }
    pub fn render(&mut self, time: u32, selected_ids: &[u32], hovered_ids: &[u32]) {
        self.renderer.render(time, selected_ids, hovered_ids);
    }

    pub fn get_nodes(&self, area: Rectangle) -> Vec<crate::wasm_interface::NodeGroupID> {
        let area = area.transform(self.transform.get_inverse_matrix());
        self.layout
            .groups
            .iter()
            .filter(|(_, node_layout)| node_layout.get_rect().overlaps(&area))
            .map(|(&group_id, _)| group_id)
            .collect()
    }
}
