use std::{
    collections::{HashMap, HashSet},
    ops::DerefMut,
};

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

    pub fn add_sources(&mut self, group: NodeGroupID, sources: Vec<NodeGroupID>) {
        let mut shared = self.shared.get();
        for reader in shared.readers.values_mut() {
            if !reader.nodes.contains(&group) {
                continue;
            }

            let sources = sources
                .iter()
                .flat_map(|source| {
                    reader
                        .sources
                        .get(&source)
                        .map(|sources| sources.iter().collect_vec())
                        .unwrap_or_else(|| vec![source])
                })
                .cloned()
                .collect_vec();

            for source in sources {
                reader
                    .sources
                    .entry(group)
                    .or_insert_with(|| HashSet::new())
                    .deref_mut()
                    .insert(source);
                reader
                    .images
                    .entry(source)
                    .or_insert_with(|| HashSet::new())
                    .deref_mut()
                    .insert(group);
            }
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
            images: HashMap::new(),
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

impl SharedData {
    fn sub_ref_count(&mut self, group_id: NodeGroupID) {
        let count = self
            .node_remaining_readers_count
            .get_mut(&group_id)
            .unwrap();
        *count -= 1;
        if *count <= 0 {
            self.node_remaining_readers_count.remove(&group_id);

            if group_id >= self.first_id {
                self.free_group_ids.make_available(group_id);
            }
        }
    }
}

struct ReaderData {
    nodes: HashSet<NodeGroupID>,
    sources: HashMap<NodeGroupID, HashSet<NodeGroupID>>, // Per node, possibly the sources of said node
    images: HashMap<NodeGroupID, HashSet<NodeGroupID>>, // Per node, possibly the images (nodes for which this is the source) of that node
}

impl ReaderData {
    fn remove_group(&mut self, group_id: NodeGroupID) {
        // Remove the group
        self.nodes.remove(&group_id);

        // Remove source data
        let sources = self.sources.remove(&group_id);
        if let Some(sources) = sources {
            for source in sources {
                let Some(source_images) = self.images.get_mut(&source) else {
                    continue;
                };
                source_images.remove(&group_id);
                if source_images.len() == 0 {
                    self.images.remove(&source);
                }
            }
        }

        // Remove image data
        let images = self.images.remove(&group_id);
        if let Some(images) = images {
            for image in images {
                let Some(image_sources) = self.sources.get_mut(&image) else {
                    continue;
                };
                image_sources.remove(&group_id);
                if image_sources.len() == 0 {
                    self.sources.remove(&image);
                }
            }
        }
    }
}

pub struct NodeTrackerM {
    shared: MutRcRefCell<SharedData>,
    id: usize,
}
impl super::graph_structure::grouped_graph_structure::SourceReader for NodeTrackerM {
    fn get_sources(&self, group: NodeGroupID) -> Vec<NodeGroupID> {
        let shared = self.shared.read();
        let reader = shared.readers.get(&self.id).unwrap();
        if let Some(sources) = reader.sources.get(&group) {
            return sources.iter().cloned().sorted().collect();
        }
        Vec::new()
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

            reader.remove_group(group);
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
