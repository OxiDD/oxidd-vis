use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    ops::Deref,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use itertools::{Either, Itertools};
use js_sys::Date;
use oxidd::{Function, Manager, NodeID};
use oxidd_core::Tag;
use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::{
        graph_structure::{
            graph_structure::DrawTag,
            grouped_graph_structure::{GroupedGraphStructure, NodeTracker, SourceReader},
        },
        group_manager::GroupManager,
    },
    util::{
        logging::console,
        rc_refcell::{MutRcRefCell, RcRefCell},
        rectangle::Rectangle,
        transformation::Transformation,
    },
    wasm_interface::NodeGroupID,
};

use super::{
    diagram_layout::{DiagramLayout, Point},
    layout_rules::LayoutRules,
    renderer::{GroupSelection, Renderer},
};

pub struct Drawer<
    T: DrawTag,
    GL,
    R: Renderer<T>,
    L: LayoutRules<T, GL, String, G>,
    G: GroupedGraphStructure<T, GL, String>,
> {
    renderer: R,
    layout_rules: L,
    layout: DiagramLayout<T>,
    graph: MutRcRefCell<G>,
    sources: G::Tracker,
    transform: Transformation,
    selection: SelectionData,
}

type SelectionData = (Vec<NodeGroupID>, Vec<NodeGroupID>);

impl<
        T: DrawTag,
        GL,
        R: Renderer<T>,
        L: LayoutRules<T, GL, String, G>,
        G: GroupedGraphStructure<T, GL, String>,
    > Drawer<T, GL, R, L, G>
{
    pub fn new(renderer: R, layout_rules: L, graph: MutRcRefCell<G>) -> Drawer<T, GL, R, L, G> {
        Drawer {
            sources: graph.get().create_node_tracker(),
            renderer,
            layout_rules,
            graph: graph.clone(),
            layout: DiagramLayout {
                groups: HashMap::new(),
                layers: Vec::new(),
            },
            transform: Transformation::default(),
            selection: (Vec::new(), Vec::new()),
        }
    }

    pub fn layout(&mut self, time: u32) {
        self.graph.get().refresh();
        self.layout =
            self.layout_rules
                .layout(&*self.graph.read(), &self.layout, &self.sources, time);
        let used_ids = self.layout.groups.keys().collect::<HashSet<_>>();

        self.sources.retain(|group_id| used_ids.contains(&group_id));
        self.sources.remove_sources();

        let old_selection = self.selection.clone();
        self.select_nodes(&[], &[]);
        self.renderer.update_layout(&self.layout);
        self.select_nodes(&old_selection.0[..], &old_selection.1[..]);
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

    pub fn render(&mut self, time: u32) {
        self.renderer.render(time);
    }

    pub fn get_nodes(&self, area: Rectangle, max_group_expansion: usize) -> Vec<NodeID> {
        let area = area.transform(self.transform.get_inverse_matrix());
        let groups = self
            .layout
            .groups
            .iter()
            .filter(|(_, node_layout)| node_layout.get_rect().overlaps(&area))
            .map(|(&group_id, _)| group_id);
        groups
            .flat_map(|group_id| {
                self.graph
                    .read()
                    .get_nodes_of_group(group_id)
                    .take(max_group_expansion)
            })
            .collect()
    }

    pub fn select_nodes(&mut self, selected_ids: &[NodeID], hovered_ids: &[NodeID]) {
        if selected_ids == &self.selection.0[..] && hovered_ids == &self.selection.1[..] {
            return;
        }

        let (old_selected_group_ids, old_partially_selected_group_ids) =
            self.get_selection_groups(&self.selection.0[..]);
        let (old_hovered_group_ids, old_partially_hovered_group_ids) =
            self.get_selection_groups(&self.selection.1[..]);

        let (selected_group_ids, partially_selected_group_ids) =
            self.get_selection_groups(selected_ids);
        let (hovered_group_ids, partially_hovered_group_ids) =
            self.get_selection_groups(hovered_ids);

        let selection = (
            &selected_group_ids[..],
            &partially_selected_group_ids[..],
            &hovered_group_ids[..],
            &partially_hovered_group_ids[..],
        );
        let old_selection = (
            &old_selected_group_ids[..],
            &old_partially_selected_group_ids[..],
            &old_hovered_group_ids[..],
            &old_partially_hovered_group_ids[..],
        );
        self.renderer.select_groups(selection, old_selection);

        self.selection = (Vec::from(selected_ids), Vec::from(hovered_ids));
    }
    fn get_selection_groups(&self, node_ids: &[NodeID]) -> (Vec<NodeGroupID>, Vec<NodeGroupID>) {
        // TODO: make the graph track sources, and use this info for selection (such that duplicate nodes select all duplications)

        let graph = self.graph.read();
        let mut group_counts = HashMap::<NodeGroupID, usize>::new();
        for &node_id in node_ids {
            let group_id = graph.get_group(node_id);
            *group_counts.entry(group_id).or_insert(0) += 1;
        }

        group_counts.iter().partition_map(|(&group_id, &count)| {
            if graph.get_nodes_of_group(group_id).len() == count {
                Either::Left(group_id)
            } else {
                Either::Right(group_id)
            }
        })
    }
}
