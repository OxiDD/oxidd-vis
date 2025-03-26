use itertools::{EitherOrBoth, Itertools};
use oxidd::util::OutOfMemory;
use oxidd::{util::Borrowed, Edge, InnerNode, Manager, ManagerRef};
use oxidd::{BooleanFunction, Function};
use oxidd_manager_index::node::fixed_arity::NodeWithLevel;
use oxidd_rules_bdd::simple::BDDTerminal;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::convert::TryInto;
use std::fmt::Display;
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

#[derive(Clone, Copy, PartialOrd)]
pub struct MTBDDTerminal(pub f32);
impl Display for MTBDDTerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl Hash for MTBDDTerminal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}
impl PartialEq for MTBDDTerminal {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}
impl Ord for MTBDDTerminal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}
impl Eq for MTBDDTerminal {}

// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Clone, PartialEq, Eq)]
pub struct DummyMTBDDManagerRef(Rc<RefCell<DummyMTBDDManager>>);

impl Hash for DummyMTBDDManagerRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.borrow().hash(state);
    }
}
impl<'a> From<&'a DummyMTBDDManager> for DummyMTBDDManagerRef {
    fn from(value: &'a DummyMTBDDManager) -> Self {
        DummyMTBDDManagerRef(Rc::new(RefCell::new(value.clone())))
    }
}
impl ManagerRef for DummyMTBDDManagerRef {
    type Manager<'id> = DummyMTBDDManager;

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
pub struct DummyMTBDDFunction(pub DummyMTBDDEdge);
impl DummyMTBDDFunction {
    pub fn from_dddmp(
        manager_ref: &mut DummyMTBDDManagerRef,
        data: &str,
    ) -> (Vec<(DummyMTBDDFunction, Vec<String>)>, Vec<String>) {
        manager_ref.with_manager_exclusive(|manager| {
            let mut terminals = HashMap::new();

            let get_text = |from: &str, to: &str| {
                let start = data.find(from).unwrap() + from.len();
                Box::new(&data[start + 1..start + data[start..].find(to).unwrap()])
            };

            let roots_text = get_text(".rootids", "\n");
            let roots = roots_text
                .trim()
                .split(" ")
                .flat_map(|n| n.parse::<usize>())
                .collect_vec();
            let root_names = if data.find(".rootnames").is_some() {
                let roots_names_text = get_text(".rootnames", "\n");
                roots_names_text
                    .trim()
                    .split(" ")
                    .map(|t| t.to_string())
                    .collect_vec()
            } else {
                roots
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("f{i}"))
                    .collect_vec()
            };

            let node_text = get_text(".nodes", ".end");
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
            let mut max_level = 0;
            for (_, level, _) in nodes_data.clone() {
                let Ok(level) = level.parse() else { continue };

                if level > max_level {
                    max_level = level;
                }
            }

            for (id, level, children) in nodes_data.clone() {
                let level_num = level.parse();
                let term_num = (level.parse() as Result<f32, _>).map(|r| MTBDDTerminal(r));
                let is_terminal = children[0] == 0;
                manager.add_node_level(
                    id.clone(),
                    if is_terminal {
                        max_level + 1 // Terminal nodes don't define a level, we have to assign it
                    } else {
                        level_num.clone().unwrap()
                    },
                    if is_terminal {
                        term_num.clone().ok()
                    } else {
                        None
                    },
                );

                if is_terminal {
                    terminals.insert(
                        term_num.unwrap(),
                        DummyMTBDDEdge::new(Arc::new(id), manager_ref.clone()),
                    );
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

            let mut func_map = HashMap::<NodeID, (DummyMTBDDFunction, Vec<String>)>::new();
            for (root, name) in roots.into_iter().zip(root_names.into_iter()) {
                func_map
                    .entry(root)
                    .or_insert_with(|| {
                        (
                            DummyMTBDDFunction(DummyMTBDDEdge::new(
                                Arc::new(root),
                                manager_ref.clone(),
                            )),
                            vec![],
                        )
                    })
                    .1
                    .push(name.to_string());
            }
            let funcs = func_map.values().cloned().collect_vec();

            let var_names = if data.find(".suppvarnames").is_some() {
                let var_names_text = get_text(".suppvarnames", ".orderedvarnames");
                var_names_text
                    .trim()
                    .split(" ")
                    .map(|t| t.to_string())
                    .collect_vec()
            } else {
                let var_count = get_text(".nsuppvars", ".").trim().parse().unwrap_or(0);
                (0..var_count)
                    .into_iter()
                    .map(|i| format!("{}", i))
                    .collect_vec()
            };

            (funcs, var_names)
        })
    }
}

unsafe impl Function for DummyMTBDDFunction {
    type Manager<'id> = DummyMTBDDManager;

    type ManagerRef = DummyMTBDDManagerRef;
    fn from_edge<'id>(
        manager: &Self::Manager<'id>,
        edge: oxidd_core::function::EdgeOfFunc<'id, Self>,
    ) -> Self {
        DummyMTBDDFunction(edge)
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
pub struct DummyMTBDDEdge(Arc<NodeID>, DummyMTBDDManagerRef);

impl PartialEq for DummyMTBDDEdge {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for DummyMTBDDEdge {}
impl PartialOrd for DummyMTBDDEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for DummyMTBDDEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        Arc::as_ptr(&self.0).cmp(&Arc::as_ptr(&other.0))
    }
}
impl Hash for DummyMTBDDEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl Drop for DummyMTBDDEdge {
    fn drop(&mut self) {
        eprintln!(
            "Edges must not be dropped. Use Manager::drop_edge(). Backtrace:\n{}",
            std::backtrace::Backtrace::capture()
        );
    }
}

impl DummyMTBDDEdge {
    /// Create a new `DummyEdge`
    pub fn new(to: Arc<NodeID>, mr: DummyMTBDDManagerRef) -> Self {
        DummyMTBDDEdge(to, mr.clone())
    }
}

impl Edge for DummyMTBDDEdge {
    type Tag = ();

    fn borrowed(&self) -> Borrowed<'_, Self> {
        let ptr = Arc::as_ptr(&self.0);
        Borrowed::new(DummyMTBDDEdge(
            unsafe { Arc::from_raw(ptr) },
            self.1.clone(),
        ))
    }
    fn with_tag(&self, _tag: ()) -> Borrowed<'_, Self> {
        let ptr = Arc::as_ptr(&self.0);
        Borrowed::new(DummyMTBDDEdge(
            unsafe { Arc::from_raw(ptr) },
            self.1.clone(),
        ))
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
pub struct DummyMTBDDManager(
    BTreeMap<NodeID, DummyMTBDDNode>,
    HashMap<MTBDDTerminal, DummyMTBDDEdge>,
);
impl DummyMTBDDManager {
    pub fn new() -> DummyMTBDDManager {
        DummyMTBDDManager(BTreeMap::new(), HashMap::new())
    }
    fn init_terminals(&mut self, terminals: HashMap<MTBDDTerminal, DummyMTBDDEdge>) {
        self.1.extend(terminals);
    }
}
impl Hash for DummyMTBDDManager {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Dummy diagram rules
pub struct DummyMTBDDRules;
impl DiagramRules<DummyMTBDDEdge, DummyMTBDDNode, MTBDDTerminal> for DummyMTBDDRules {
    // type Cofactors<'a> = Iter<'a, Borrowed<'a, DummyEdge>>;
    type Cofactors<'a>
        = <DummyMTBDDNode as InnerNode<DummyMTBDDEdge>>::ChildrenIter<'a>
    where
        DummyMTBDDNode: 'a,
        DummyMTBDDEdge: 'a;

    fn reduce<M>(
        _manager: &M,
        level: LevelNo,
        children: impl IntoIterator<Item = DummyMTBDDEdge>,
    ) -> ReducedOrNew<DummyMTBDDEdge, DummyMTBDDNode>
    where
        M: Manager<Edge = DummyMTBDDEdge, InnerNode = DummyMTBDDNode>,
    {
        ReducedOrNew::New(DummyMTBDDNode::new(level, children), ())
    }

    fn cofactors(_tag: (), node: &DummyMTBDDNode) -> Self::Cofactors<'_> {
        node.children()
    }
}

impl DummyMTBDDManager {
    fn add_node_level(
        &mut self,
        from: NodeID,
        level: LevelNo,
        terminal: Option<MTBDDTerminal>,
    ) -> &mut DummyMTBDDNode {
        self.0.entry(from).or_insert_with(|| {
            if terminal.is_some() {
                DummyMTBDDNode(level, Vec::new(), terminal)
            } else {
                DummyMTBDDNode::new(level, Vec::new())
            }
        })
    }
    fn add_node(&mut self, from: NodeID) -> &mut DummyMTBDDNode {
        self.add_node_level(from, from.try_into().unwrap(), None)
    }
    fn add_edge(&mut self, from: NodeID, to: NodeID, mr: DummyMTBDDManagerRef) {
        let from_children = &mut self.0.get_mut(&from).unwrap().1;
        let edge = DummyMTBDDEdge::new(Arc::new(to), mr);
        from_children.push(edge);
    }
    fn has_edges(&self, node: NodeID) -> bool {
        let from_children = &self.0.get(&node).unwrap().1;
        from_children.len() > 0
    }
}

unsafe impl Manager for DummyMTBDDManager {
    type Edge = DummyMTBDDEdge;
    type EdgeTag = ();
    type InnerNode = DummyMTBDDNode;
    type Terminal = MTBDDTerminal;
    type TerminalRef<'a> = &'a MTBDDTerminal;
    type TerminalIterator<'a>
        = Cloned<std::collections::hash_map::Values<'a, MTBDDTerminal, DummyMTBDDEdge>>
    where
        Self: 'a;
    type Rules = DummyMTBDDRules;
    type NodeSet = HashSet<NodeID>;
    type LevelView<'a>
        = DummyMTBDDLevelView
    where
        Self: 'a;
    type LevelIterator<'a>
        = std::iter::Empty<DummyMTBDDLevelView>
    where
        Self: 'a;

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
        DummyMTBDDEdge(edge.0.clone(), edge.1.clone())
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
pub struct DummyMTBDDLevelView;

unsafe impl LevelView<DummyMTBDDEdge, DummyMTBDDNode> for DummyMTBDDLevelView {
    type Iterator<'a>
        = std::iter::Empty<&'a DummyMTBDDEdge>
    where
        Self: 'a,
        DummyMTBDDEdge: 'a;

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

    fn get(&self, _node: &DummyMTBDDNode) -> Option<&DummyMTBDDEdge> {
        unreachable!()
    }

    fn insert(&mut self, _edge: DummyMTBDDEdge) -> bool {
        unreachable!()
    }

    fn get_or_insert(&mut self, _node: DummyMTBDDNode) -> AllocResult<DummyMTBDDEdge> {
        unreachable!()
    }

    unsafe fn gc(&mut self) {
        unreachable!()
    }

    unsafe fn remove(&mut self, _node: &DummyMTBDDNode) -> bool {
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
pub struct DummyMTBDDNode(LevelNo, Vec<DummyMTBDDEdge>, Option<MTBDDTerminal>);

impl DropWith<DummyMTBDDEdge> for DummyMTBDDNode {
    fn drop_with(self, _drop_edge: impl Fn(DummyMTBDDEdge)) {
        unimplemented!()
    }
}

unsafe impl HasLevel for DummyMTBDDNode {
    fn level(&self) -> LevelNo {
        self.0
    }

    unsafe fn set_level(&self, _level: LevelNo) {
        unimplemented!()
    }
}

impl InnerNode<DummyMTBDDEdge> for DummyMTBDDNode {
    const ARITY: usize = 0;

    // type ChildrenIter<'a> = std::iter::Empty<Borrowed<'a, DummyEdge>>
    // where
    //     Self: 'a;
    type ChildrenIter<'a>
        = BorrowedEdgeIter<'a, DummyMTBDDEdge, Iter<'a, DummyMTBDDEdge>>
    where
        Self: 'a;

    fn new(level: LevelNo, children: impl IntoIterator<Item = DummyMTBDDEdge>) -> Self {
        DummyMTBDDNode(level, children.into_iter().collect(), None)
    }

    fn check_level(&self, _check: impl FnOnce(LevelNo) -> bool) -> bool {
        true
    }

    fn children(&self) -> Self::ChildrenIter<'_> {
        BorrowedEdgeIter::from(self.1.iter())
    }

    fn child(&self, _n: usize) -> Borrowed<DummyMTBDDEdge> {
        unimplemented!()
    }

    unsafe fn set_child(&self, _n: usize, _child: DummyMTBDDEdge) -> DummyMTBDDEdge {
        unimplemented!()
    }

    fn ref_count(&self) -> usize {
        unimplemented!()
    }
}
