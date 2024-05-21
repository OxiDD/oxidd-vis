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
use super::util::drawing::layouts::random_test_layout::RandomTestLayout;
use super::util::drawing::layouts::sugiyama_layout::SugiyamaLayout;
use super::util::drawing::layouts::toggle_layout::ToggleLayout;
use super::util::drawing::layouts::transition_layout::TransitionLayout;
use super::util::drawing::renderer::Renderer;
use super::util::drawing::renderers::webgl_renderer::WebglRenderer;
use super::util::edge_type::EdgeType;
use super::util::graph_structure::GraphStructure;
use super::util::graph_structure::OxiddGraphStructure;
use super::util::group_manager::GroupManager;
use super::util::grouped_graph_structure::GroupedGraphStructure;

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
        ET: Tag + 'static,
        T: 'static,
        E: Edge<Tag = ET> + 'static,
        N: InnerNode<E> + HasLevel + 'static,
        R: DiagramRules<E, N, T> + 'static,
        MR: ManagerRef + 'static,
        F: Function<ManagerRef = MR> + 'static,
    > Diagram for BDDDiagram<MR, F>
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn create_drawer(&self, canvas: HtmlCanvasElement) -> Box<dyn DiagramDrawer> {
        let renderer = Box::new(WebglRenderer::from_canvas(canvas).unwrap());
        let graph = OxiddGraphStructure::new(self.root.clone());
        let diagram = BDDDiagramDrawer::new(Box::new(graph), renderer);
        Box::new(diagram)
    }
}

pub struct BDDDiagramDrawer<T: Tag> {
    group_manager: Rc<RefCell<GroupManager<T>>>,
    drawer: Drawer<T>,
}

impl<T: Tag + 'static> BDDDiagramDrawer<T> {
    pub fn new(
        graph: Box<dyn GraphStructure<T>>,
        renderer: Box<dyn Renderer<T>>,
    ) -> BDDDiagramDrawer<T> {
        let group_manager = Rc::new(RefCell::new(GroupManager::new(graph)));
        let mut out = BDDDiagramDrawer {
            group_manager: group_manager.clone(),
            drawer: Drawer::new(
                renderer,
                // Box::new(TransitionLayout::new(Box::new(RandomTestLayout))),
                Box::new(TransitionLayout::new(Box::new(SugiyamaLayout))),
                // Box::new(TransitionLayout::new(Box::new(ToggleLayout::new(vec![
                //     Box::new(RandomTestLayout),
                //     Box::new(SugiyamaLayout),
                // ])))),
                group_manager,
            ),
        };
        out.reveal_all(30000);
        out
    }

    pub fn reveal_all(&mut self, limit: u32) {
        let nodes = {
            let explored_group = self.create_group(vec![TargetID(TargetIDType::NodeGroupID, 0)]);
            let groups = (*self.group_manager).borrow_mut();
            groups.get_nodes_of_group(explored_group)
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

impl<T: Tag> DiagramDrawer for BDDDiagramDrawer<T> {
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

    fn set_transform(&mut self, x: f32, y: f32, scale: f32) -> () {
        self.drawer.set_transform(x, y, scale);
    }

    fn set_step(&mut self, step: i32) -> Option<crate::wasm_interface::StepData> {
        todo!()
    }

    fn set_group(
        &mut self,
        from: Vec<crate::wasm_interface::TargetID>,
        to: crate::wasm_interface::NodeGroupID,
    ) -> bool {
        (*self.group_manager).borrow_mut().set_group(from, to)
    }

    fn create_group(
        &mut self,
        from: Vec<crate::wasm_interface::TargetID>,
    ) -> crate::wasm_interface::NodeGroupID {
        (*self.group_manager).borrow_mut().create_group(from)
    }

    fn get_nodes(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Vec<crate::wasm_interface::NodeGroupID> {
        todo!()
        // (*self.group_manager)
        //     .borrow_mut()
        //     .get_nodes(x, y, width, height)
    }
}
