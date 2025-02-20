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
use std::u32;
use web_sys::console::log;

use crate::configuration::configuration::Configuration;
use crate::configuration::configuration_object::AbstractConfigurationObject;
use crate::configuration::configuration_object::Abstractable;
use crate::configuration::configuration_object::ConfigObjectGetter;
use crate::configuration::observe_configuration::after_configuration_change;
use crate::configuration::observe_configuration::on_configuration_change;
use crate::configuration::types::button_config::ButtonConfig;
use crate::configuration::types::choice_config::Choice;
use crate::configuration::types::choice_config::ChoiceConfig;
use crate::configuration::types::composite_config;
use crate::configuration::types::composite_config::CompositeConfig;
use crate::configuration::types::int_config::IntConfig;
use crate::configuration::types::label_config::LabelConfig;
use crate::configuration::types::text_output_config::TextOutputConfig;
use crate::traits::Diagram;
use crate::traits::DiagramSection;
use crate::traits::DiagramSectionDrawer;
use crate::types::util::drawing::renderers::webgl_renderer::WebglLayerStyle;
use crate::types::util::graph_structure::graph_manipulators::child_edge_adjuster::ChildEdgeAdjuster;
use crate::types::util::graph_structure::graph_manipulators::edge_to_adjuster::EdgeToAdjuster;
use crate::types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceGroups;
use crate::types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceRemainder;
use crate::types::util::graph_structure::oxidd_graph_structure::NodeType;
use crate::util::color::Color;
use crate::util::color::TransparentColor;
use crate::util::dummy_bdd::DummyEdge;
use crate::util::dummy_bdd::DummyFunction;
use crate::util::dummy_bdd::DummyManager;
use crate::util::dummy_bdd::DummyManagerRef;
use crate::util::free_id_manager::FreeIdManager;
use crate::util::logging::console;
use crate::util::rc_refcell::MutRcRefCell;
use crate::util::rectangle::Rectangle;
use crate::util::transition::Interpolatable;
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

use super::super::util::drawing::diagram_layout::LayerStyle;
use super::super::util::drawing::diagram_layout::NodeStyle;
use super::super::util::drawing::drawer::Drawer;
use super::super::util::drawing::layout_rules::LayoutRules;
use super::super::util::drawing::layouts::layer_group_sorting::average_group_alignment::AverageGroupAlignment;
use super::super::util::drawing::layouts::layer_group_sorting::ordering_group_alignment::OrderingGroupAlignment;
use super::super::util::drawing::layouts::layer_orderings::combinators::sequence_ordering::SequenceOrdering;
use super::super::util::drawing::layouts::layer_orderings::pseudo_random_layer_ordering::PseudoRandomLayerOrdering;
use super::super::util::drawing::layouts::layer_orderings::random_layer_ordering::RandomLayerOrdering;
use super::super::util::drawing::layouts::layer_orderings::sugiyama_ordering::SugiyamaOrdering;
use super::super::util::drawing::layouts::layer_positionings::brandes_kopf_positioning::BrandesKopfPositioning;
use super::super::util::drawing::layouts::layer_positionings::brandes_kopf_positioning_corrected::BrandesKopfPositioningCorrected;
use super::super::util::drawing::layouts::layer_positionings::dummy_layer_positioning::DummyLayerPositioning;
use super::super::util::drawing::layouts::layered_layout::LayeredLayout;
use super::super::util::drawing::layouts::layered_layout_traits::WidthLabel;
use super::super::util::drawing::layouts::random_test_layout::RandomTestLayout;
use super::super::util::drawing::layouts::sugiyama_lib_layout::SugiyamaLibLayout;
use super::super::util::drawing::layouts::toggle_layout::IndexedSelect;
use super::super::util::drawing::layouts::toggle_layout::ToggleLayout;
use super::super::util::drawing::layouts::toggle_layout::ToggleLayoutUnit;
use super::super::util::drawing::layouts::transition::transition_layout::TransitionLayout;
use super::super::util::drawing::renderer::Renderer;
use super::super::util::drawing::renderers::latex_renderer::LatexLayerStyle;
use super::super::util::drawing::renderers::latex_renderer::LatexNodeStyle;
use super::super::util::drawing::renderers::latex_renderer::LatexRenderer;
use super::super::util::drawing::renderers::util::Font::Font;
use super::super::util::drawing::renderers::webgl::edge_renderer::EdgeRenderingType;
use super::super::util::drawing::renderers::webgl::node_renderer::NodeRenderingColorConfig;
use super::super::util::drawing::renderers::webgl_renderer::WebglNodeStyle;
use super::super::util::drawing::renderers::webgl_renderer::WebglRenderer;
use super::super::util::graph_structure::graph_manipulators::group_presence_adjuster::GroupPresenceAdjuster;
use super::super::util::graph_structure::graph_manipulators::label_adjusters::group_label_adjuster::GroupLabelAdjuster;
use super::super::util::graph_structure::graph_manipulators::node_presence_adjuster::NodePresenceAdjuster;
use super::super::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceLabel;
use super::super::util::graph_structure::graph_manipulators::pointer_node_adjuster::PointerLabel;
use super::super::util::graph_structure::graph_manipulators::pointer_node_adjuster::PointerNodeAdjuster;
use super::super::util::graph_structure::graph_manipulators::rc_graph::RCGraph;
use super::super::util::graph_structure::graph_manipulators::terminal_level_adjuster::TerminalLevelAdjuster;
use super::super::util::graph_structure::graph_structure::{DrawTag, EdgeType, GraphStructure};
use super::super::util::graph_structure::grouped_graph_structure::GroupedGraphStructure;
use super::super::util::graph_structure::oxidd_graph_structure::NodeLabel;
use super::super::util::graph_structure::oxidd_graph_structure::OxiddGraphStructure;
use super::super::util::group_manager::GroupManager;
use super::super::util::storage::state_storage::Serializable;
use super::super::util::storage::state_storage::StateStorage;

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
        let c0 = Color(1.0, 0.2, 0.2);
        let c1 = Color(0.2, 1.0, 0.2);
        let c2 = Color(0.6, 0.6, 0.6);

        let select_color = (Color(0.3, 0.3, 1.0), 0.8);
        let partial_select_color = (Color(0.6, 0.0, 1.0), 0.7);
        let hover_color = (Color(0.0, 0.0, 1.0), 0.3);
        let partial_hover_color = (Color(1.0, 0.0, 0.8), 0.2);

        let font = Rc::new(Font::new(
            include_bytes!("../../../resources/Roboto-Bold.ttf").to_vec(),
            1.0,
        ));
        let renderer = WebglRenderer::from_canvas(
            canvas,
            HashMap::from([
                // True edge
                (
                    EdgeType::new((), 0),
                    EdgeRenderingType {
                        hover_color: c1.mix(&hover_color.0, hover_color.1),
                        select_color: c1.mix(&select_color.0, select_color.1),
                        partial_hover_color: c1.mix(&partial_hover_color.0, partial_hover_color.1),
                        partial_select_color: c1
                            .mix(&partial_select_color.0, partial_select_color.1),
                        color: c1,
                        width: 0.2,
                        dash_solid: 1.0,
                        dash_transparent: 0.0, // No dashing
                    },
                ),
                // False edge
                (
                    EdgeType::new((), 1),
                    EdgeRenderingType {
                        hover_color: c0.mix(&hover_color.0, hover_color.1),
                        select_color: c0.mix(&select_color.0, select_color.1),
                        partial_hover_color: c0.mix(&partial_hover_color.0, partial_hover_color.1),
                        partial_select_color: c0
                            .mix(&partial_select_color.0, partial_select_color.1),
                        color: c0,
                        width: 0.2,
                        dash_solid: 0.3,
                        dash_transparent: 0.15,
                    },
                ),
                // Both edge
                (
                    EdgeType::new((), 2),
                    EdgeRenderingType {
                        hover_color: c2.mix(&hover_color.0, hover_color.1),
                        select_color: c2.mix(&select_color.0, select_color.1),
                        partial_hover_color: c2.mix(&partial_hover_color.0, partial_hover_color.1),
                        partial_select_color: c2
                            .mix(&partial_select_color.0, partial_select_color.1),
                        color: c2,
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
        let layout = ToggleLayout::new(
            LayeredLayout::new(
                // SugiyamaOrdering::new(2, 2),
                SequenceOrdering::new(
                    PseudoRandomLayerOrdering::new(2, 0),
                    SugiyamaOrdering::new(2, 2),
                ),
                // AverageGroupAlignment,
                OrderingGroupAlignment,
                // BrandesKopfPositioning,
                BrandesKopfPositioningCorrected,
                // DummyLayerPositioning,
                0.3,
            ),
            ToggleLayoutUnit::new(LayeredLayout::new(
                // SugiyamaOrdering::new(2, 2),
                SequenceOrdering::new(
                    PseudoRandomLayerOrdering::new(2, 0),
                    SugiyamaOrdering::new(2, 2),
                ),
                // AverageGroupAlignment,
                OrderingGroupAlignment,
                // BrandesKopfPositioning,
                BrandesKopfPositioning,
                // DummyLayerPositioning,
                0.1,
            )),
        );
        let layout = TransitionLayout::new(layout);
        let graph = OxiddGraphStructure::new(
            self.roots.iter().cloned().collect(),
            self.levels.clone(),
            terminal_to_string,
        );
        let diagram = QDDDiagramDrawer::new(graph, renderer, layout, font);
        Box::new(diagram)
    }
}

fn terminal_to_string<T: ToString>(terminal: &T) -> String {
    terminal.to_string()
}

trait LayoutEditing {
    fn set_seed(&mut self, seed: usize) -> ();
    fn select_layout(&mut self, layout: usize) -> ();
}
impl<
        T: ToString + Clone + 'static,
        E: Edge<Tag = ()> + 'static,
        N: InnerNode<E> + HasLevel + 'static,
        R: DiagramRules<E, N, T> + 'static,
        F: Function + 'static,
        S: Fn(&T) -> String,
    > LayoutEditing
    for TransitionLayout<
        (),
        NodeData,
        LayerData,
        GMGraph<(), OxiddGraphStructure<(), F, T, S>>,
        ToggleLayout<
            (),
            NodeData,
            LayerData,
            GMGraph<(), OxiddGraphStructure<(), F, T, S>>,
            LayeredLayout<
                (),
                NodeData,
                LayerData,
                SequenceOrdering<
                    (),
                    NodeData,
                    LayerData,
                    PseudoRandomLayerOrdering,
                    SugiyamaOrdering,
                >,
                OrderingGroupAlignment,
                BrandesKopfPositioningCorrected,
            >,
            ToggleLayoutUnit<
                (),
                NodeData,
                LayerData,
                GMGraph<(), OxiddGraphStructure<(), F, T, S>>,
                LayeredLayout<
                    (),
                    NodeData,
                    LayerData,
                    SequenceOrdering<
                        (),
                        NodeData,
                        LayerData,
                        PseudoRandomLayerOrdering,
                        SugiyamaOrdering,
                    >,
                    OrderingGroupAlignment,
                    BrandesKopfPositioning,
                >,
            >,
        >,
    >
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = (), Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn set_seed(&mut self, seed: usize) -> () {
        self.get_layout_rules()
            .get_layout_rules1()
            .get_ordering()
            .get_ordering1()
            .set_seed(seed);
        self.get_layout_rules()
            .get_layout_rules2()
            .get_layout_rules()
            .get_ordering()
            .get_ordering1()
            .set_seed(seed);
    }
    fn select_layout(&mut self, layout: usize) -> () {
        self.get_layout_rules().select_layout(layout);
    }
}

#[derive(Clone)]
pub struct NodeData {
    color: Color,
    border_color: TransparentColor,
    width: f32,
    name: Option<String>,
    is_terminal: Option<usize>,
    is_group: bool,
}

impl Interpolatable for NodeData {
    fn mix(&self, other: &Self, frac: f32) -> Self {
        NodeData {
            color: self.color.mix(&other.color, frac),
            border_color: self.border_color.mix(&other.border_color, frac),
            width: self.width * (1.0 - frac) + other.width * frac,
            name: other.name.clone(),
            is_terminal: other.is_terminal.clone(),
            is_group: other.is_group,
        }
    }
}
impl LatexNodeStyle for NodeData {
    fn is_terminal(&self) -> Option<String> {
        self.is_terminal.map(|v| format!("terminal{}", v))
    }

    fn is_group(&self) -> bool {
        self.is_group
    }

    fn get_label(&self) -> Option<String> {
        self.name.clone()
    }
}
impl WebglNodeStyle for NodeData {
    fn get_color(&self) -> Color {
        self.color.clone()
    }

    fn get_outline_color(&self) -> TransparentColor {
        self.border_color.clone()
    }

    fn get_label(&self) -> Option<String> {
        self.name.clone()
    }
}
impl WidthLabel for NodeData {
    fn get_width(&self) -> f32 {
        self.width
    }
}
impl NodeStyle for NodeData {}

#[derive(Clone)]
pub struct LayerData {
    name: String,
}
impl Interpolatable for LayerData {
    fn mix(&self, other: &Self, frac: f32) -> Self {
        LayerData {
            name: self.name.clone(),
        }
    }
}
impl LayerStyle for LayerData {
    fn squash(layers: Vec<Self>) -> Self {
        LayerData {
            name: layers.into_iter().map(|s| s.name).join(", \n"),
        }
    }
}
impl WebglLayerStyle for LayerData {
    fn get_label(&self) -> String {
        self.name.clone()
    }
}
impl LatexLayerStyle for LayerData {
    fn get_label(&self) -> String {
        self.name.clone()
    }
}

pub struct QDDDiagramDrawer<
    T: DrawTag + Serializable<T> + 'static,
    G: GraphStructure<T, NodeLabel<String>, String> + StateStorage + 'static,
    R: Renderer<T, NodeData, LayerData>,
    L: LayoutRules<T, NodeData, LayerData, GMGraph<T, G>>,
> {
    graph: MGraph<T, G>,
    group_manager: MutRcRefCell<GM<T, G>>,
    presence_adjuster: MPresenceAdjuster<T, G>,
    time: MutRcRefCell<u32>,
    drawer: MutRcRefCell<Drawer<T, NodeData, LayerData, R, L, GMGraph<T, G>>>,
    config: Configuration<
        CompositeConfig<(
            LabelConfig<ChoiceConfig<usize>>,
            LabelConfig<ChoiceConfig<PresenceRemainder>>,
            LabelConfig<ChoiceConfig<PresenceRemainder>>,
            LabelConfig<ChoiceConfig<bool>>,
            LabelConfig<ChoiceConfig<bool>>,
            LabelConfig<IntConfig>,
            ButtonConfig,
            ButtonConfig,
            TextOutputConfig,
            ButtonConfig,
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
    NodePresenceAdjuster<T, PointerLabel<NodeLabel<String>>, String, MEdgeToAdjuster<T, G>>,
>;
type MEdgeToAdjuster<T, G> = RCGraph<
    T,
    PointerLabel<NodeLabel<String>>,
    String,
    EdgeToAdjuster<T, PointerLabel<NodeLabel<String>>, String, MChildEdgeAdjuster<T, G>>,
>;
type MChildEdgeAdjuster<T, G> = RCGraph<
    T,
    PointerLabel<NodeLabel<String>>,
    String,
    ChildEdgeAdjuster<T, PointerLabel<NodeLabel<String>>, String, MPointerAdjuster<T, G>>,
>;
type MPointerAdjuster<T, G> = PointerNodeAdjuster<T, NodeLabel<String>, String, MBaseGraph<T, G>>;
type MBaseGraph<T, G> = TerminalLevelAdjuster<T, NodeLabel<String>, String, G>;
type GMGraph<T, G> = GroupPresenceAdjuster<
    T,
    NodeData,
    LayerData,
    GroupLabelAdjuster<T, Vec<GraphLabel>, String, GM<T, G>, NodeData, LayerData>,
>;

fn move_shared_edge<T: DrawTag + 'static>(
    children: Vec<(EdgeType<T>, NodeID, PointerLabel<NodeLabel<String>>)>,
) -> Option<Vec<(EdgeType<T>, NodeID)>> {
    if children.len() != 3 {
        return None;
    }
    let edges = children
        .into_iter()
        .map(|(edge, to, label)| {
            if let PointerLabel::Node(NodeLabel {
                pointers: _,
                kind: NodeType::Terminal(t),
            }) = label
            {
                (t.clone(), (edge, to))
            } else {
                ("inner".to_string(), (edge, to))
            }
        })
        .collect::<HashMap<_, _>>();
    let Some((to_true_edge, true_node)) = edges.get("T") else {
        return None;
    };
    let Some((to_false_edge, false_node)) = edges.get("F") else {
        return None;
    };
    let Some((to_inner_edge, rest_node)) = edges.get("inner") else {
        return None;
    };

    if to_true_edge.index == 2 {
        return None;
    }

    return Some(vec![
        (to_true_edge.clone(), *rest_node),
        (to_inner_edge.clone(), *true_node),
        (to_false_edge.clone(), *false_node),
    ]);
    // return Some(vec![]);
}

impl<
        T: DrawTag + Serializable<T> + 'static,
        G: GraphStructure<T, NodeLabel<String>, String> + StateStorage + 'static,
        R: Renderer<T, NodeData, LayerData> + 'static,
        L: LayoutRules<T, NodeData, LayerData, GMGraph<T, G>> + LayoutEditing + 'static,
    > QDDDiagramDrawer<T, G, R, L>
{
    pub fn new(graph: G, renderer: R, layout: L, font: Rc<Font>) -> QDDDiagramDrawer<T, G, R, L> {
        let original_roots = graph.get_roots().clone();
        let base_graph = TerminalLevelAdjuster::new(graph); // Make sure that terminal levels make sense before possibly adding pointers to these terminals
        let pointer_adjuster = PointerNodeAdjuster::new(
            base_graph,
            EdgeType {
                tag: T::default(),
                index: 2,
            },
            true,
            "".to_string(),
        );
        let child_edge_adjuster =
            RCGraph::new(ChildEdgeAdjuster::new(pointer_adjuster, move_shared_edge));
        let edge_to_adjuster = RCGraph::new(EdgeToAdjuster::new(child_edge_adjuster.clone()));
        let presence_adjuster = RCGraph::new(NodePresenceAdjuster::new(edge_to_adjuster.clone()));
        let modified_graph = RCGraph::new(TerminalLevelAdjuster::new(presence_adjuster.clone()));
        let roots = modified_graph.get_roots();
        let group_manager = MutRcRefCell::new(GroupManager::new(modified_graph.clone()));

        let mut grouped_graph = GMGraph::new(GroupLabelAdjuster::new_shared(
            group_manager.clone(),
            move |nodes| {
                // TODO: make this adjuster lazy, e.g. don't recompute for the same list of nodes
                console::log!("Get group data");
                let (is_terminal, is_group, color) = match (nodes.get(0), nodes.get(1)) {
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
                            (Some(1), false, Color(0.2, 1., 0.2))
                        } else {
                            (Some(0), false, Color(1., 0.2, 0.2))
                        }
                    }
                    (
                        Some(&PresenceLabel {
                            original_label: PointerLabel::Pointer(_),
                            original_id: _,
                        }),
                        None,
                    ) => (None, false, Color(0.5, 0.5, 1.0)),
                    (Some(_), None) => (None, false, Color(0.1, 0.1, 0.1)),
                    _ => (None, true, Color(0.7, 0.7, 0.7)),
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
                    border_color: TransparentColor(0.0, 0.0, 0.0, 0.0),
                    width: 1.
                        + match name {
                            Some(ref text) => font.measure_width(&text),
                            None => 0.,
                        },
                    name,
                    is_terminal,
                    is_group,
                }
            },
            move |layer_label| LayerData {
                name: layer_label.clone(),
            },
        ));
        grouped_graph.hide(0);

        let composite_config = CompositeConfig::new(
            (
                LabelConfig::new(
                    "Layout",
                    ChoiceConfig::new([Choice::new(0, "1"), Choice::new(1, "2")]),
                ),
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
                LabelConfig::new("Hide shared true", {
                    let mut c =
                        ChoiceConfig::new([Choice::new(false, "show"), Choice::new(true, "hide")]);
                    c.set_index(1).commit();
                    c
                }),
                LabelConfig::new("Move shared", {
                    let mut c = ChoiceConfig::new([
                        Choice::new(true, "enabled"),
                        Choice::new(false, "disabled"),
                    ]);
                    c
                }),
                LabelConfig::new("Seed", IntConfig::new_min_max(0, Some(0), None)),
                ButtonConfig::new_labeled("Change seed"),
                ButtonConfig::new_labeled("Generate latex"),
                TextOutputConfig::new(true),
                ButtonConfig::new_labeled("Expand all"),
            ),
            |(f1, f2, f3, f4, f5, f6, f7, f8, f9, f10)| {
                vec![
                    Box::new(f1.clone()),
                    Box::new(f2.clone()),
                    Box::new(f3.clone()),
                    Box::new(f4.clone()),
                    Box::new(f5.clone()),
                    Box::new(f6.clone()),
                    Box::new(f7.clone()),
                    Box::new(f8.clone()),
                    Box::new(f9.clone()),
                    Box::new(f10.clone()),
                ]
            },
        );
        let config = Configuration::new(composite_config.clone());

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

        let drawer = out.drawer.clone();
        let false_config = composite_config.0.clone();
        let _ = on_configuration_change(&composite_config.0, move || {
            drawer
                .get()
                .get_layout_rules()
                .select_layout(false_config.get());
        });

        let drawer = out.drawer.clone();
        let mut seed = composite_config.5.clone();
        composite_config.6.clone().add_press_listener(move || {
            let new_seed = seed.get() + 1;
            seed.set(new_seed).commit();
        });
        let seed = composite_config.5.clone();
        let seed2 = seed.clone();
        let _ = on_configuration_change(&seed, move || {
            drawer
                .get()
                .get_layout_rules()
                .set_seed(seed2.get() as usize);
        });

        let drawer = out.drawer.clone();
        let mut latex_renderer = LatexRenderer::new();
        let mut output = composite_config.8.clone();
        composite_config.7.clone().add_press_listener(move || {
            latex_renderer.update_layout(&drawer.get().get_current_layout());
            latex_renderer.render(u32::MAX);
            let out = latex_renderer.get_output();
            output.set(out.into()).commit();
        });

        let from = out.create_group(vec![TargetID(TargetIDType::NodeGroupID, 0)]);
        // let from = 0;
        for root in roots {
            out.create_group(vec![TargetID(TargetIDType::NodeID, root)]);
        }

        let max = 500;
        if out.group_manager.read().get_nodes_of_group(from).len() < max {
            reveal_all(&out.group_manager, from, max);
        }

        let group_manager = out.group_manager.clone();
        composite_config
            .9
            .clone()
            .add_press_listener(move || reveal_all(&group_manager, from, 10_000_000));

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
        let false_config = composite_config.1.clone();
        let false_presence_adjuster = out.presence_adjuster.clone();
        let _ = on_configuration_change(&composite_config.1, move || {
            set_terminal_presence(&false_presence_adjuster, "F".into(), false_config.get());
        });
        let true_config = composite_config.2.clone();
        let true_presence_adjuster = out.presence_adjuster.clone();
        let _ = on_configuration_change(&composite_config.2, move || {
            set_terminal_presence(&true_presence_adjuster, "T".into(), true_config.get());
        });

        let hide_shared_true_config = composite_config.3.clone();
        let _ = on_configuration_change(&composite_config.3, move || {
            if hide_shared_true_config.get() {
                let hide_edges = edge_to_adjuster
                    .get_terminals()
                    .into_iter()
                    .filter_map(|node| match edge_to_adjuster.get_node_label(node) {
                        PointerLabel::Node(NodeLabel {
                            pointers: _,
                            kind: NodeType::Terminal(t),
                        }) if t == "T" => Some((node, EdgeType::new(T::default(), 2))),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .into_iter();
                edge_to_adjuster.get().set_remove_to_edges(hide_edges);
            } else {
                edge_to_adjuster
                    .get()
                    .set_remove_to_edges(HashSet::new().into_iter());
            }
        });

        let move_shared_config = composite_config.4.clone();
        on_configuration_change(&composite_config.4, move || {
            child_edge_adjuster
                .get()
                .set_enabled(move_shared_config.get());
        });

        let _ = after_configuration_change(&composite_config, move || {
            drawer.get().layout(*time.get());
        });

        out
    }
}

fn reveal_all<
    T: DrawTag + Serializable<T> + 'static,
    G: GraphStructure<T, NodeLabel<String>, String> + StateStorage + 'static,
>(
    group_manager: &MutRcRefCell<GM<T, G>>,
    from_id: NodeGroupID,
    limit: usize,
) {
    let nodes = {
        let explored_group = group_manager
            .get()
            .create_group(vec![TargetID(TargetIDType::NodeGroupID, from_id)]);
        group_manager.read().get_nodes_of_group(explored_group)
    };
    let mut count = 0;
    let mut group_manager = group_manager.get();
    for node_id in nodes.into_iter().rev() {
        // console::log!("{node_id}");
        group_manager.create_group(vec![TargetID(TargetIDType::NodeID, node_id)]);

        count = count + 1;
        if limit > 0 && count >= limit {
            break;
        }
    }
}

impl<
        T: DrawTag + Serializable<T> + 'static,
        G: GraphStructure<T, NodeLabel<String>, String> + StateStorage + 'static,
        R: Renderer<T, NodeData, LayerData>,
        L: LayoutRules<T, NodeData, LayerData, GMGraph<T, G>>,
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
        // self.config.
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
