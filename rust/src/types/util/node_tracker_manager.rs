use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use multimap::MultiMap;

use crate::{
    util::{free_id_manager::FreeIdManager, logging::console, rc_refcell::MutRcRefCell},
    wasm_interface::NodeGroupID,
};

use super::graph_structure::grouped_graph_structure::NodeTracker;

pub struct NodeTrackerManager {
    shared: MutRcRefCell<SharedData>,
}
impl NodeTrackerManager {
    pub fn new(first_id: usize) -> Self {
        NodeTrackerManager {
            shared: MutRcRefCell::new(SharedData {
                first_id,
                existing: HashSet::new(),
                node_remaining_readers_count: HashMap::new(),
                readers: HashMap::new(),
                free_tracker_ids: FreeIdManager::new(0),
                free_group_ids: FreeIdManager::new(first_id),
            }),
        }
    }

    pub fn add_source(&mut self, group: NodeGroupID, source: NodeGroupID) {
        let mut shared = self.shared.get();
        for reader in shared.readers.values_mut() {
            // Remove the current source
            if let Some(&cur_source) = reader.sources.get(&group) {
                reader
                    .images
                    .get_vec_mut(&cur_source)
                    .map(|images| images.retain(|&image| image != group));
            }

            let source = if let Some(&source_source) = reader.sources.get(&source) {
                source_source
            } else {
                source
            };

            reader.sources.insert(group, source);
            reader.images.insert(source, group);
        }
    }

    pub fn add_group_id(&mut self, id: NodeGroupID) {
        let mut shared = self.shared.get();
        shared.existing.insert(id);
        for reader in shared.readers.values_mut() {
            reader.nodes.insert(id);
        }
        let ref_count = shared.readers.len() + 1;
        shared.node_remaining_readers_count.insert(id, ref_count);
    }
    pub fn get_free_group_id_and_add(&mut self) -> NodeGroupID {
        let mut shared = self.shared.get();
        let id = shared.free_group_ids.get_next();
        drop(shared);
        self.add_group_id(id);
        id
    }

    pub fn make_available(&mut self, group_id: NodeGroupID) {
        let mut shared = self.shared.get();
        let contained = shared.existing.remove(&group_id);
        if contained {
            shared.sub_ref_count(group_id);
        }
    }

    pub fn create_reader(&mut self) -> NodeTrackerM {
        let mut shared = self.shared.get();
        let id = shared.free_tracker_ids.get_next();
        let nodes = shared.existing.clone();
        for node in nodes.iter() {
            let count = shared.node_remaining_readers_count.get_mut(&node).unwrap();
            *count += 1;
        }
        let reader = ReaderData {
            nodes,
            sources: HashMap::new(),
            images: MultiMap::new(),
        };
        shared.readers.insert(id, reader);
        NodeTrackerM {
            shared: self.shared.clone(),
            id,
        }
    }
}

struct SharedData {
    existing: HashSet<NodeGroupID>, // The nodes that are still in the actual graph currently
    node_remaining_readers_count: HashMap<NodeGroupID, usize>, // Per node, the number of readers that still include it
    readers: HashMap<usize, ReaderData>, // Per reader, the nodes it is still interested in
    free_tracker_ids: FreeIdManager<usize>,
    free_group_ids: FreeIdManager<usize>,
    first_id: usize,
}

struct ReaderData {
    nodes: HashSet<NodeGroupID>,
    sources: HashMap<NodeGroupID, NodeGroupID>, // Per node, possibly the source of said node
    images: MultiMap<NodeGroupID, NodeGroupID>, // Per node, possibly the images (nodes for which this is the source) of that node
}

impl SharedData {
    fn remove_group(&mut self, group_id: NodeGroupID) {
        for reader in self.readers.values_mut() {
            // Remove source data
            let source = reader.sources.remove(&group_id);
            if let Some(source) = source {
                let source_images = reader.images.get_vec_mut(&source);
                if let Some(source_images) = source_images {
                    source_images.retain(|&image| image != group_id);
                }
            }

            // Remove image data
            let images = reader.images.remove(&group_id);
            if let Some(images) = images {
                for image in images {
                    reader.sources.remove(&image);
                }
            }
        }

        // Free the id
        if group_id >= self.first_id {
            self.free_group_ids.make_available(group_id);
        }
    }

    fn sub_ref_count(&mut self, group_id: NodeGroupID) {
        let count = self
            .node_remaining_readers_count
            .get_mut(&group_id)
            .unwrap();
        *count -= 1;
        if *count <= 0 {
            self.node_remaining_readers_count.remove(&group_id);
            self.remove_group(group_id);
        }
    }
}

pub struct NodeTrackerM {
    shared: MutRcRefCell<SharedData>,
    id: usize,
}
impl super::graph_structure::grouped_graph_structure::SourceReader for NodeTrackerM {
    fn get_source(&self, group: NodeGroupID) -> Option<NodeGroupID> {
        let shared = self.shared.read();
        let reader = shared.readers.get(&self.id).unwrap();
        if !reader.nodes.contains(&group) {
            return None;
        }

        if let Some(source) = reader.sources.get(&group) {
            if reader.nodes.contains(source) {
                return Some(*source);
            }
        }
        None
    }

    fn remove_sources(&mut self) {
        let mut shared = self.shared.get();
        let reader = shared.readers.get_mut(&self.id).unwrap();
        reader.images.clear();
        reader.sources.clear();
    }
}
impl super::graph_structure::grouped_graph_structure::NodeTracker for NodeTrackerM {
    fn retain<F: Fn(NodeGroupID) -> bool>(&mut self, filter: F) -> () {
        let mut shared = self.shared.get();
        let reader = shared.readers.get_mut(&self.id).unwrap();

        let mut removed = HashSet::new();
        for group in reader.nodes.iter().cloned().collect_vec() {
            let keep = filter(group);
            if keep {
                continue;
            }

            reader.nodes.remove(&group);
            removed.insert(group);
        }

        for group in removed {
            shared.sub_ref_count(group);
        }
    }
}
impl Drop for NodeTrackerM {
    fn drop(&mut self) {
        self.retain(|_| false);
        let mut shared = self.shared.get();
        shared.readers.remove(&self.id);
        shared.free_tracker_ids.make_available(self.id);
    }
}
