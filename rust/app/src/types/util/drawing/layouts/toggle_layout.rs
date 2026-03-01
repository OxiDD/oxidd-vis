use std::{collections::HashMap, marker::PhantomData};

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
pub struct ToggleLayout<L1: LayoutRules, L2: LayoutRules<G = L1::G>> {
    layout1: L1,
    layout2: L2,
    selected_one: bool,
}

pub struct ToggleLayoutUnit<L: LayoutRules> {
    layout: L,
}

impl<L1: LayoutRules, L2: LayoutRules<G = L1::G>> ToggleLayout<L1, L2> {
    pub fn new(layout1: L1, layout2: L2) -> Self {
        ToggleLayout {
            layout1,
            layout2,
            selected_one: true,
        }
    }

    pub fn select_layout_one(&mut self, selected_one: bool) -> () {
        self.selected_one = selected_one;
    }
    pub fn is_layout_one_selected(&self) -> bool {
        self.selected_one
    }
    pub fn get_layout_rules1(&mut self) -> &mut L1 {
        &mut self.layout1
    }
    pub fn get_layout_rules2(&mut self) -> &mut L2 {
        &mut self.layout2
    }
}

impl<L: LayoutRules> ToggleLayoutUnit<L> {
    pub fn new(layout: L) -> Self {
        ToggleLayoutUnit { layout: layout }
    }
    pub fn get_layout_rules(&mut self) -> &mut L {
        &mut self.layout
    }
}

pub trait IndexedSelect {
    fn select_layout(&mut self, index: usize) -> ();
    fn get_selected_layout(&self) -> usize;
}

impl<L1: LayoutRules, L2: LayoutRules<G = L1::G> + IndexedSelect> IndexedSelect
    for ToggleLayout<L1, L2>
{
    fn select_layout(&mut self, index: usize) -> () {
        if index == 0 {
            self.select_layout_one(true);
        } else {
            self.select_layout_one(false);
            self.layout2.select_layout(index - 1);
        }
    }
    fn get_selected_layout(&self) -> usize {
        if self.is_layout_one_selected() {
            0
        } else {
            self.layout2.get_selected_layout() + 1
        }
    }
}

impl<L: LayoutRules> IndexedSelect for ToggleLayoutUnit<L> {
    fn select_layout(&mut self, index: usize) -> () {}
    fn get_selected_layout(&self) -> usize {
        0
    }
}

impl<L: LayoutRules> LayoutRules for ToggleLayoutUnit<L> {
    type T = L::T;
    type NS = L::NS;
    type LS = L::LS;
    type Tracker = L::Tracker;
    type G = L::G;
    fn layout(
        &mut self,
        graph: &Self::G,
        old: &DiagramLayout<Self::T, Self::NS, Self::LS>,
        /* Sources for new nodes that did not yet exist in the previous layout iteration */
        new_sources: &Self::Tracker,
        time: u32,
    ) -> DiagramLayout<Self::T, Self::NS, Self::LS> {
        self.layout.layout(graph, old, new_sources, time)
    }
}

impl<
        L1: LayoutRules,
        L2: LayoutRules<G = L1::G, LS = L1::LS, NS = L1::NS, T = L1::T, Tracker = L1::Tracker>,
    > LayoutRules for ToggleLayout<L1, L2>
{
    type T = L1::T;
    type NS = L1::NS;
    type LS = L1::LS;
    type Tracker = L1::Tracker;
    type G = L1::G;
    fn layout(
        &mut self,
        graph: &Self::G,
        old: &DiagramLayout<Self::T, Self::NS, Self::LS>,
        new_sources: &Self::Tracker,
        time: u32,
    ) -> DiagramLayout<Self::T, Self::NS, Self::LS> {
        if self.is_layout_one_selected() {
            self.layout1.layout(graph, old, new_sources, time)
        } else {
            self.layout2.layout(graph, old, new_sources, time)
        }
    }
}
