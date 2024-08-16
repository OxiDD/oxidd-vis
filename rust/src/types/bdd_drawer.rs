use oxidd_core::DiagramRules;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::hash::Hash;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use web_sys::console::log;

use crate::traits::Diagram;
use crate::traits::DiagramDrawer;
use crate::util::free_id_manager::FreeIdManager;
use crate::util::logging::console;
use crate::util::rc_refcell::MutRcRefCell;
use crate::util::rectangle::Rectangle;
use crate::wasm_interface::NodeGroupID;
use crate::wasm_interface::NodeID;
use crate::wasm_interface::TargetID;
use crate::wasm_interface::TargetIDType;
use oxidd::bdd;
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
use oxidd_rules_bdd::simple::BDDOp;
use oxidd_rules_bdd::simple::BDDTerminal;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use super::util::drawing::drawer::Drawer;
use super::util::drawing::layout_rules::LayoutRules;
use super::util::drawing::layouts::layer_group_sorting::average_group_alignment::AverageGroupAlignment;
use super::util::drawing::layouts::layer_orderings::sugiyama_ordering::SugiyamaOrdering;
use super::util::drawing::layouts::layer_positionings::brandes_kopf_positioning::BrandesKopfPositioning;
use super::util::drawing::layouts::layer_positionings::dummy_layer_positioning::DummyLayerPositioning;
use super::util::drawing::layouts::layered_layout::LayeredLayout;
use super::util::drawing::layouts::random_test_layout::RandomTestLayout;
use super::util::drawing::layouts::sugiyama_lib_layout::SugiyamaLibLayout;
use super::util::drawing::layouts::toggle_layout::ToggleLayout;
use super::util::drawing::layouts::transition_layout::TransitionLayout;
use super::util::drawing::renderer::Renderer;
use super::util::drawing::renderers::webgl::edge_renderer::EdgeRenderingType;
use super::util::drawing::renderers::webgl_renderer::WebglRenderer;
use super::util::graph_structure::graph_structure::DrawTag;
use super::util::graph_structure::graph_structure::EdgeType;
use super::util::graph_structure::graph_structure::GraphStructure;
use super::util::graph_structure::grouped_graph_structure::GroupedGraphStructure;
use super::util::graph_structure::oxidd_graph_structure::NodeLabel;
use super::util::graph_structure::oxidd_graph_structure::OxiddGraphStructure;
use super::util::group_manager::GroupManager;

pub struct BDDDiagram<MR: ManagerRef, F: Function<ManagerRef = MR> + 'static>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    manager_ref: MR,
    root: F,
}
impl<MR: ManagerRef, F: Function<ManagerRef = MR>> BDDDiagram<MR, F>
where
    for<'id> <<F as oxidd::Function>::Manager<'id> as Manager>::InnerNode: HasLevel,
{
    pub fn new(mut manager_ref: MR, root: impl Fn(&mut MR) -> F) -> BDDDiagram<MR, F> {
        // let mut manager_ref = manager_ref;
        BDDDiagram {
            root: root(&mut manager_ref),
            manager_ref,
        }
    }
}

impl<
        T: ToString + Clone + 'static,
        E: Edge<Tag = ()> + 'static,
        N: InnerNode<E> + HasLevel + 'static,
        R: DiagramRules<E, N, T> + 'static,
        MR: ManagerRef + 'static,
        F: Function<ManagerRef = MR> + 'static,
    > Diagram for BDDDiagram<MR, F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = (), Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn create_drawer(&self, canvas: HtmlCanvasElement) -> Box<dyn DiagramDrawer> {
        let renderer = WebglRenderer::from_canvas(
            canvas,
            HashMap::from([
                (
                    EdgeType::new((), 0),
                    EdgeRenderingType {
                        color: (0., 0., 0.),
                        width: 0.15,
                        dash_solid: 1.0,
                        dash_transparent: 0.0, // No dashing, just solid
                    },
                ),
                (
                    EdgeType::new((), 1),
                    EdgeRenderingType {
                        color: (0.6, 0.6, 0.6),
                        width: 0.1,
                        dash_solid: 0.2,
                        dash_transparent: 0.1,
                    },
                ),
            ]),
        )
        .unwrap();
        let layout = LayeredLayout::new(
            SugiyamaOrdering::new(1, 1),
            AverageGroupAlignment,
            BrandesKopfPositioning,
            0.3,
        );
        let layout = TransitionLayout::new(layout);
        let graph = OxiddGraphStructure::new(self.root.clone(), |t| t.to_string());
        let diagram = BDDDiagramDrawer::new(graph, renderer, layout);
        Box::new(diagram)
    }
}

pub struct BDDDiagramDrawer<
    T: DrawTag,
    G: GraphStructure<T, NodeLabel<String>, String> + 'static,
    R: Renderer<T>,
    L: LayoutRules<T, String, String, GroupManager<T, NodeLabel<String>, String, G>>,
> {
    group_manager: MutRcRefCell<GroupManager<T, NodeLabel<String>, String, G>>,
    drawer: Drawer<T, R, L, GroupManager<T, NodeLabel<String>, String, G>>,
}

impl<
        T: DrawTag + 'static,
        G: GraphStructure<T, NodeLabel<String>, String> + 'static,
        R: Renderer<T>,
        L: LayoutRules<T, String, String, GroupManager<T, NodeLabel<String>, String, G>>,
    > BDDDiagramDrawer<T, G, R, L>
{
    pub fn new(graph: G, renderer: R, layout: L) -> BDDDiagramDrawer<T, G, R, L> {
        let root = graph.get_root();
        let group_manager = MutRcRefCell::new(GroupManager::new(graph));
        let mut out = BDDDiagramDrawer {
            group_manager: group_manager.clone(),
            drawer: Drawer::new(
                renderer,
                // Box::new(TransitionLayout::new(Box::new(RandomTestLayout))),
                // Box::new(TransitionLayout::new(Box::new(SugiyamaLibLayout::new()))),
                // Box::new(TransitionLayout::new(Box::new(LayeredLayout::new(
                //     Box::new(SugiyamaOrdering::new(1, 1)),
                //     Box::new(BrandesKopfPositioning),
                //     // Box::new(DummyLayerPositioning),
                // )))),
                layout,
                // Box::new(TransitionLayout::new(Box::new(ToggleLayout::new(vec![
                //     Box::new(RandomTestLayout),
                //     Box::new(LayeredLayout::new(
                //         Box::new(SugiyamaOrdering::new(1, 1)),
                //         Box::new(BrandesKopfPositioning),
                //         // Box::new(DummyLayerPositioning),
                //     )),
                // ])))),
                // Box::new(TransitionLayout::new(Box::new(ToggleLayout::new(vec![
                //     Box::new(RandomTestLayout),
                //     Box::new(SugiyamaLayout),
                // ])))),
                group_manager,
            ),
        };
        out.create_group(vec![TargetID(TargetIDType::NodeGroupID, 0)]);
        out.create_group(vec![TargetID(TargetIDType::NodeID, root)]);
        out.reveal_all(30000);
        out
    }

    pub fn reveal_all(&mut self, limit: u32) {
        let nodes = {
            let explored_group = self.create_group(vec![TargetID(TargetIDType::NodeGroupID, 0)]);
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
        T: DrawTag + 'static,
        G: GraphStructure<T, NodeLabel<String>, String> + 'static,
        R: Renderer<T>,
        L: LayoutRules<T, String, String, GroupManager<T, NodeLabel<String>, String, G>>,
    > DiagramDrawer for BDDDiagramDrawer<T, G, R, L>
{
    fn render(&mut self, time: u32, selected_ids: &[u32], hovered_ids: &[u32]) -> () {
        // let children = self.get_children(&self.root);
        // for (_, child) in children {
        //     let c: F = child;

        //     let level = c.with_manager_shared(|mgr, f| mgr.get_node(f).unwrap_inner().level());
        //     console::log!("{}", level);
        // }
        self.drawer.render(time, selected_ids, hovered_ids);
    }

    fn layout(&mut self, time: u32) -> () {
        self.drawer.layout(time);
    }

    fn set_transform(&mut self, width: u32, height: u32, x: f32, y: f32, scale: f32) -> () {
        self.drawer.set_transform(width, height, x, y, scale);
    }

    fn set_step(&mut self, step: i32) -> Option<crate::wasm_interface::StepData> {
        todo!()
    }

    fn set_group(
        &mut self,
        from: Vec<crate::wasm_interface::TargetID>,
        to: crate::wasm_interface::NodeGroupID,
    ) -> bool {
        self.group_manager.get().set_group(from, to)
    }

    fn create_group(
        &mut self,
        from: Vec<crate::wasm_interface::TargetID>,
    ) -> crate::wasm_interface::NodeGroupID {
        self.group_manager.get().create_group(from)
    }

    fn split_edges(&mut self, group: NodeGroupID, fully: bool) {
        self.group_manager.get().split_edges(group, fully);
    }

    fn get_nodes(&self, area: Rectangle) -> Vec<crate::wasm_interface::NodeGroupID> {
        self.drawer.get_nodes(area)
    }
}
