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
    types::util::{
        group_manager::GroupManager,
        grouped_graph_structure::{GroupedGraphStructure, SourceReader, SourceTracker},
    },
    util::{
        logging::console,
        rc_refcell::{MutRcRefCell, RcRefCell},
        rectangle::Rectangle,
        transformation::Transformation,
    },
};

use super::{
    diagram_layout::{DiagramLayout, Point},
    layout_rules::LayoutRules,
    renderer::Renderer,
};

pub struct Drawer<T: Tag, R: Renderer<T>, L: LayoutRules<T, G>, G: GroupedGraphStructure<T>> {
    renderer: R,
    layout_rules: L,
    layout: DiagramLayout<T>,
    graph: RcRefCell<G>,
    sources: G::Tracker,
    transform: Transformation,
}

impl<T: Tag, R: Renderer<T>, L: LayoutRules<T, G>, G: GroupedGraphStructure<T>> Drawer<T, R, L, G> {
    pub fn new(renderer: R, layout_rules: L, graph: MutRcRefCell<G>) -> Drawer<T, R, L, G> {
        Drawer {
            sources: graph.get().get_source_reader(),
            renderer,
            layout_rules,
            graph: graph.clone_readonly(),
            layout: DiagramLayout {
                groups: HashMap::new(),
                layers: Vec::new(),
            },
            transform: Transformation::default(),
        }
    }

    pub fn layout(&mut self, time: u32) {
        self.layout =
            self.layout_rules
                .layout(&*self.graph.read(), &self.layout, &self.sources, time);
        for group_id in self.sources.get_sourced_nodes() {
            self.sources.delete_source(group_id);
        }
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
