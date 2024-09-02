use itertools::Itertools;
use oxidd::util::OutOfMemory;
use oxidd::{util::Borrowed, Edge, InnerNode, Manager, ManagerRef};
use oxidd::{BooleanFunction, Function};
use oxidd_manager_index::node::fixed_arity::NodeWithLevel;
use oxidd_rules_bdd::simple::BDDTerminal;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::convert::TryInto;
use std::hash::Hasher;
use std::hash::{DefaultHasher, Hash};
use std::iter::Cloned;
use std::rc::Rc;
use std::slice::Iter;
use std::sync::Arc;

use oxidd_core::util::DropWith;
use oxidd_core::util::{AllocResult, BorrowedEdgeIter};
use oxidd_core::DiagramRules;
use oxidd_core::LevelNo;
use oxidd_core::LevelView;
use oxidd_core::Node;
use oxidd_core::NodeID;
use oxidd_core::ReducedOrNew;
use oxidd_core::WorkerManager;
use oxidd_core::{BroadcastContext, HasLevel};

use crate::util::logging::console;

// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Clone, PartialEq, Eq)]
pub struct DummyManagerRef(Rc<RefCell<DummyManager>>);

impl Hash for DummyManagerRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}
impl<'a> From<&'a DummyManager> for DummyManagerRef {
    fn from(value: &'a DummyManager) -> Self {
        DummyManagerRef(Rc::new(RefCell::new(value.clone())))
    }
}
impl ManagerRef for DummyManagerRef {
    type Manager<'id> = DummyManager;

    fn with_manager_shared<F, T>(&self, f: F) -> T
    where
        F: for<'id> FnOnce(&Self::Manager<'id>) -> T,
    {
        f(&self.0.borrow())
    }

    fn with_manager_exclusive<F, T>(&self, f: F) -> T
    where
        F: for<'id> FnOnce(&mut Self::Manager<'id>) -> T,
    {
        f(&mut self.0.borrow_mut())
    }
}

#[derive(Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DummyFunction(pub DummyEdge);
impl DummyFunction {
    pub fn from(manager_ref: &mut DummyManagerRef, data: &str) -> DummyFunction {
        manager_ref.with_manager_exclusive(|manager| {
            let mut root = Option::None;
            let transition_texts = data.split(",");
            let edges = transition_texts.flat_map(|item| {
                let trans = item.split(">");
                let mut out = Vec::new();
                let mut prev_node = Option::None;
                for node in trans {
                    let node: NodeID = node.trim().parse().unwrap();

                    if let Some(prev) = prev_node {
                        out.push((prev, node.clone()));
                    }
                    prev_node = Some(node);
                }
                out
            });
            for (from, to) in edges.clone() {
                if root == None {
                    root = Some(from.clone());
                }
                manager.add_node(from);
                manager.add_node(to);
            }
            for (from, to) in edges {
                manager.add_edge(from, to, manager_ref.clone());
            }

            DummyFunction(DummyEdge::new(Arc::new(root.unwrap()), manager_ref.clone()))
        })
    }
    pub fn from_dddmp(manager_ref: &mut DummyManagerRef, data: &str) -> DummyFunction {
        manager_ref.with_manager_exclusive(|manager| {
            console::log!("Started loading graph");
            let mut terminals = HashMap::new();
            let node_text = &data[data.find(".nodes").unwrap()..data.find(".end").unwrap()];
            let nodes_data = node_text.split("\n").filter_map(|node| {
                let parts = node.trim().split(" ").collect::<Vec<&str>>();
                if parts.len() >= 4 {
                    let id: NodeID = parts[0].parse().unwrap();
                    let level = parts[1];
                    let children = parts[2..].iter().map(|v| v.parse().unwrap()).collect_vec();
                    Some((id, level, children))
                } else {
                    None
                }
            });
            let mut root = Option::None;
            let mut max_level = 0;
            for (_, level, _) in nodes_data.clone() {
                let Ok(level) = level.parse() else { continue };

                if level > max_level {
                    max_level = level;
                }
            }

            for (id, level, children) in nodes_data.clone() {
                let level_num = level.parse();
                manager.add_node_level(
                    id.clone(),
                    if let Ok(level) = level_num {
                        level
                    } else {
                        max_level + 1 // Terminal nodes don't define a level, we have to assign it
                    },
                    if level_num.is_ok() {
                        None
                    } else {
                        Some(level.to_string())
                    },
                );

                if level_num.is_err() {
                    terminals.insert(
                        level.to_string(),
                        DummyEdge::new(Arc::new(id), manager_ref.clone()),
                    );
                }

                if level_num == Ok(0) {
                    root = Some(id);
                }
            }

            for (id, level, children) in nodes_data {
                if manager.has_edges(id) {
                    continue; // This node was already loaded
                }
                if level.parse::<i32>().is_err() {
                    continue;
                }; // Filter out terminals

                let is_terminal = |_: NodeID| false;
                // let is_terminal = |to: NodeID| to == 1 || to == 2;
                // let is_terminal = |to: NodeID| to == 1; // Only filter connections to false

                for child in children {
                    if !is_terminal(child) {
                        manager.add_edge(id.clone(), child, manager_ref.clone());
                    }
                }
            }

            manager.init_terminals(terminals);

            console::log!("Loaded graph!");
            DummyFunction(DummyEdge::new(Arc::new(root.unwrap()), manager_ref.clone()))
        })
    }
}

unsafe impl Function for DummyFunction {
    type Manager<'id> = DummyManager;

    type ManagerRef = DummyManagerRef;
    fn from_edge<'id>(
        manager: &Self::Manager<'id>,
        edge: oxidd_core::function::EdgeOfFunc<'id, Self>,
    ) -> Self {
        DummyFunction(edge)
    }

    fn as_edge<'id>(
        &self,
        manager: &Self::Manager<'id>,
    ) -> &oxidd_core::function::EdgeOfFunc<'id, Self> {
        &self.0
    }

    fn into_edge<'id>(
        self,
        manager: &Self::Manager<'id>,
    ) -> oxidd_core::function::EdgeOfFunc<'id, Self> {
        self.0
    }

    fn manager_ref(&self) -> Self::ManagerRef {
        todo!()
    }

    fn with_manager_shared<F, T>(&self, f: F) -> T
    where
        F: for<'id> FnOnce(&Self::Manager<'id>, &oxidd_core::function::EdgeOfFunc<'id, Self>) -> T,
    {
        self.0
             .1
            .with_manager_shared(|manager| f(manager, self.as_edge(manager)))
    }

    fn with_manager_exclusive<F, T>(&self, f: F) -> T
    where
        F: for<'id> FnOnce(
            &mut Self::Manager<'id>,
            &oxidd_core::function::EdgeOfFunc<'id, Self>,
        ) -> T,
    {
        self.0
             .1
            .with_manager_exclusive(|manager| f(manager, self.as_edge(manager)))
    }
}

/// Simple dummy edge implementation based on [`Arc`]
///
/// The implementation is very limited but perfectly fine to test e.g. an apply
/// cache.
#[derive(Clone)]
pub struct DummyEdge(Arc<NodeID>, DummyManagerRef);

impl PartialEq for DummyEdge {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for DummyEdge {}
impl PartialOrd for DummyEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for DummyEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        Arc::as_ptr(&self.0).cmp(&Arc::as_ptr(&other.0))
    }
}
impl Hash for DummyEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl Drop for DummyEdge {
    fn drop(&mut self) {
        eprintln!(
            "Edges must not be dropped. Use Manager::drop_edge(). Backtrace:\n{}",
            std::backtrace::Backtrace::capture()
        );
    }
}

impl DummyEdge {
    /// Create a new `DummyEdge`
    pub fn new(to: Arc<NodeID>, mr: DummyManagerRef) -> Self {
        DummyEdge(to, mr.clone())
    }
}

impl Edge for DummyEdge {
    type Tag = ();

    fn borrowed(&self) -> Borrowed<'_, Self> {
        let ptr = Arc::as_ptr(&self.0);
        Borrowed::new(DummyEdge(unsafe { Arc::from_raw(ptr) }, self.1.clone()))
    }
    fn with_tag(&self, _tag: ()) -> Borrowed<'_, Self> {
        let ptr = Arc::as_ptr(&self.0);
        Borrowed::new(DummyEdge(unsafe { Arc::from_raw(ptr) }, self.1.clone()))
    }
    fn with_tag_owned(self, _tag: ()) -> Self {
        self
    }
    fn tag(&self) -> Self::Tag {}

    fn node_id(&self) -> NodeID {
        *self.0
    }
}

/// Dummy manager that does not actually manage anything. It is only useful to
/// clone and drop edges.
// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Clone, PartialEq, Eq)]
pub struct DummyManager(BTreeMap<NodeID, DummyNode>, HashMap<String, DummyEdge>);
impl DummyManager {
    pub fn new() -> DummyManager {
        DummyManager(BTreeMap::new(), HashMap::new())
    }
    fn init_terminals(&mut self, terminals: HashMap<String, DummyEdge>) {
        self.1.extend(terminals);
    }
}
impl Hash for DummyManager {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Dummy diagram rules
pub struct DummyRules;
impl DiagramRules<DummyEdge, DummyNode, String> for DummyRules {
    // type Cofactors<'a> = Iter<'a, Borrowed<'a, DummyEdge>>;
    type Cofactors<'a> = <DummyNode as InnerNode<DummyEdge>>::ChildrenIter<'a> where DummyNode: 'a, DummyEdge: 'a;

    fn reduce<M>(
        _manager: &M,
        level: LevelNo,
        children: impl IntoIterator<Item = DummyEdge>,
    ) -> ReducedOrNew<DummyEdge, DummyNode>
    where
        M: Manager<Edge = DummyEdge, InnerNode = DummyNode>,
    {
        ReducedOrNew::New(DummyNode::new(level, children), ())
    }

    fn cofactors(_tag: (), node: &DummyNode) -> Self::Cofactors<'_> {
        node.children()
    }
}

impl DummyManager {
    fn add_node_level(
        &mut self,
        from: NodeID,
        level: LevelNo,
        terminal: Option<String>,
    ) -> &mut DummyNode {
        self.0.entry(from).or_insert_with(|| {
            if terminal.is_some() {
                DummyNode(level, Vec::new(), terminal)
            } else {
                DummyNode::new(level, Vec::new())
            }
        })
    }
    fn add_node(&mut self, from: NodeID) -> &mut DummyNode {
        self.add_node_level(from, from.try_into().unwrap(), None)
    }
    fn add_edge(&mut self, from: NodeID, to: NodeID, mr: DummyManagerRef) {
        let from_children = &mut self.0.get_mut(&from).unwrap().1;
        let edge = DummyEdge::new(Arc::new(to), mr);
        from_children.push(edge);
    }
    fn has_edges(&self, node: NodeID) -> bool {
        let from_children = &self.0.get(&node).unwrap().1;
        from_children.len() > 0
    }
}

unsafe impl Manager for DummyManager {
    type Edge = DummyEdge;
    type EdgeTag = ();
    type InnerNode = DummyNode;
    type Terminal = String;
    type TerminalRef<'a> = &'a String;
    type TerminalIterator<'a> = Cloned<std::collections::hash_map::Values<'a, String, DummyEdge>> where Self: 'a;
    type Rules = DummyRules;
    type NodeSet = HashSet<NodeID>;
    type LevelView<'a> = DummyLevelView where Self: 'a;
    type LevelIterator<'a> = std::iter::Empty<DummyLevelView> where Self: 'a;

    fn get_node(&self, edge: &Self::Edge) -> Node<Self> {
        let to_node = self
            .0
            .get(&*edge.0)
            .expect("Edge should refer to defined node");
        if let Some(terminal) = &to_node.2 {
            Node::Terminal(terminal)
        } else {
            Node::Inner(to_node)
        }
    }

    fn clone_edge(&self, edge: &Self::Edge) -> Self::Edge {
        DummyEdge(edge.0.clone(), edge.1.clone())
    }

    fn drop_edge(&self, edge: Self::Edge) {
        // Move the inner arc out. We need to use `std::ptr::read` since
        // `DummyEdge` implements `Drop` (to print an error).
        let inner = unsafe { std::ptr::read(&edge.0) };
        std::mem::forget(edge);
        drop(inner);
    }

    fn num_inner_nodes(&self) -> usize {
        0
    }

    fn num_levels(&self) -> LevelNo {
        0
    }

    fn add_level(
        &mut self,
        _f: impl FnOnce(LevelNo) -> Self::InnerNode,
    ) -> AllocResult<Self::Edge> {
        unimplemented!()
    }

    fn level(&self, _no: LevelNo) -> Self::LevelView<'_> {
        panic!("out of range")
    }

    fn levels(&self) -> Self::LevelIterator<'_> {
        std::iter::empty()
    }

    fn get_terminal(&self, terminal: Self::Terminal) -> AllocResult<Self::Edge> {
        if let Some(terminal) = self.1.get(&terminal) {
            AllocResult::Ok(terminal.clone())
        } else {
            AllocResult::Err(OutOfMemory)
        }
    }

    fn num_terminals(&self) -> usize {
        self.1.len()
    }

    fn terminals(&self) -> Self::TerminalIterator<'_> {
        self.1.values().into_iter().cloned()
    }

    fn gc(&self) -> usize {
        0
    }

    fn reorder<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        f(self)
    }

    fn reorder_count(&self) -> u64 {
        0
    }
}

/// Dummy level view (not constructible)
pub struct DummyLevelView;

unsafe impl LevelView<DummyEdge, DummyNode> for DummyLevelView {
    type Iterator<'a> = std::iter::Empty<&'a DummyEdge>
    where
        Self: 'a,
        DummyEdge: 'a;

    type Taken = Self;

    fn len(&self) -> usize {
        unreachable!()
    }

    fn level_no(&self) -> LevelNo {
        unreachable!()
    }

    fn reserve(&mut self, _additional: usize) {
        unreachable!()
    }

    fn get(&self, _node: &DummyNode) -> Option<&DummyEdge> {
        unreachable!()
    }

    fn insert(&mut self, _edge: DummyEdge) -> bool {
        unreachable!()
    }

    fn get_or_insert(&mut self, _node: DummyNode) -> AllocResult<DummyEdge> {
        unreachable!()
    }

    unsafe fn gc(&mut self) {
        unreachable!()
    }

    unsafe fn remove(&mut self, _node: &DummyNode) -> bool {
        unreachable!()
    }

    unsafe fn swap(&mut self, _other: &mut Self) {
        unreachable!()
    }

    fn iter(&self) -> Self::Iterator<'_> {
        unreachable!()
    }

    fn take(&mut self) -> Self::Taken {
        unreachable!()
    }
}

/// Dummy node
#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct DummyNode(LevelNo, Vec<DummyEdge>, Option<String>);

impl DropWith<DummyEdge> for DummyNode {
    fn drop_with(self, _drop_edge: impl Fn(DummyEdge)) {
        unimplemented!()
    }
}

unsafe impl HasLevel for DummyNode {
    fn level(&self) -> LevelNo {
        self.0
    }

    unsafe fn set_level(&self, _level: LevelNo) {
        unimplemented!()
    }
}

impl InnerNode<DummyEdge> for DummyNode {
    const ARITY: usize = 0;

    // type ChildrenIter<'a> = std::iter::Empty<Borrowed<'a, DummyEdge>>
    // where
    //     Self: 'a;
    type ChildrenIter<'a> = BorrowedEdgeIter<'a, DummyEdge, Iter<'a, DummyEdge>> where Self: 'a;

    fn new(level: LevelNo, children: impl IntoIterator<Item = DummyEdge>) -> Self {
        DummyNode(level, children.into_iter().collect(), None)
    }

    fn check_level(&self, _check: impl FnOnce(LevelNo) -> bool) -> bool {
        true
    }

    fn children(&self) -> Self::ChildrenIter<'_> {
        BorrowedEdgeIter::from(self.1.iter())
    }

    fn child(&self, _n: usize) -> Borrowed<DummyEdge> {
        unimplemented!()
    }

    unsafe fn set_child(&self, _n: usize, _child: DummyEdge) -> DummyEdge {
        unimplemented!()
    }

    fn ref_count(&self) -> usize {
        unimplemented!()
    }
}

/// Assert that the reference counts of edges match
///
/// # Example
///
/// ```
/// # use oxidd_core::{Edge, Manager};
/// # use oxidd_test_utils::assert_ref_counts;
/// # use oxidd_test_utils::edge::{DummyEdge, DummyManager};
/// let e1 = DummyEdge::new();
/// let e2 = DummyManager.clone_edge(&e1);
/// let e3 = DummyEdge::new();
/// assert_ref_counts!(e1, e2 = 2; e3 = 1);
/// # DummyManager.drop_edge(e1);
/// # DummyManager.drop_edge(e2);
/// # DummyManager.drop_edge(e3);
/// ```
#[macro_export]
macro_rules! assert_ref_counts {
    ($edge:ident = $count:literal) => {
        assert_eq!($edge.ref_count(), $count);
    };
    ($edge:ident, $($edges:ident),+ = $count:literal) => {
        assert_ref_counts!($edge = $count);
        assert_ref_counts!($($edges),+ = $count);
    };
    // spell-checker:ignore edgess
    ($($edges:ident),+ = $count:literal; $($($edgess:ident),+ = $counts:literal);+) => {
        assert_ref_counts!($($edges),+ = $count);
        assert_ref_counts!($($($edgess),+ = $counts);+);
    };
}
