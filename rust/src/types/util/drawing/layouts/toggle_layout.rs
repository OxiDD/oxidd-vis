use std::collections::HashMap;

use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::{
            diagram_layout::{DiagramLayout, LayerStyle, NodeStyle},
            layout_rules::LayoutRules,
        },
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
    },
    util::transition::Interpolatable,
};

///
/// A higher level layout that toggles between a set of other layout types, every time that the layout function is called. Intended for testing/demoing purposes
///
pub struct ToggleLayout<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>> {
    layouts: Vec<Box<dyn LayoutRules<T, GL, LL, G>>>,
    current: usize,
}

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>> ToggleLayout<T, GL, LL, G> {
    pub fn new(layouts: Vec<Box<dyn LayoutRules<T, GL, LL, G>>>) -> ToggleLayout<T, GL, LL, G> {
        ToggleLayout {
            layouts,
            current: 0,
        }
    }
}

impl<T: DrawTag, S: NodeStyle, LS: LayerStyle, G: GroupedGraphStructure<T, S, LS>>
    LayoutRules<T, S, LS, G> for ToggleLayout<T, S, LS, G>
{
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T, S, LS>,
        new_sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T, S, LS> {
        if self.layouts.len() == 0 {
            return DiagramLayout {
                groups: HashMap::new(),
                layers: Vec::new(),
            };
        }
        let layout =
            self.layouts
                .get_mut(self.current)
                .unwrap()
                .layout(graph, old, new_sources, time);

        self.current += 1;
        if self.current >= self.layouts.len() {
            self.current = 0;
        }

        layout
    }
}
