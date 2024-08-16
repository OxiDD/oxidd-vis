use std::{cell::RefCell, collections::HashMap, rc::Rc};

use itertools::Itertools;

use crate::{
    types::util::graph_structure::graph_structure::{
        Change, DrawTag, GraphListener, GraphStructure,
    },
    util::free_id_manager::FreeIdManager,
};

pub struct GraphListenerManager {
    listeners: HashMap<usize, Box<GraphListener>>,
    free_id: FreeIdManager<usize>,
    changes: Vec<Change>,
}

impl GraphListenerManager {
    pub fn new() -> GraphListenerManager {
        GraphListenerManager {
            listeners: HashMap::new(),
            free_id: FreeIdManager::new(0),
            changes: Vec::new(),
        }
    }

    pub fn add(&mut self, listener: Box<GraphListener>) -> usize {
        let id = self.free_id.get_next();
        self.listeners.insert(id, listener);
        id
    }

    pub fn remove(&mut self, listener: usize) {
        if self.listeners.contains_key(&listener) {
            self.free_id.make_available(listener);
            self.listeners.remove(&listener);
        }
    }

    pub fn add_change(&mut self, change: Change) {
        self.changes.push(change);
    }

    pub fn dispatch_change(&mut self) {
        for listener in self.listeners.values() {
            listener(&self.changes);
        }
        self.changes.clear();
    }

    pub fn dispatch_changes(&mut self, changes: &Vec<Change>) {
        &self.changes.extend(changes.iter().cloned());
        self.dispatch_change();
    }
}
