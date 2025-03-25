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
pub struct ToggleLayout<
    T: DrawTag,
    S: NodeStyle,
    LS: LayerStyle,
    G: GroupedGraphStructure<T, S, LS>,
    L1: LayoutRules<T, S, LS, G>,
    L2: LayoutRules<T, S, LS, G>,
> {
    layout1: L1,
    layout2: L2,
    selected_one: bool,
    tag: PhantomData<T>,
    node_style: PhantomData<S>,
    layer_style: PhantomData<LS>,
    graph: PhantomData<G>,
}

pub struct ToggleLayoutUnit<
    T: DrawTag,
    S: NodeStyle,
    LS: LayerStyle,
    G: GroupedGraphStructure<T, S, LS>,
    L: LayoutRules<T, S, LS, G>,
> {
    layout: L,
    tag: PhantomData<T>,
    node_style: PhantomData<S>,
    layer_style: PhantomData<LS>,
    graph: PhantomData<G>,
}

impl<
        T: DrawTag,
        S: NodeStyle,
        LS: LayerStyle,
        G: GroupedGraphStructure<T, S, LS>,
        L1: LayoutRules<T, S, LS, G>,
        L2: LayoutRules<T, S, LS, G>,
    > ToggleLayout<T, S, LS, G, L1, L2>
{
    pub fn new(layout1: L1, layout2: L2) -> ToggleLayout<T, S, LS, G, L1, L2> {
        ToggleLayout {
            layout1,
            layout2,
            selected_one: true,
            tag: PhantomData,
            node_style: PhantomData,
            layer_style: PhantomData,
            graph: PhantomData,
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

impl<
        T: DrawTag,
        S: NodeStyle,
        LS: LayerStyle,
        G: GroupedGraphStructure<T, S, LS>,
        L: LayoutRules<T, S, LS, G>,
    > ToggleLayoutUnit<T, S, LS, G, L>
{
    pub fn new(layout: L) -> ToggleLayoutUnit<T, S, LS, G, L> {
        ToggleLayoutUnit {
            layout: layout,
            tag: PhantomData,
            node_style: PhantomData,
            layer_style: PhantomData,
            graph: PhantomData,
        }
    }
    pub fn get_layout_rules(&mut self) -> &mut L {
        &mut self.layout
    }
}

pub trait IndexedSelect {
    fn select_layout(&mut self, index: usize) -> ();
    fn get_selected_layout(&self) -> usize;
}

impl<
        T: DrawTag,
        S: NodeStyle,
        LS: LayerStyle,
        G: GroupedGraphStructure<T, S, LS>,
        L1: LayoutRules<T, S, LS, G>,
        L2: LayoutRules<T, S, LS, G> + IndexedSelect,
    > IndexedSelect for ToggleLayout<T, S, LS, G, L1, L2>
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

impl<
        T: DrawTag,
        S: NodeStyle,
        LS: LayerStyle,
        G: GroupedGraphStructure<T, S, LS>,
        L: LayoutRules<T, S, LS, G>,
    > IndexedSelect for ToggleLayoutUnit<T, S, LS, G, L>
{
    fn select_layout(&mut self, index: usize) -> () {}
    fn get_selected_layout(&self) -> usize {
        0
    }
}

impl<
        T: DrawTag,
        S: NodeStyle,
        LS: LayerStyle,
        G: GroupedGraphStructure<T, S, LS>,
        L: LayoutRules<T, S, LS, G>,
    > LayoutRules<T, S, LS, G> for ToggleLayoutUnit<T, S, LS, G, L>
{
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T, S, LS>,
        /* Sources for new nodes that did not yet exist in the previous layout iteration */
        new_sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T, S, LS> {
        self.layout.layout(graph, old, new_sources, time)
    }
}

impl<
        T: DrawTag,
        S: NodeStyle,
        LS: LayerStyle,
        G: GroupedGraphStructure<T, S, LS>,
        L1: LayoutRules<T, S, LS, G>,
        L2: LayoutRules<T, S, LS, G>,
    > LayoutRules<T, S, LS, G> for ToggleLayout<T, S, LS, G, L1, L2>
{
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T, S, LS>,
        new_sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T, S, LS> {
        if self.is_layout_one_selected() {
            self.layout1.layout(graph, old, new_sources, time)
        } else {
            self.layout2.layout(graph, old, new_sources, time)
        }
    }
}
