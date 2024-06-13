use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::{
    util::{free_id_manager::FreeIdManager, rc_refcell::MutRcRefCell},
    wasm_interface::NodeGroupID,
};

pub struct SourceTrackerManager {
    shared: MutRcRefCell<SharedData>,
}
impl SourceTrackerManager {
    pub fn new() -> Self {
        SourceTrackerManager {
            shared: MutRcRefCell::new(SharedData {
                readers: HashMap::new(),
                free_ids: FreeIdManager::new(0),
            }),
        }
    }

    pub fn add_source(&mut self, group: NodeGroupID, source: NodeGroupID) {
        for reader_nodes in self.shared.get().readers.values_mut() {
            // Make sure the source always refers to something that's no longer contained in the map
            let source = if let Some(new_source) = reader_nodes.0.get(&source) {
                *new_source
            } else {
                source
            };

            reader_nodes.0.insert(group, source);
            let count = reader_nodes.1.entry(source).or_insert(0);
            *count += 1;
        }
    }

    pub fn is_tracked_source(&self, group: NodeGroupID) -> bool {
        self.shared
            .read()
            .readers
            .values()
            .any(|reader| reader.1.contains_key(&group))
    }

    pub fn get_reader(&mut self) -> SourceReader {
        let mut shared = self.shared.get();
        let id = shared.free_ids.get_next();
        shared.readers.insert(id, (HashMap::new(), HashMap::new()));
        SourceReader {
            shared: self.shared.clone(),
            id,
        }
    }
}

struct SharedData {
    readers: HashMap<
        usize,
        (
            HashMap<NodeGroupID, NodeGroupID>,
            // The number of nodes for which this is a source, strictly larger than 0
            HashMap<NodeGroupID, usize>,
        ),
    >,
    free_ids: FreeIdManager<usize>,
}

pub struct SourceReader {
    shared: MutRcRefCell<SharedData>,
    id: usize,
}
impl super::grouped_graph_structure::SourceReader for SourceReader {
    fn get_source(&self, group: NodeGroupID) -> NodeGroupID {
        let shared = self.shared.read();
        let nodes = shared.readers.get(&self.id).unwrap();
        if let Some(source) = nodes.0.get(&group) {
            *source
        } else {
            group
        }
    }

    fn get_sourced_nodes(&self) -> HashSet<NodeGroupID> {
        let shared = self.shared.read();
        let nodes = shared.readers.get(&self.id).unwrap();
        nodes.0.keys().cloned().collect()
    }
}
impl super::grouped_graph_structure::SourceTracker for SourceReader {
    fn delete_source(&mut self, group: NodeGroupID) -> () {
        let mut shared = self.shared.get();
        let nodes = shared.readers.get_mut(&self.id).unwrap();

        if let Some(&source) = nodes.0.get(&group) {
            nodes.0.remove(&group);
            let source_count = nodes.1.get_mut(&source).unwrap();
            *source_count -= 1;
            if *source_count <= 0 {
                nodes.1.remove(&source);
            }
        }
    }
}
impl Drop for SourceReader {
    fn drop(&mut self) {
        let mut shared = self.shared.get();
        shared.readers.remove(&self.id);
        shared.free_ids.make_available(self.id);
    }
}
