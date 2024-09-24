use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
    iter::{self, FromIterator},
    rc::Rc,
    usize,
};

use js_sys::Math::random;
use oxidd::{Edge, Function, InnerNode, LevelNo, Manager};
use oxidd_core::{DiagramRules, HasLevel, Node, Tag};

use crate::{
    util::{logging::console, rc_refcell::MutRcRefCell},
    wasm_interface::NodeID,
};

/// A graph structure trait used as the data to visualize
pub trait GraphStructure<T: DrawTag, NL: Clone, LL: Clone> {
    fn get_roots(&self) -> Vec<NodeID>;
    fn get_terminals(&self) -> Vec<NodeID>;
    /// Only returns connections that have already been discovered by calling get_children
    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)>;
    /// This is only supported for nodeIDs that have been obtained from this interface before
    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)>;
    fn get_level(&mut self, node: NodeID) -> LevelNo;

    // Labels for displaying information about nodes
    fn get_node_label(&self, node: NodeID) -> NL;
    fn get_level_label(&self, level: LevelNo) -> LL;

    /// Change events are created for all nodes that have been obtained from this interface before, but might not be invoked for not yet "discovered" nodes
    fn create_event_reader(&mut self) -> GraphEventsReader;
    fn consume_events(&mut self, reader: &GraphEventsReader) -> Vec<Change>;

    /// Retrieves the sources (nodes of the source diagram) of the modified diagram
    fn local_nodes_to_sources(&self, nodes: Vec<NodeID>) -> Vec<NodeID>;
    /// Retrieves the local nodes representing the collection of sources
    fn source_nodes_to_local(&self, nodes: Vec<NodeID>) -> Vec<NodeID>;
}

// pub type GraphListener = dyn Fn(&Vec<Change>) -> ();

pub trait DrawTag: Tag + Hash + Ord {}
impl DrawTag for () {}

#[derive(Eq, PartialEq, Copy, Clone, PartialOrd, Ord, Hash)]
pub struct EdgeType<T: DrawTag> {
    pub tag: T,
    pub index: i32,
}
impl<T: DrawTag> EdgeType<T> {
    pub fn new(tag: T, index: i32) -> EdgeType<T> {
        EdgeType { tag, index }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Change {
    NodeLabelChange {
        node: NodeID,
    },
    LevelChange {
        node: NodeID,
    },
    LevelLabelChange {
        level: LevelNo,
    },
    NodeConnectionsChange {
        node: NodeID,
    },
    NodeRemoval {
        node: NodeID,
    },
    NodeInsertion {
        node: NodeID,
        source: Option<NodeID>,
    },
    ParentDiscover {
        // When a new edge to a parent is discovered
        child: NodeID,
    },
}
impl Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Change::NodeLabelChange { node } => write!(f, "NodeLabelChange {{node: {}}}", node),
            Change::LevelChange { node } => write!(f, "LevelChange {{node: {}}}", node),
            Change::LevelLabelChange { level } => {
                write!(f, "LevelLabelChange {{level: {}}}", level)
            }
            Change::NodeConnectionsChange { node } => {
                write!(f, "NodeConnectionsChange {{node: {}}}", node)
            }
            Change::NodeRemoval { node } => write!(f, "NodeRemoval {{node: {}}}", node),
            Change::NodeInsertion { node, source } => {
                write!(
                    f,
                    "NodeInsertion {{node: {}, source: {}}}",
                    node,
                    source.map(|v| v.to_string()).unwrap_or("null".to_string())
                )
            }
            Change::ParentDiscover { child } => write!(f, "ParentDiscover {{child: {}}}", child),
        }
    }
}

pub struct GraphEventsWriter {
    readers: Vec<MutRcRefCell<GraphEventsReaderInner>>,
}

pub struct GraphEventsReader {
    inner: MutRcRefCell<GraphEventsReaderInner>,
}
struct GraphEventsReaderInner {
    alive: bool,
    events: Vec<Change>,
}

impl GraphEventsWriter {
    pub fn new() -> GraphEventsWriter {
        GraphEventsWriter {
            readers: Vec::new(),
        }
    }

    pub fn create_reader(&mut self) -> GraphEventsReader {
        let inner = MutRcRefCell::new(GraphEventsReaderInner {
            alive: true,
            events: Vec::new(),
        });
        self.readers.push(inner.clone());
        GraphEventsReader { inner }
    }

    fn remove_dead(&mut self) {
        self.readers.retain(|r| r.read().alive);
    }
    pub fn write_vec(&mut self, events: Vec<Change>) {
        self.remove_dead();
        for reader in &self.readers {
            reader.get().events.extend(events.clone())
        }
    }
    pub fn write(&mut self, event: Change) {
        self.remove_dead();
        for reader in &self.readers {
            reader.get().events.push(event.clone());
        }
    }
    pub fn read(&mut self, reader: &GraphEventsReader) -> Vec<Change> {
        let mut inner = reader.inner.get();
        let events = inner.events.clone();
        inner.events.clear();
        events
    }
}

impl Drop for GraphEventsReader {
    fn drop(&mut self) {
        let mut inner = self.inner.get();
        inner.alive = false;
        inner.events.clear()
    }
}
