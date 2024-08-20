use std::{
    collections::{HashMap, HashSet},
    default,
    iter::FromIterator,
    marker::PhantomData,
};

use itertools::Itertools;
use oxidd::LevelNo;

use crate::{
    types::util::graph_structure::graph_structure::{
        Change, DrawTag, EdgeType, GraphListener, GraphStructure,
    },
    util::{logging::console, rc_refcell::MutRcRefCell},
    wasm_interface::NodeID,
};

use super::util::graph_listener_manager::GraphListenerManager;

pub struct TerminalLevelAdjuster<
    T: DrawTag + 'static,
    NL: Clone + 'static,
    LL: Clone + 'static,
    G: GraphStructure<T, NL, LL> + 'static,
> {
    inner: MutRcRefCell<TerminalLevelAdjusterInner<T, NL, LL, G>>,
    listener_handle: Option<usize>,
}

struct TerminalLevelAdjusterInner<
    T: DrawTag + 'static,
    NL: Clone,
    LL: Clone,
    G: GraphStructure<T, NL, LL>,
> {
    graph: G,
    listeners: GraphListenerManager,
    level_cache: HashMap<NodeID, LevelNo>,
    terminal_parents_cache: HashMap<NodeID, HashSet<NodeID>>,
    node_label: PhantomData<NL>,
    level_label: PhantomData<LL>,
    tag: PhantomData<T>,
}

impl<T: DrawTag + 'static, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>>
    TerminalLevelAdjuster<T, NL, LL, G>
{
    pub fn new(mut graph: G) -> TerminalLevelAdjuster<T, NL, LL, G> {
        let mut adjuster = TerminalLevelAdjuster {
            inner: MutRcRefCell::new(TerminalLevelAdjusterInner {
                level_cache: HashMap::new(),
                terminal_parents_cache: HashMap::from_iter(
                    graph
                        .get_terminals()
                        .iter()
                        .map(|&t| {
                            (
                                t,
                                graph.get_known_parents(t).iter().map(|&(_, p)| p).collect(),
                            )
                        })
                        .collect::<HashMap<NodeID, HashSet<NodeID>>>(),
                ),
                listeners: GraphListenerManager::new(),
                node_label: PhantomData,
                level_label: PhantomData,
                tag: PhantomData,
                graph,
            }),
            listener_handle: None,
        };
        adjuster.setup_listener_forwarding();
        adjuster
    }

    fn setup_listener_forwarding(&mut self) {
        let inner = self.inner.clone();
        self.listener_handle = Some(self.inner.get().graph.on_change(Box::new(move |events| {
            let mut maybe_terminals: Option<Vec<NodeID>> = None;
            for event in events {
                match event {
                    Change::LevelChange { node } => {
                        inner.get().level_cache.remove(&node);
                    }
                    Change::NodeRemoval { node } => {
                        let mut inner = inner.get();
                        inner.level_cache.remove(&node);
                        inner.terminal_parents_cache.remove(&node);
                    }
                    Change::NodeInsertion { node, source: _ } => {
                        console::log!("before");
                        let mut inner = inner.get();
                        console::log!("after1");
                        let terminals =
                            maybe_terminals.get_or_insert_with(|| inner.graph.get_terminals());
                        console::log!("after2");
                        let is_terminal = terminals.contains(node);
                        console::log!("after3");
                        if is_terminal {
                            let parents = inner
                                .graph
                                .get_known_parents(*node)
                                .iter()
                                .map(|&(_, p)| p)
                                .collect();
                            console::log!("after4");
                            inner.terminal_parents_cache.insert(*node, parents);
                        }
                        console::log!("after");
                    }
                    _ => {}
                }
            }

            inner.get().listeners.dispatch_changes(events);
        })));
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> Drop
    for TerminalLevelAdjuster<T, NL, LL, G>
{
    fn drop(&mut self) {
        if let Some(listener_handle) = self.listener_handle {
            self.inner.get().graph.off_change(listener_handle);
        }
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> GraphStructure<T, NL, LL>
    for TerminalLevelAdjuster<T, NL, LL, G>
{
    fn get_root(&self) -> NodeID {
        self.inner.read().graph.get_root()
    }

    fn get_terminals(&self) -> Vec<NodeID> {
        self.inner.read().graph.get_terminals()
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)> {
        self.inner.get().graph.get_known_parents(node)
    }

    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)> {
        let mut inner = self.inner.get();
        let children = inner.graph.get_children(node);

        // When new children are found, it might be a connection to a terminal which might cause the level of the terminal to need to be updated, so we do this here
        let terminal_children = children.iter().filter_map(|(_, child)| {
            inner
                .terminal_parents_cache
                .get(child)
                .map(|parents| (*child, parents))
        });
        let new_parent_terminals = terminal_children
            .filter_map(|(_, parents)| {
                if !parents.contains(&node) {
                    Some(node)
                } else {
                    None
                }
            })
            .collect_vec();
        // drop(inner);
        for terminal in new_parent_terminals {
            let mut inner = self.inner.get();
            let parents = inner.terminal_parents_cache.get_mut(&terminal).unwrap();
            parents.insert(node);

            // known_parents.insert(node);
            let maybe_old_level = inner.level_cache.get(&terminal).cloned();
            if let Some(old_level) = maybe_old_level {
                inner.level_cache.remove(&terminal);
                // drop(inner);
                // let new_level = self.get_level(node);
                // if new_level != old_level {
                //     // TODO: call level change listeners
                // }
                inner
                    .listeners
                    .add_change(Change::LevelChange { node: terminal });
            }
        }
        inner.listeners.dispatch_change();

        children
    }

    fn get_level(&mut self, node: NodeID) -> LevelNo {
        let mut inner = self.inner.get();
        if inner.terminal_parents_cache.contains_key(&node) {
            if let Some(level) = inner.level_cache.get(&node) {
                *level
            } else {
                let level = inner
                    .graph
                    .get_known_parents(node)
                    .iter()
                    .fold(0, |max, &(_, parent)| {
                        max.max(inner.graph.get_level(parent))
                    })
                    + 1;
                inner.level_cache.insert(node, level);
                level
            }
        } else {
            inner.graph.get_level(node)
        }
    }

    fn get_node_label(&self, node: NodeID) -> NL {
        self.inner.read().graph.get_node_label(node)
    }

    fn get_level_label(&self, level: LevelNo) -> LL {
        self.inner.read().graph.get_level_label(level)
    }

    fn on_change(&mut self, listener: Box<GraphListener>) -> usize {
        self.inner.get().listeners.add(listener)
    }

    fn off_change(&mut self, listener: usize) {
        self.inner.get().listeners.remove(listener)
    }
}
