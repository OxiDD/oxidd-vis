use std::collections::HashMap;

use oxidd_core::Tag;

use crate::types::util::{
    drawing::{diagram_layout::DiagramLayout, layout_rules::LayoutRules},
    grouped_graph_structure::GroupedGraphStructure,
};

///
/// A higher level layout that toggles between a set of other layout types, every time that the layout function is called. Intended for testing/demoing purposes
///
pub struct ToggleLayout<T: Tag> {
    layouts: Vec<Box<dyn LayoutRules<T>>>,
    current: usize,
}

impl<T: Tag> ToggleLayout<T> {
    pub fn new(layouts: Vec<Box<dyn LayoutRules<T>>>) -> ToggleLayout<T> {
        ToggleLayout {
            layouts,
            current: 0,
        }
    }
}

impl<T: Tag> LayoutRules<T> for ToggleLayout<T> {
    fn layout(
        &mut self,
        graph: &GroupedGraphStructure<T>,
        old: &DiagramLayout<T>,
        time: u32,
    ) -> DiagramLayout<T> {
        if self.layouts.len() == 0 {
            return DiagramLayout {
                groups: HashMap::new(),
                layers: HashMap::new(),
            };
        }
        let layout = self
            .layouts
            .get_mut(self.current)
            .unwrap()
            .layout(graph, old, time);

        self.current += 1;
        if self.current >= self.layouts.len() {
            self.current = 0;
        }

        layout
    }
}
