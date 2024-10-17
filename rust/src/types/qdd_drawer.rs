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

use crate::configuration::configuration::Configuration;
use crate::configuration::configuration_object::AbstractConfigurationObject;
use crate::configuration::configuration_object::Abstractable;
use crate::configuration::observe_configuration::after_configuration_change;
use crate::configuration::observe_configuration::on_configuration_change;
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
use crate::types::util::graph_structure::oxidd_graph_structure::NodeType;
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
use super::util::drawing::layouts::layered_layout_traits::WidthLabel;
use super::util::drawing::layouts::random_test_layout::RandomTestLayout;
use super::util::drawing::layouts::sugiyama_lib_layout::SugiyamaLibLayout;
use super::util::drawing::layouts::toggle_layout::ToggleLayout;
use super::util::drawing::layouts::transition::transition_layout::TransitionLayout;
use super::util::drawing::layouts::util::color_label::Color;
use super::util::drawing::layouts::util::color_label::ColorLabel;
use super::util::drawing::layouts::util::color_label::TransparentColor;
use super::util::drawing::renderer::Renderer;
use super::util::drawing::renderers::util::Font::Font;
use super::util::drawing::renderers::webgl::edge_renderer::EdgeRenderingType;
use super::util::drawing::renderers::webgl::node_renderer::NodeRenderingColorConfig;
use super::util::drawing::renderers::webgl::util::mix_color::mix_color;
use super::util::drawing::renderers::webgl_renderer::WebglRenderer;
use super::util::graph_structure::graph_manipulators::label_adjusters::group_label_adjuster::GroupLabelAdjuster;
use super::util::graph_structure::graph_manipulators::node_presence_adjuster::NodePresenceAdjuster;
use super::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceLabel;
use super::util::graph_structure::graph_manipulators::pointer_node_adjuster::PointerLabel;
use super::util::graph_structure::graph_manipulators::pointer_node_adjuster::PointerNodeAdjuster;
use super::util::graph_structure::graph_manipulators::rc_graph::RCGraph;
use super::util::graph_structure::graph_manipulators::terminal_level_adjuster::TerminalLevelAdjuster;
use super::util::graph_structure::graph_structure::{DrawTag, EdgeType, GraphStructure};
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
        let (roots, levels) = DummyFunction::from_dddmp(&mut self.manager_ref, &dddmp);
        Some(Box::new(QDDDiagramSection::new(roots, levels)))
    }
    fn create_section_from_buddy(
        &mut self,
        data: String,
        vars: Option<String>,
    ) -> Option<Box<dyn DiagramSection>> {
        let (roots, levels) =
            DummyFunction::from_buddy(&mut self.manager_ref, &data, vars.as_deref());
        Some(Box::new(QDDDiagramSection::new(roots, levels)))
    }
    fn create_section_from_ids(
        &self,
        sources: &[(oxidd::NodeID, &Box<dyn DiagramSection>)],
    ) -> Option<Box<dyn DiagramSection>> {
        let mut levels = Vec::new();
        let roots = sources
            .iter()
            .map(|&(id, section)| {
                let root_edge = DummyEdge::new(Arc::new(id), self.manager_ref.clone());
                levels = section.get_level_labels();
                (DummyFunction(root_edge), section.get_node_labels(id))
            })
            .collect_vec();
        Some(Box::new(QDDDiagramSection::new(roots, levels)))
    }
}

pub struct QDDDiagramSection<F: Function>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    roots: Vec<(F, Vec<String>)>,
    labels: HashMap<NodeID, Vec<String>>,
    levels: Vec<String>,
}

impl<F: Function> QDDDiagramSection<F>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    fn new(roots: Vec<(F, Vec<String>)>, levels: Vec<String>) -> Self {
        let s = QDDDiagramSection {
            labels: roots
                .iter()
                .map(|(f, names)| {
                    (
                        f.with_manager_shared(|_, edge| edge.node_id()),
                        names.clone(),
                    )
                })
                .collect(),
            roots,
            levels,
        };
        console::log!(
            "init {}",
            s.labels
                .iter()
                .map(|(id, names)| format!("{}:[{}]", id, names.iter().join(",")))
                .join(", ")
        );
        s
    }
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
    fn get_level_labels(&self) -> Vec<String> {
        self.levels.clone()
    }
    fn get_node_labels(&self, node: NodeID) -> Vec<String> {
        self.labels.get(&node).cloned().unwrap_or_else(|| vec![])
    }
    fn create_drawer(&self, canvas: HtmlCanvasElement) -> Box<dyn DiagramSectionDrawer> {
        let c0 = (1.0, 0.2, 0.2);
        let c1 = (0.2, 1.0, 0.2);
        let c2 = (0.6, 0.6, 0.6);

        let select_color = ((0.3, 0.3, 1.0), 0.8);
        let partial_select_color = ((0.6, 0.0, 1.0), 0.7);
        let hover_color = ((0.0, 0.0, 1.0), 0.3);
        let partial_hover_color = ((1.0, 0.0, 0.8), 0.2);

        let font = Rc::new(Font::new(
            include_bytes!("../../resources/Roboto-Bold.ttf").to_vec(),
            1.0,
        ));
        let renderer = WebglRenderer::from_canvas(
            canvas,
            HashMap::from([
                // True edge
                (
                    EdgeType::new((), 0),
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
                        width: 0.2,
                        dash_solid: 1.0,
                        dash_transparent: 0.0, // No dashing
                    },
                ),
                // False edge
                (
                    EdgeType::new((), 1),
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
                        width: 0.2,
                        dash_solid: 0.3,
                        dash_transparent: 0.15,
                    },
                ),
                // Both edge
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
                        width: 0.15,
                        dash_solid: 1.0,
                        dash_transparent: 0.0,
                    },
                ),
            ]),
            NodeRenderingColorConfig {
                select: select_color,
                partial_select: partial_select_color,
                hover: hover_color,
                partial_hover: partial_hover_color,
            },
            font.clone(),
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
        let graph = OxiddGraphStructure::new(
            self.roots.iter().cloned().collect(),
            self.levels.clone(),
            |t| t.to_string(),
        );
        let diagram = QDDDiagramDrawer::new(graph, renderer, layout, font);
        Box::new(diagram)
    }
}

pub struct NodeData {
    color: Color,
    border_color: TransparentColor,
    width: f32,
    name: Option<String>,
}

impl ColorLabel for NodeData {
    fn get_color(&self) -> Color {
        self.color
    }

    fn get_outline_color(&self) -> TransparentColor {
        self.border_color
    }
}

impl WidthLabel for NodeData {
    fn get_width(&self) -> f32 {
        self.width
    }
}

impl Into<Option<String>> for NodeData {
    fn into(self) -> Option<String> {
        self.name
    }
}

pub struct QDDDiagramDrawer<
    T: DrawTag + Serializable<T> + 'static,
    G: GraphStructure<T, NodeLabel<String>, String> + StateStorage + 'static,
    R: Renderer<T>,
    L: LayoutRules<T, NodeData, String, GMGraph<T, G>>,
> {
    graph: MGraph<T, G>,
    group_manager: MutRcRefCell<GM<T, G>>,
    presence_adjuster: MPresenceAdjuster<T, G>,
    time: MutRcRefCell<u32>,
    drawer: MutRcRefCell<Drawer<T, NodeData, R, L, GMGraph<T, G>>>,
    config: Configuration<
        CompositeConfig<(
            LabelConfig<ChoiceConfig<PresenceRemainder>>,
            LabelConfig<ChoiceConfig<PresenceRemainder>>,
        )>,
    >,
}

type GraphLabel = PresenceLabel<PointerLabel<NodeLabel<String>>>;
type GM<T, G> = GroupManager<T, GraphLabel, String, MGraph<T, G>>;
type MGraph<T, G> = RCGraph<
    T,
    GraphLabel,
    String,
    TerminalLevelAdjuster<T, GraphLabel, String, MPresenceAdjuster<T, G>>,
>;
type MPresenceAdjuster<T, G> = RCGraph<
    T,
    GraphLabel,
    String,
    NodePresenceAdjuster<T, PointerLabel<NodeLabel<String>>, String, MPointerAdjuster<T, G>>,
>;
type MPointerAdjuster<T, G> = PointerNodeAdjuster<T, NodeLabel<String>, String, G>;
type GMGraph<T, G> = GroupLabelAdjuster<T, Vec<GraphLabel>, String, GM<T, G>, NodeData>;

impl<
        T: DrawTag + Serializable<T> + 'static,
        G: GraphStructure<T, NodeLabel<String>, String> + StateStorage + 'static,
        R: Renderer<T> + 'static,
        L: LayoutRules<T, NodeData, String, GMGraph<T, G>> + 'static,
    > QDDDiagramDrawer<T, G, R, L>
{
    pub fn new(graph: G, renderer: R, layout: L, font: Rc<Font>) -> QDDDiagramDrawer<T, G, R, L> {
        let original_roots = graph.get_roots().clone();
        let pointer_adjuster = PointerNodeAdjuster::new(
            graph,
            EdgeType {
                tag: T::default(),
                index: 2,
            },
            true,
            "".to_string(),
        );
        let presence_adjuster = RCGraph::new(NodePresenceAdjuster::new(pointer_adjuster));
        let modified_graph = RCGraph::new(TerminalLevelAdjuster::new(presence_adjuster.clone()));
        let roots = modified_graph.get_roots();
        let group_manager = MutRcRefCell::new(GroupManager::new(modified_graph.clone()));

        let grouped_graph = GMGraph::new_shared(group_manager.clone(), move |nodes| {
            // TODO: make this adjuster lazy, e.g. don't recompute for the same list of nodes

            let color = match (nodes.get(0), nodes.get(1)) {
                (
                    Some(&PresenceLabel {
                        original_label:
                            PointerLabel::Node(NodeLabel {
                                pointers: _,
                                kind: NodeType::Terminal(ref terminal),
                            }),
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
                (
                    Some(&PresenceLabel {
                        original_label: PointerLabel::Pointer(_),
                        original_id: _,
                    }),
                    None,
                ) => (0.5, 0.5, 1.0),
                (Some(_), None) => (0.1, 0.1, 0.1),
                _ => (0.7, 0.7, 0.7),
            };
            let name: Option<String> = match (nodes.get(0), nodes.get(1)) {
                (
                    Some(&PresenceLabel {
                        original_label: PointerLabel::Pointer(ref text),
                        original_id: _,
                    }),
                    None,
                ) => Some(text.clone()),
                _ => None,
            };

            NodeData {
                color,
                border_color: (0.0, 0.0, 0.0, 0.0),
                width: 1.
                    + match name {
                        Some(ref text) => font.measure_width(&text),
                        None => 0.,
                    },
                name,
            }
        });

        let terminal_config = CompositeConfig::new(
            (
                LabelConfig::new("False terminal", {
                    let mut c = ChoiceConfig::new([
                        Choice::new(PresenceRemainder::Show, "show"),
                        Choice::new(PresenceRemainder::Duplicate, "duplicate"),
                        Choice::new(PresenceRemainder::Hide, "hide"),
                    ]);
                    c.set_index(2).commit();
                    c
                }),
                LabelConfig::new(
                    "True terminal",
                    ChoiceConfig::new([
                        Choice::new(PresenceRemainder::Show, "show"),
                        Choice::new(PresenceRemainder::Duplicate, "duplicate"),
                        Choice::new(PresenceRemainder::Hide, "hide"),
                    ]),
                ),
            ),
            |(f1, f2)| vec![Box::new(f1.clone()), Box::new(f2.clone())],
        );
        let config = Configuration::new(terminal_config.clone());

        let mut out = QDDDiagramDrawer {
            group_manager,
            presence_adjuster,
            graph: modified_graph,
            time: MutRcRefCell::new(0),
            drawer: MutRcRefCell::new(Drawer::new(
                renderer,
                layout,
                MutRcRefCell::new(grouped_graph),
            )),
            config,
        };
        let from = out.create_group(vec![TargetID(TargetIDType::NodeGroupID, 0)]);
        for root in roots {
            out.create_group(vec![TargetID(TargetIDType::NodeID, root)]);
        }

        let max = 500;
        if out.group_manager.read().get_nodes_of_group(from).len() < max {
            out.reveal_all(from, max);
        }

        // Connect the config
        let drawer = out.drawer.clone();
        let time = out.time.clone();
        fn set_terminal_presence<
            T: DrawTag + Serializable<T> + 'static,
            G: GraphStructure<T, NodeLabel<String>, String> + StateStorage + 'static,
        >(
            presence_adjuster: &MPresenceAdjuster<T, G>,
            terminal: String,
            presence: PresenceRemainder,
        ) -> () {
            let mut adjuster = presence_adjuster.get();
            let terminals = adjuster.get_terminals();
            let mut terminals = terminals.iter().filter_map(|&node| {
                match adjuster.get_node_label(node).original_label {
                    PointerLabel::Node(NodeLabel {
                        pointers: _,
                        kind: NodeType::Terminal(t),
                    }) if t == terminal => Some(node),
                    _ => None,
                }
            });
            let Some(target_terminal) = terminals.next() else {
                return;
            };

            adjuster.set_node_presence(target_terminal, PresenceGroups::remainder(presence));
        }
        let false_config = terminal_config.0.clone();
        let false_presence_adjuster = out.presence_adjuster.clone();
        let _ = on_configuration_change(&terminal_config.0, move || {
            set_terminal_presence(&false_presence_adjuster, "F".into(), false_config.get());
        });
        let true_config = terminal_config.1.clone();
        let true_presence_adjuster = out.presence_adjuster.clone();
        let _ = on_configuration_change(&terminal_config.1, move || {
            set_terminal_presence(&true_presence_adjuster, "T".into(), true_config.get());
        });
        let _ = after_configuration_change(&terminal_config, move || {
            drawer.get().layout(*time.get());
        });

        out
    }

    pub fn reveal_all(&mut self, from_id: NodeGroupID, limit: usize) {
        let nodes = {
            let explored_group =
                self.create_group(vec![TargetID(TargetIDType::NodeGroupID, from_id)]);
            self.group_manager.read().get_nodes_of_group(explored_group)
        };
        let mut count = 0;
        for node_id in nodes.rev() {
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
        L: LayoutRules<T, NodeData, String, GMGraph<T, G>>,
    > DiagramSectionDrawer for QDDDiagramDrawer<T, G, R, L>
{
    fn render(&mut self, time: u32) -> () {
        *self.time.get() = time;
        self.drawer.get().render(time);
    }

    fn layout(&mut self, time: u32) -> () {
        self.drawer.get().layout(time);
    }

    fn set_transform(&mut self, width: u32, height: u32, x: f32, y: f32, scale: f32) -> () {
        self.drawer.get().set_transform(width, height, x, y, scale);
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

    fn get_nodes(&self, area: Rectangle, max_group_expansion: usize) -> Vec<NodeID> {
        self.drawer.read().get_nodes(area, max_group_expansion)
    }

    fn set_selected_nodes(&mut self, selected_ids: &[NodeID], hovered_ids: &[NodeID]) {
        self.drawer.get().select_nodes(selected_ids, hovered_ids);
    }

    fn local_nodes_to_sources(&self, nodes: &[NodeID]) -> Vec<NodeID> {
        self.graph
            .local_nodes_to_sources(nodes.iter().cloned().collect())
    }

    fn source_nodes_to_local(&self, nodes: &[NodeID]) -> Vec<NodeID> {
        self.graph
            .source_nodes_to_local(nodes.iter().cloned().collect())
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
