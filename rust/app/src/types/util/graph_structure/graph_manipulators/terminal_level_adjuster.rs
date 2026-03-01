use std::{
    collections::{HashMap, HashSet},
    default,
    iter::FromIterator,
    marker::PhantomData,
};

use itertools::Itertools;
use oxidd::LevelNo;

use crate::{
    types::util::{
        graph_structure::graph_structure::{
            Change, DrawTag, EdgeType, GraphEventsReader, GraphEventsWriter, GraphStructure,
        },
        storage::state_storage::StateStorage,
    },
    util::{logging::console, rc_refcell::MutRcRefCell},
    wasm_interface::NodeID,
};

pub struct TerminalLevelAdjuster<G: GraphStructure + 'static> {
    graph: G,
    event_writer: GraphEventsWriter,
    graph_events: GraphEventsReader,

    level_cache: HashMap<NodeID, LevelNo>,
    terminal_parents_cache: HashMap<NodeID, HashSet<NodeID>>,
}

impl<G: GraphStructure> TerminalLevelAdjuster<G> {
    pub fn new(mut graph: G) -> TerminalLevelAdjuster<G> {
        let mut ta = TerminalLevelAdjuster {
            level_cache: HashMap::new(),
            graph_events: graph.create_event_reader(),
            event_writer: GraphEventsWriter::new(),
            terminal_parents_cache: HashMap::new(),
            graph,
        };
        ta.init_terminals_cache();
        ta
    }

    fn init_terminals_cache(&mut self) {
        self.terminal_parents_cache.clear();
        for terminal in self.graph.get_terminals() {
            self.terminal_parents_cache.insert(
                terminal,
                self.graph
                    .get_known_parents(terminal)
                    .iter()
                    .map(|&(_, p)| p)
                    .collect(),
            );
        }
    }

    fn process_graph_changes(&mut self) {
        let events = self.graph.consume_events(&self.graph_events);

        let mut level_change_events = Vec::<Change>::new();
        let mut maybe_terminals: Option<Vec<NodeID>> = None;
        for event in events.clone() {
            match event {
                Change::LevelChange { node } => {
                    self.level_cache.remove(&node);
                }
                Change::NodeRemoval { node } => {
                    self.level_cache.remove(&node);
                    self.terminal_parents_cache.remove(&node);
                }
                Change::NodeInsertion { node, source: _ } => {
                    let terminals =
                        maybe_terminals.get_or_insert_with(|| self.graph.get_terminals());
                    let is_terminal = terminals.contains(&node);
                    if is_terminal {
                        let parents = self
                            .graph
                            .get_known_parents(node)
                            .iter()
                            .map(|&(_, p)| p)
                            .collect();
                        self.terminal_parents_cache.insert(node, parents);
                        level_change_events.push(Change::LevelChange { node });
                    }
                }
                _ => {}
            }
        }

        self.event_writer.write_vec(
            level_change_events
                .into_iter()
                .chain(events.into_iter())
                .collect(),
        );
    }
}

impl<G: GraphStructure> StateStorage for TerminalLevelAdjuster<G>
where
    G: StateStorage,
{
    fn read(&mut self, stream: &mut std::io::Cursor<&Vec<u8>>) -> std::io::Result<()> {
        self.graph.read(stream)?;
        self.init_terminals_cache();
        Ok(())
    }
    fn write(&self, stream: &mut std::io::Cursor<&mut Vec<u8>>) -> std::io::Result<()> {
        self.graph.write(stream)
    }
}

impl<G: GraphStructure> GraphStructure for TerminalLevelAdjuster<G> {
    type T = G::T;
    type NL = G::NL;
    type LL = G::LL;

    fn get_roots(&self) -> Vec<NodeID> {
        self.graph.get_roots()
    }

    fn get_terminals(&self) -> Vec<NodeID> {
        self.graph.get_terminals()
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<Self::T>, NodeID)> {
        self.graph.get_known_parents(node)
    }

    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<Self::T>, NodeID)> {
        let children = self.graph.get_children(node);

        // When new children are found, it might be a connection to a terminal which might cause the level of the terminal to need to be updated, so we do this here
        let terminal_children = children.iter().filter_map(|(_, child)| {
            self.terminal_parents_cache
                .get(child)
                .map(|parents| (*child, parents))
        });
        let new_parent_terminals = terminal_children
            .filter_map(|(terminal, parents)| {
                if !parents.contains(&node) {
                    Some(terminal)
                } else {
                    None
                }
            })
            .collect_vec();
        // drop(inner);
        for terminal in new_parent_terminals {
            let parents = self.terminal_parents_cache.get_mut(&terminal).unwrap();
            parents.insert(node);

            // known_parents.insert(node);
            let maybe_old_level = self.level_cache.get(&terminal).cloned();
            if let Some(old_level) = maybe_old_level {
                self.level_cache.remove(&terminal);
                // drop(inner);
                // let new_level = self.get_level(node);
                // if new_level != old_level {
                //     // TODO: call level change listeners
                // }
                self.event_writer
                    .write(Change::LevelChange { node: terminal });
            }
        }

        children
    }

    fn get_level(&mut self, node: NodeID) -> LevelNo {
        self.process_graph_changes();
        if self.terminal_parents_cache.contains_key(&node) {
            if let Some(level) = self.level_cache.get(&node) {
                *level
            } else {
                let level = self
                    .graph
                    .get_known_parents(node)
                    .iter()
                    .fold(0, |max, &(_, parent)| max.max(self.graph.get_level(parent)))
                    + 1;
                self.level_cache.insert(node, level);
                level
            }
        } else {
            self.graph.get_level(node)
        }
    }

    fn get_node_label(&self, node: NodeID) -> Self::NL {
        self.graph.get_node_label(node)
    }

    fn get_level_label(&self, level: LevelNo) -> Self::LL {
        self.graph.get_level_label(level)
    }

    fn create_event_reader(&mut self) -> GraphEventsReader {
        self.event_writer.create_reader()
    }
    fn consume_events(&mut self, reader: &GraphEventsReader) -> Vec<Change> {
        self.process_graph_changes();
        self.event_writer.read(reader)
    }

    fn local_nodes_to_sources(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.local_nodes_to_sources(nodes)
    }

    fn source_nodes_to_local(&self, nodes: Vec<NodeID>) -> Vec<NodeID> {
        self.graph.source_nodes_to_local(nodes)
    }
}
