use std::{
    collections::HashSet,
    io::{Cursor, Result},
    marker::PhantomData,
    vec::IntoIter,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use oxidd::LevelNo;

use crate::{
    types::util::{
        graph_structure::{
            graph_structure::{DrawTag, GraphStructure},
            grouped_graph_structure::{EdgeCountData, GroupedGraphStructure},
        },
        storage::state_storage::StateStorage,
    },
    util::rc_refcell::MutRcRefCell,
    wasm_interface::{NodeGroupID, NodeID},
};

pub struct GroupPresenceAdjuster<G: GroupedGraphStructure> {
    graph: MutRcRefCell<G>,
    hidden_groups: HashSet<NodeGroupID>,
}

impl<G: GroupedGraphStructure> GroupPresenceAdjuster<G> {
    pub fn new(graph: G) -> GroupPresenceAdjuster<G> {
        GroupPresenceAdjuster::new_shared(MutRcRefCell::new(graph))
    }
    pub fn new_shared(graph: MutRcRefCell<G>) -> GroupPresenceAdjuster<G> {
        GroupPresenceAdjuster {
            graph,
            hidden_groups: HashSet::new(),
        }
    }

    pub fn show(&mut self, group: NodeGroupID) {
        self.hidden_groups.remove(&group);
    }

    pub fn hide(&mut self, group: NodeGroupID) {
        self.hidden_groups.insert(group);
    }
}

impl<G: GroupedGraphStructure> GroupedGraphStructure for GroupPresenceAdjuster<G> {
    type T = G::T;
    type GL = G::GL;
    type LL = G::LL;
    type Tracker = G::Tracker;

    fn get_roots(&self) -> Vec<NodeGroupID> {
        self.graph
            .read()
            .get_roots()
            .iter()
            .filter(|node| !self.hidden_groups.contains(node))
            .cloned()
            .collect()
    }

    fn get_all_groups(&self) -> Vec<NodeGroupID> {
        self.graph
            .read()
            .get_all_groups()
            .iter()
            .filter(|node| !self.hidden_groups.contains(node))
            .cloned()
            .collect()
    }

    fn get_hidden(&self) -> Vec<NodeGroupID> {
        self.graph
            .read()
            .get_hidden()
            .iter()
            .chain(self.hidden_groups.iter())
            .cloned()
            .collect()
    }

    fn get_group(&self, node: NodeID) -> NodeGroupID {
        self.graph.read().get_group(node)
    }

    fn get_group_label(&self, group: NodeID) -> G::GL {
        self.graph.read().get_group_label(group)
    }

    fn get_parents(&self, group: NodeGroupID) -> Vec<EdgeCountData<G::T>> {
        self.graph
            .read()
            .get_parents(group)
            .into_iter()
            .filter(|ed| !self.hidden_groups.contains(&ed.to))
            .collect()
    }

    fn get_children(&self, group: NodeGroupID) -> Vec<EdgeCountData<G::T>> {
        self.graph
            .read()
            .get_children(group)
            .into_iter()
            .filter(|ed| !self.hidden_groups.contains(&ed.to))
            .collect()
    }

    fn get_nodes_of_group(&self, group: NodeGroupID) -> Vec<NodeID> {
        self.graph.read().get_nodes_of_group(group)
    }

    fn get_level_range(&self, group: NodeGroupID) -> (LevelNo, LevelNo) {
        self.graph.read().get_level_range(group)
    }

    fn get_level_label(&self, level: LevelNo) -> G::LL {
        self.graph.read().get_level_label(level)
    }

    fn refresh(&mut self) {
        self.graph.get().refresh();
    }

    fn create_node_tracker(&mut self) -> Self::Tracker {
        self.graph.get().create_node_tracker()
    }
}

impl<G: GroupedGraphStructure + StateStorage> StateStorage for GroupPresenceAdjuster<G> {
    fn write(&self, stream: &mut Cursor<&mut Vec<u8>>) -> Result<()> {
        let hidden_count = self.hidden_groups.len();
        stream.write_u32::<LittleEndian>(hidden_count as u32)?;
        for &group_id in &self.hidden_groups {
            stream.write_u32::<LittleEndian>(group_id as u32)?;
        }
        self.graph.read().write(stream)?;
        Ok(())
    }

    fn read(&mut self, stream: &mut Cursor<&Vec<u8>>) -> Result<()> {
        self.hidden_groups.clear();
        let group_count = stream.read_u32::<LittleEndian>()?;
        for _ in 0..group_count {
            let group_id = stream.read_u32::<LittleEndian>()? as usize;
            self.hidden_groups.insert(group_id);
        }
        self.graph.get().read(stream)?;
        Ok(())
    }
}
