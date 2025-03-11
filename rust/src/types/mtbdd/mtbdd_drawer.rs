use itertools::Itertools;
use std::{collections::HashMap, io::Cursor, rc::Rc};
use web_sys::HtmlCanvasElement;

use oxidd::{Edge, Function, InnerNode, Manager, ManagerRef, NodeID};
use oxidd_core::{DiagramRules, HasLevel};

use crate::{
    configuration::{
        configuration::Configuration,
        configuration_object::{AbstractConfigurationObject, Abstractable},
        types::{
            button_config::ButtonConfig, composite_config::CompositeConfig,
            text_output_config::TextOutputConfig,
        },
    },
    traits::{Diagram, DiagramSection, DiagramSectionDrawer},
    types::{
        qdd::qdd_drawer::QDDDiagramDrawer,
        util::{
            drawing::{
                diagram_layout::{LayerStyle, NodeStyle},
                drawer::Drawer,
                layout_rules::LayoutRules,
                layouts::{
                    layer_group_sorting::ordering_group_alignment::OrderingGroupAlignment,
                    layer_orderings::{
                        combinators::sequence_ordering::SequenceOrdering,
                        edge_layer_ordering::EdgeLayerOrdering,
                        pseudo_random_layer_ordering::PseudoRandomLayerOrdering,
                        sugiyama_ordering::SugiyamaOrdering,
                    },
                    layer_positionings::brandes_kopf_positioning_corrected::BrandesKopfPositioningCorrected,
                    layered_layout::LayeredLayout,
                    layered_layout_traits::WidthLabel,
                    transition::transition_layout::TransitionLayout,
                },
                renderer::Renderer,
                renderers::{
                    latex_renderer::{LatexLayerStyle, LatexNodeStyle, LatexRenderer},
                    util::Font::Font,
                    webgl::{
                        edge_renderer::EdgeRenderingType, node_renderer::NodeRenderingColorConfig,
                    },
                    webgl_renderer::{WebglLayerStyle, WebglNodeStyle, WebglRenderer},
                },
            },
            graph_structure::{
                graph_manipulators::{
                    group_presence_adjuster::GroupPresenceAdjuster,
                    label_adjusters::group_label_adjuster::GroupLabelAdjuster,
                    pointer_node_adjuster::{PointerLabel, PointerNodeAdjuster},
                    rc_graph::RCGraph,
                    terminal_level_adjuster::TerminalLevelAdjuster,
                },
                graph_structure::{DrawTag, EdgeType, GraphStructure},
                grouped_graph_structure::GroupedGraphStructure,
                oxidd_graph_structure::{NodeLabel, NodeType, OxiddGraphStructure},
            },
            group_manager::GroupManager,
            storage::state_storage::{Serializable, StateStorage},
        },
    },
    util::{
        color::{Color, TransparentColor},
        dummy_mtbdd::{DummyMTBDDFunction, DummyMTBDDManager, DummyMTBDDManagerRef},
        logging::console,
        rc_refcell::MutRcRefCell,
        rectangle::Rectangle,
        transition::Interpolatable,
    },
    wasm_interface::{NodeGroupID, StepData, TargetID, TargetIDType},
};

pub struct MTBDDDiagram<MR: ManagerRef>
where
    for<'id> <<MR as oxidd::ManagerRef>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    manager_ref: MR,
}
impl MTBDDDiagram<DummyMTBDDManagerRef> {
    pub fn new() -> MTBDDDiagram<DummyMTBDDManagerRef> {
        let manager_ref = DummyMTBDDManagerRef::from(&DummyMTBDDManager::new());
        MTBDDDiagram { manager_ref }
    }
}

impl Diagram for MTBDDDiagram<DummyMTBDDManagerRef> {
    fn create_section_from_dddmp(
        &mut self,
        dddmp: String,
    ) -> Option<Box<dyn crate::traits::DiagramSection>> {
        let (roots, levels) = DummyMTBDDFunction::from_dddmp(&mut self.manager_ref, &dddmp);
        Some(Box::new(MTBDDDiagramSection::new(roots, levels)))
    }

    fn create_section_from_buddy(
        &mut self,
        data: String,
        vars: Option<String>,
    ) -> Option<Box<dyn crate::traits::DiagramSection>> {
        todo!()
    }

    fn create_section_from_ids(
        &self,
        id: &[(oxidd::NodeID, &Box<dyn crate::traits::DiagramSection>)],
    ) -> Option<Box<dyn crate::traits::DiagramSection>> {
        todo!()
    }
}

pub struct MTBDDDiagramSection<F: Function>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    roots: Vec<(F, Vec<String>)>,
    labels: HashMap<NodeID, Vec<String>>,
    levels: Vec<String>,
}
impl<F: Function> MTBDDDiagramSection<F>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    fn new(roots: Vec<(F, Vec<String>)>, levels: Vec<String>) -> Self {
        let s = MTBDDDiagramSection {
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
        E: Edge<Tag = ()> + 'static,
        N: InnerNode<E> + HasLevel + 'static,
        R: DiagramRules<E, N, i32> + 'static,
        F: Function + 'static,
    > DiagramSection for MTBDDDiagramSection<F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = (), Edge = E, InnerNode = N, Rules = R, Terminal = i32>,
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
        let layout = LayeredLayout::new(
            // SugiyamaOrdering::new(2, 2),
            SequenceOrdering::new(EdgeLayerOrdering, SugiyamaOrdering::new(2, 2)),
            // AverageGroupAlignment,
            OrderingGroupAlignment,
            // BrandesKopfPositioning,
            BrandesKopfPositioningCorrected,
            // DummyLayerPositioning,
            0.3,
        );
        let layout = TransitionLayout::new(layout);
        let graph = OxiddGraphStructure::new(
            self.roots.iter().cloned().collect(),
            self.levels.clone(),
            terminal_to_string,
        );
        let diagram = MTBDDDiagramDrawer::new(graph, renderer, layout, font);
        Box::new(diagram)
    }
}

fn terminal_to_string<T: ToString>(terminal: &T) -> String {
    terminal.to_string()
}

#[derive(Clone)]
pub struct NodeData {
    color: Color,
    border_color: TransparentColor,
    width: f32,
    name: Option<String>,
    is_terminal: Option<i32>,
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
    fn is_terminal(&self) -> Option<(String, Option<String>)> {
        self.is_terminal
            .map(|v| ("terminal".to_string(), Some(format!("{}", v))))
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

pub struct MTBDDDiagramDrawer<
    T: DrawTag + Serializable<T> + 'static,
    G: GraphStructure<T, NodeLabel<i32>, String> + StateStorage + 'static,
    R: Renderer<T, NodeData, LayerData>,
    L: LayoutRules<T, NodeData, LayerData, GMGraph<T, G>>,
> {
    graph: MGraph<T, G>,
    group_manager: MutRcRefCell<GM<T, G>>,
    time: MutRcRefCell<u32>,
    drawer: MutRcRefCell<Drawer<T, NodeData, LayerData, R, L, GMGraph<T, G>>>,
    config: Configuration<CompositeConfig<(ButtonConfig, TextOutputConfig, ButtonConfig)>>,
}

type GraphLabel = PointerLabel<NodeLabel<i32>>;
type GM<T, G> = GroupManager<T, GraphLabel, String, MGraph<T, G>>;
type MGraph<T, G> = RCGraph<T, GraphLabel, String, MPointerAdjuster<T, G>>;
type MPointerAdjuster<T, G> = PointerNodeAdjuster<T, NodeLabel<i32>, String, MBaseGraph<T, G>>;
type MBaseGraph<T, G> = TerminalLevelAdjuster<T, NodeLabel<i32>, String, G>;
type GMGraph<T, G> = GroupPresenceAdjuster<
    T,
    NodeData,
    LayerData,
    GroupLabelAdjuster<T, Vec<GraphLabel>, String, GM<T, G>, NodeData, LayerData>,
>;

impl<
        T: DrawTag + Serializable<T> + 'static,
        G: GraphStructure<T, NodeLabel<i32>, String> + StateStorage + 'static,
        R: Renderer<T, NodeData, LayerData> + 'static,
        L: LayoutRules<T, NodeData, LayerData, GMGraph<T, G>> + 'static,
    > MTBDDDiagramDrawer<T, G, R, L>
{
    pub fn new(graph: G, renderer: R, layout: L, font: Rc<Font>) -> Self {
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
        let modified_graph = RCGraph::new(pointer_adjuster);
        let roots = modified_graph.get_roots();
        let group_manager = MutRcRefCell::new(GroupManager::new(modified_graph.clone()));

        let mut grouped_graph = GMGraph::new(GroupLabelAdjuster::new_shared(
            group_manager.clone(),
            move |nodes| {
                let (is_terminal, is_group, color) = match (nodes.get(0), nodes.get(1)) {
                    (
                        Some(PointerLabel::Node(NodeLabel {
                            pointers: _,
                            kind: NodeType::Terminal(ref terminal),
                        })),
                        None,
                    ) => (Some(*terminal), false, Color(0.2, 1., 0.2)),
                    (Some(PointerLabel::Pointer(_)), None) => (None, false, Color(0.5, 0.5, 1.0)),
                    (Some(_), None) => (None, false, Color(0.1, 0.1, 0.1)),
                    _ => (None, true, Color(0.7, 0.7, 0.7)),
                };
                let name: Option<String> = match (nodes.get(0), nodes.get(1)) {
                    (Some(PointerLabel::Pointer(ref text)), None) => Some(text.clone()),
                    _ => None,
                }
                .or_else(|| is_terminal.map(|t| format!("{}", t)));

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
                ButtonConfig::new_labeled("Generate latex"),
                TextOutputConfig::new(true),
                ButtonConfig::new_labeled("Expand all"),
            ),
            |(f1, f2, f3)| {
                vec![
                    Box::new(f1.clone()),
                    Box::new(f2.clone()),
                    Box::new(f3.clone()),
                ]
            },
        );
        let config = Configuration::new(composite_config.clone());

        let mut out = MTBDDDiagramDrawer {
            group_manager,
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
        let mut latex_renderer = LatexRenderer::new();
        let mut output = composite_config.1.clone();
        composite_config.0.clone().add_press_listener(move || {
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

        let group_manager = out.group_manager.clone();
        composite_config
            .2
            .clone()
            .add_press_listener(move || reveal_all(&group_manager, from, 10_000_000));

        out
    }
}

fn reveal_all<
    T: DrawTag + Serializable<T> + 'static,
    G: GraphStructure<T, NodeLabel<i32>, String> + StateStorage + 'static,
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
        G: GraphStructure<T, NodeLabel<i32>, String> + StateStorage + 'static,
        R: Renderer<T, NodeData, LayerData>,
        L: LayoutRules<T, NodeData, LayerData, GMGraph<T, G>>,
    > DiagramSectionDrawer for MTBDDDiagramDrawer<T, G, R, L>
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
