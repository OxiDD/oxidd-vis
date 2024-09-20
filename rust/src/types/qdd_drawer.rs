use itertools::Itertools;
use oxidd_core::DiagramRules;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::hash::Hash;
use std::io::Cursor;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use web_sys::console::log;

use crate::configuration::configuration_object::AbstractConfigurationObject;
use crate::configuration::configuration_object::Abstractable;
use crate::configuration::types::choice_config::Choice;
use crate::configuration::types::choice_config::ChoiceConfig;
use crate::configuration::types::composite_config::CompositeConfig;
use crate::configuration::types::int_config::IntConfig;
use crate::configuration::types::label_config::LabelConfig;
use crate::traits::Diagram;
use crate::traits::DiagramSection;
use crate::traits::DiagramSectionDrawer;
use crate::types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceGroups;
use crate::types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceRemainder;
use crate::util::dummy_bdd::DummyEdge;
use crate::util::dummy_bdd::DummyFunction;
use crate::util::dummy_bdd::DummyManager;
use crate::util::dummy_bdd::DummyManagerRef;
use crate::util::free_id_manager::FreeIdManager;
use crate::util::logging::console;
use crate::util::rc_refcell::MutRcRefCell;
use crate::util::rectangle::Rectangle;
use crate::wasm_interface::NodeGroupID;
use crate::wasm_interface::NodeID;
use crate::wasm_interface::StepData;
use crate::wasm_interface::TargetID;
use crate::wasm_interface::TargetIDType;
use oxidd::bdd::BDDFunction;
use oxidd::util::Borrowed;
use oxidd::BooleanFunction;
use oxidd::Edge;
use oxidd::Function;
use oxidd::InnerNode;
use oxidd::{Manager, ManagerRef};
use oxidd_core::HasApplyCache;
use oxidd_core::HasLevel;
use oxidd_core::Node;
use oxidd_core::{util::DropWith, Tag};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use super::util::drawing::drawer::Drawer;
use super::util::drawing::layout_rules::LayoutRules;
use super::util::drawing::layouts::layer_group_sorting::average_group_alignment::AverageGroupAlignment;
use super::util::drawing::layouts::layer_group_sorting::ordering_group_alignment::OrderingGroupAlignment;
use super::util::drawing::layouts::layer_orderings::sugiyama_ordering::SugiyamaOrdering;
use super::util::drawing::layouts::layer_positionings::brandes_kopf_positioning::BrandesKopfPositioning;
use super::util::drawing::layouts::layer_positionings::dummy_layer_positioning::DummyLayerPositioning;
use super::util::drawing::layouts::layered_layout::LayeredLayout;
use super::util::drawing::layouts::random_test_layout::RandomTestLayout;
use super::util::drawing::layouts::sugiyama_lib_layout::SugiyamaLibLayout;
use super::util::drawing::layouts::toggle_layout::ToggleLayout;
use super::util::drawing::layouts::transition::transition_layout::TransitionLayout;
use super::util::drawing::layouts::util::color_label::Color;
use super::util::drawing::renderer::Renderer;
use super::util::drawing::renderers::webgl::edge_renderer::EdgeRenderingType;
use super::util::drawing::renderers::webgl::util::mix_color::mix_color;
use super::util::drawing::renderers::webgl_renderer::WebglRenderer;
use super::util::graph_structure::graph_manipulators::label_adjusters::group_label_adjuster::GroupLabelAdjuster;
use super::util::graph_structure::graph_manipulators::node_presence_adjuster::NodePresenceAdjuster;
use super::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceLabel;
use super::util::graph_structure::graph_manipulators::rc_graph::RCGraph;
use super::util::graph_structure::graph_manipulators::terminal_level_adjuster::TerminalLevelAdjuster;
use super::util::graph_structure::graph_structure::DrawTag;
use super::util::graph_structure::graph_structure::EdgeType;
use super::util::graph_structure::graph_structure::GraphStructure;
use super::util::graph_structure::grouped_graph_structure::GroupedGraphStructure;
use super::util::graph_structure::oxidd_graph_structure::NodeLabel;
use super::util::graph_structure::oxidd_graph_structure::OxiddGraphStructure;
use super::util::group_manager::GroupManager;
use super::util::storage::state_storage::Serializable;
use super::util::storage::state_storage::StateStorage;

pub struct QDDDiagram<MR: ManagerRef>
where
    for<'id> <<MR as oxidd::ManagerRef>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    manager_ref: MR,
}
impl QDDDiagram<DummyManagerRef> {
    pub fn new() -> QDDDiagram<DummyManagerRef> {
        let manager_ref = DummyManagerRef::from(&DummyManager::new());
        QDDDiagram { manager_ref }
    }
}

impl Diagram for QDDDiagram<DummyManagerRef> {
    fn create_section_from_dddmp(&mut self, dddmp: String) -> Option<Box<dyn DiagramSection>> {
        let root = DummyFunction::from_dddmp(&mut self.manager_ref, &dddmp[..]);
        Some(Box::new(QDDDiagramSection {
            roots: Vec::from([root]),
        }))
    }

    fn create_section_from_ids(&self, ids: &[oxidd::NodeID]) -> Option<Box<dyn DiagramSection>> {
        let roots = ids
            .iter()
            .map(|&id| {
                console::log!("for: {}", id);
                let root_edge = DummyEdge::new(Arc::new(id), self.manager_ref.clone());
                DummyFunction(root_edge)
            })
            .collect_vec();
        Some(Box::new(QDDDiagramSection { roots }))
    }
}

pub struct QDDDiagramSection<F: Function>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    roots: Vec<F>,
}

impl<
        T: ToString + Clone + 'static,
        E: Edge<Tag = ()> + 'static,
        N: InnerNode<E> + HasLevel + 'static,
        R: DiagramRules<E, N, T> + 'static,
        F: Function + 'static,
    > DiagramSection for QDDDiagramSection<F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = (), Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn create_drawer(&self, canvas: HtmlCanvasElement) -> Box<dyn DiagramSectionDrawer> {
        let c0 = (0., 0., 0.);
        let c1 = (0.4, 0.4, 0.4);
        let c2 = (0.6, 0.6, 0.6);

        let select_color = ((0.3, 0.3, 1.0), 0.8);
        let partial_select_color = ((0.6, 0.0, 1.0), 0.7);
        let hover_color = ((0.0, 0.0, 1.0), 0.3);
        let partial_hover_color = ((1.0, 0.0, 0.8), 0.2);

        let renderer = WebglRenderer::from_canvas(
            canvas,
            HashMap::from([
                (
                    EdgeType::new((), 0),
                    EdgeRenderingType {
                        color: c0,
                        hover_color: mix_color(c0, hover_color.0, hover_color.1),
                        select_color: mix_color(c0, select_color.0, select_color.1),
                        partial_hover_color: mix_color(
                            c0,
                            partial_hover_color.0,
                            partial_hover_color.1,
                        ),
                        partial_select_color: mix_color(
                            c0,
                            partial_select_color.0,
                            partial_select_color.1,
                        ),
                        width: 0.15,
                        dash_solid: 1.0,
                        dash_transparent: 0.0, // No dashing, just solid
                    },
                ),
                (
                    EdgeType::new((), 1),
                    EdgeRenderingType {
                        color: c1,
                        hover_color: mix_color(c1, hover_color.0, hover_color.1),
                        select_color: mix_color(c1, select_color.0, select_color.1),
                        partial_hover_color: mix_color(
                            c1,
                            partial_hover_color.0,
                            partial_hover_color.1,
                        ),
                        partial_select_color: mix_color(
                            c1,
                            partial_select_color.0,
                            partial_select_color.1,
                        ),
                        width: 0.15,
                        dash_solid: 0.2,
                        dash_transparent: 0.1,
                    },
                ),
                (
                    EdgeType::new((), 2),
                    EdgeRenderingType {
                        color: c2,
                        hover_color: mix_color(c2, hover_color.0, hover_color.1),
                        select_color: mix_color(c2, select_color.0, select_color.1),
                        partial_hover_color: mix_color(
                            c2,
                            partial_hover_color.0,
                            partial_hover_color.1,
                        ),
                        partial_select_color: mix_color(
                            c2,
                            partial_select_color.0,
                            partial_select_color.1,
                        ),
                        width: 0.1,
                        dash_solid: 1.0,
                        dash_transparent: 0.0,
                    },
                ),
            ]),
            select_color,
            partial_select_color,
            hover_color,
            partial_hover_color,
        )
        .unwrap();
        let layout = LayeredLayout::new(
            SugiyamaOrdering::new(1, 1),
            // AverageGroupAlignment,
            OrderingGroupAlignment,
            BrandesKopfPositioning,
            // DummyLayerPositioning,
            0.3,
        );
        let layout = TransitionLayout::new(layout);
        let graph =
            OxiddGraphStructure::new(self.roots.iter().cloned().collect(), |t| t.to_string());
        let diagram = QDDDiagramDrawer::new(graph, renderer, layout);
        Box::new(diagram)
    }
}

pub struct QDDDiagramDrawer<
    T: DrawTag + Serializable<T> + 'static,
    G: GraphStructure<T, NodeLabel<String>, String> + StateStorage + 'static,
    R: Renderer<T>,
    L: LayoutRules<T, Color, String, GMGraph<T, G>>,
> {
    graph: MGraph<T, G>,
    group_manager: MutRcRefCell<GM<T, G>>,
    presence_adjuster: MPresenceAdjuster<T, G>,
    drawer: Drawer<T, Color, R, L, GMGraph<T, G>>,
    config: CompositeConfig<(
        LabelConfig<IntConfig>,
        LabelConfig<ChoiceConfig<PresenceRemainder>>,
    )>,
}
type GraphLabel = PresenceLabel<NodeLabel<String>>;
type GM<T, G> = GroupManager<T, GraphLabel, String, MGraph<T, G>>;
type MGraph<T, G> = RCGraph<
    T,
    GraphLabel,
    String,
    TerminalLevelAdjuster<T, GraphLabel, String, MPresenceAdjuster<T, G>>,
>;
type MPresenceAdjuster<T, G> =
    RCGraph<T, GraphLabel, String, NodePresenceAdjuster<T, NodeLabel<String>, String, G>>;
type GMGraph<T, G> = GroupLabelAdjuster<T, Vec<GraphLabel>, String, GM<T, G>, (f32, f32, f32)>;

impl<
        T: DrawTag + Serializable<T> + 'static,
        G: GraphStructure<T, NodeLabel<String>, String> + StateStorage + 'static,
        R: Renderer<T>,
        L: LayoutRules<T, Color, String, GMGraph<T, G>>,
    > QDDDiagramDrawer<T, G, R, L>
{
    pub fn new(graph: G, renderer: R, layout: L) -> QDDDiagramDrawer<T, G, R, L> {
        let presence_adjuster = RCGraph::new(NodePresenceAdjuster::new(graph));
        let modified_graph = RCGraph::new(TerminalLevelAdjuster::new(presence_adjuster.clone()));
        let roots = modified_graph.get_roots();
        let group_manager = MutRcRefCell::new(GroupManager::new(modified_graph.clone()));
        let grouped_graph = GMGraph::new_shared(group_manager.clone(), |nodes| {
            match (nodes.get(0), nodes.get(1)) {
                (
                    Some(&PresenceLabel {
                        original_label: NodeLabel::Terminal(ref terminal),
                        original_id: _,
                    }),
                    None,
                ) => {
                    if terminal == "T" {
                        (0.2, 1., 0.2)
                    } else {
                        (1., 0.2, 0.2)
                    }
                }
                (Some(_), None) => (0., 0., 0.),
                _ => (0.7, 0.7, 0.7),
            }
        });

        let f1 = LabelConfig::new("Test", IntConfig::new(3));
        let mut f1clone = f1.clone();
        let f2 = LabelConfig::new(
            "False terminal",
            ChoiceConfig::new([
                Choice::new(PresenceRemainder::Show, "show"),
                Choice::new(PresenceRemainder::Duplicate, "duplicate"),
                Choice::new(PresenceRemainder::Hide, "hide"),
            ]),
        );
        let config = CompositeConfig::new((f1, f2), |(f1, f2)| {
            vec![Box::new(f1.clone()), Box::new(f2.clone())]
        });

        let mut out = QDDDiagramDrawer {
            group_manager,
            presence_adjuster,
            graph: modified_graph,
            drawer: Drawer::new(renderer, layout, MutRcRefCell::new(grouped_graph)),
            config,
        };
        let from = out.create_group(vec![TargetID(TargetIDType::NodeGroupID, 0)]);
        for root in roots {
            out.create_group(vec![TargetID(TargetIDType::NodeID, root)]);
        }

        // Config testing
        {
            let cfg2 = (*f1clone).clone();
            let cfg3 = (*f1clone).clone();
            let mut k = f1clone.clone();
            f1clone.add_value_dirty_listener(move || {
                console::log!("Dirty {}", cfg2.get());
                k.set_label(if cfg2.get() % 2 == 0 { &"Even" } else { &"Odd" })
                    .commit();
            });
            f1clone.add_value_change_listener(move || console::log!("Change {}", cfg3.get()));
        }

        // out.reveal_all(from, 30000);
        // out.reveal_all(from, 10);
        out.set_terminal_mode("F".to_string(), PresenceRemainder::Hide);
        out
    }

    pub fn reveal_all(&mut self, from_id: NodeGroupID, limit: u32) {
        let nodes = {
            let explored_group =
                self.create_group(vec![TargetID(TargetIDType::NodeGroupID, from_id)]);
            self.group_manager.read().get_nodes_of_group(explored_group)
        };
        let mut count = 0;
        for node_id in nodes {
            // console::log!("{node_id}");
            self.create_group(vec![TargetID(TargetIDType::NodeID, node_id)]);

            count = count + 1;
            if limit > 0 && count >= limit {
                break;
            }
        }
    }
}

impl<
        T: DrawTag + Serializable<T> + 'static,
        G: GraphStructure<T, NodeLabel<String>, String> + StateStorage + 'static,
        R: Renderer<T>,
        L: LayoutRules<T, Color, String, GMGraph<T, G>>,
    > DiagramSectionDrawer for QDDDiagramDrawer<T, G, R, L>
{
    fn render(&mut self, time: u32) -> () {
        self.drawer.render(time);
    }

    fn layout(&mut self, time: u32) -> () {
        self.drawer.layout(time);
    }

    fn set_transform(&mut self, width: u32, height: u32, x: f32, y: f32, scale: f32) -> () {
        self.drawer.set_transform(width, height, x, y, scale);
    }

    fn set_step(&mut self, step: i32) -> Option<StepData> {
        todo!()
    }

    fn set_group(&mut self, from: Vec<TargetID>, to: NodeGroupID) -> bool {
        self.group_manager.get().set_group(from, to)
    }

    fn create_group(&mut self, from: Vec<TargetID>) -> NodeGroupID {
        self.group_manager.get().create_group(from)
    }

    fn split_edges(&mut self, nodes: &[NodeID], fully: bool) {
        self.group_manager.get().split_edges(nodes, fully);
    }
    fn set_terminal_mode(&mut self, terminal: String, mode: PresenceRemainder) -> () {
        let mut adjuster = self.presence_adjuster.get();
        let terminals = adjuster.get_terminals();
        let terminals = terminals.iter().filter_map(|&node| {
            match adjuster.get_node_label(node).original_label {
                NodeLabel::Terminal(t) if t == terminal => Some(node),
                _ => None,
            }
        });
        let Some(target_terminal) = terminals.last() else {
            return;
        };

        adjuster.set_node_presence(target_terminal, PresenceGroups::remainder(mode));
    }

    fn get_nodes(&self, area: Rectangle, max_group_expansion: usize) -> Vec<NodeID> {
        self.drawer.get_nodes(area, max_group_expansion)
    }

    fn set_selected_nodes(&mut self, selected_ids: &[NodeID], hovered_ids: &[NodeID]) {
        self.drawer.select_nodes(selected_ids, hovered_ids);
    }

    fn local_nodes_to_sources(&self, nodes: &[NodeID]) -> Vec<NodeID> {
        self.graph
            .local_nodes_to_sources(nodes.iter().cloned().collect())
    }

    fn source_nodes_to_local(&self, nodes: &[NodeID]) -> Vec<NodeID> {
        self.graph
            .source_nodes_to_local(nodes.iter().cloned().collect())
    }
    fn get_terminals(&self) -> Vec<(String, String)> {
        vec![
            ("T".to_string(), "True".to_string()),
            ("F".to_string(), "False".to_string()),
        ]
    }

    fn serialize_state(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.group_manager.read().write(&mut Cursor::new(&mut out));
        out
    }

    fn deserialize_state(&mut self, state: Vec<u8>) -> () {
        self.group_manager.get().read(&mut Cursor::new(&state));
    }

    fn get_configuration(&self) -> AbstractConfigurationObject {
        self.config.get_abstract()
    }
}
