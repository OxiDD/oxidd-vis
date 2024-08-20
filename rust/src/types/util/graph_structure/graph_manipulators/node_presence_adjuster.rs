use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Ref, RefCell},
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
    rc::Rc,
};

use itertools::{Either, Itertools};
use multimap::MultiMap;
use oxidd::{LevelNo, NodeID};

use crate::{
    types::util::graph_structure::graph_structure::{
        Change, DrawTag, EdgeType, GraphListener, GraphStructure,
    },
    util::{free_id_manager::FreeIdManager, logging::console},
};

use super::util::graph_listener_manager::GraphListenerManager;

pub struct NodePresenceAdjuster<
    T: DrawTag + 'static,
    NL: Clone,
    LL: Clone,
    G: GraphStructure<T, NL, LL>,
> {
    graph: G,
    node_data: Rc<RefCell<NodeData<T>>>,
    listener_handle: Option<usize>,
    node_label: PhantomData<NL>,
    level_label: PhantomData<LL>,
}

/// We distinguish 2 different nodeID kinds:
/// - source node IDs, corresponding to the ID of the underlying graph(s)
/// - output node IDs, corresponding to the IDs used to interface with this graph
///
/// The source node IDs are distinguished into 2 labeled kinds:
/// - left node IDs, corresponding to the underlying graph we are wrapping
/// - right node IDs, corresponding to the created virtual nodes

struct NodeData<T: DrawTag + 'static> {
    adjustments: HashMap<NodeID, PresenceGroups<T>>, // Specifies the adjustments for the left source node ID
    sources: HashMap<NodeID, NodeID>, // Maps the right source nodeID to the corresponding left source node ID
    images: MultiMap<NodeID, NodeID>, // Maps the left source nodeID to all of the corresponding right source node IDs
    // node_group: HashMap<NodeID, PresenceGroup>, // Maps the left source nodeID to the presence group it represents
    replacements: HashMap<(NodeID, EdgeConstraint<T>, NodeID), NodeID>, // For a combination of parent output nodeID and a child left source nodeID, the replacement child right source nodeID
    parent_nodes: HashMap<NodeID, HashSet<NodeID>>, // The parent nodes (output node IDs) of a right source nodeID.
    known_parents: HashMap<NodeID, Vec<(EdgeType<T>, NodeID)>>, // The parents (output node IDs) and edge type of a right source nodeID. Note that these are the known parents, because we may for sure these are the only parents that can exist for the created node, but can not be sure these are the only edge types.
    children: HashMap<NodeID, Vec<(EdgeType<T>, NodeID)>>, // The children (output node IDs) and edge type of a output nodeID
    free_id: FreeIdManager<usize>,
    listeners: GraphListenerManager,
}

#[derive(Eq, PartialEq, Clone)]
pub struct PresenceGroups<T: DrawTag> {
    // A set of "parent groups" where for every parent group a unique node is created, NodeID here refers to an output NodeID
    groups: Vec<Vec<(EdgeConstraint<T>, NodeID)>>,
    // The way to handle how the presence for any parent node in any of the above defined groups
    remainder: PresenceRemainder,
}
impl<T: DrawTag> PresenceGroups<T> {
    pub fn new(
        groups: Vec<Vec<(EdgeConstraint<T>, NodeID)>>,
        remainder: PresenceRemainder,
    ) -> PresenceGroups<T> {
        PresenceGroups { groups, remainder }
    }

    pub fn remainder(remainder: PresenceRemainder) -> PresenceGroups<T> {
        PresenceGroups::new(Vec::new(), remainder)
    }
}

// #[derive(Eq, PartialEq, Clone)]
// pub enum PresenceGroup {
//     SharedRemainder,      // The original graph's node is used
//     DuplicateRemainder,   // One node is created per discovered parent
//     Parents(Vec<NodeID>), // One node is created for the specified parents (output node IDs)
// }
// Default presence group if not specified: PresenceGroups(Vec::from([PresenceGroup::SharedRemainder]))

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum EdgeConstraint<T: DrawTag> {
    Exact(EdgeType<T>),
    Any,
}

#[derive(Eq, PartialEq, Clone)]
pub enum PresenceRemainder {
    // Show this unique terminal the regular way (default)
    Show,
    // Hide this terminal
    Hide,
    // Make a unique instance of every occurrence of this terminal
    Duplicate,
    // Make a unique instance of every parent this terminal (multiple edges from the same parent share a single duplication)
    DuplicateParent,
}

// Values on the right side should only be used for nodes that are being adjusted to be duplicated, everything else retains the left version of the ID
type SourcedNodeID = Either<NodeID, NodeID>;
fn to_sourced(id: NodeID) -> SourcedNodeID {
    if id % 2 == 0 {
        Either::Left(id / 2)
    } else {
        Either::Right(id / 2)
    }
}
fn from_sourced(id: SourcedNodeID) -> NodeID {
    match id {
        Either::Left(id) => id * 2,
        Either::Right(id) => id * 2 + 1,
    }
}

fn get_all_copies<T: DrawTag>(
    node_data: Ref<NodeData<T>>,
    left_source_node: NodeID,
) -> Vec<NodeID> {
    let source_out = from_sourced(Either::Left(left_source_node));
    let maybe_images = node_data.images.get_vec(&left_source_node).cloned();
    if let Some(images) = maybe_images {
        let mut images = images
            .into_iter()
            .map(|image| from_sourced(Either::Right(image)))
            .collect_vec();
        images.push(source_out);
        images
    } else {
        vec![source_out]
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>>
    NodePresenceAdjuster<T, NL, LL, G>
{
    pub fn new(graph: G) -> NodePresenceAdjuster<T, NL, LL, G> {
        let mut adjuster = NodePresenceAdjuster {
            graph,
            node_data: Rc::new(RefCell::new(NodeData {
                adjustments: HashMap::new(),
                sources: HashMap::new(),
                images: MultiMap::new(),
                replacements: HashMap::new(),
                parent_nodes: HashMap::new(),
                known_parents: HashMap::new(),
                children: HashMap::new(),
                free_id: FreeIdManager::new(0),
                listeners: GraphListenerManager::new(),
            })),
            // tag: PhantomData,
            listener_handle: None,
            level_label: PhantomData,
            node_label: PhantomData,
        };
        adjuster.setup_listener_forwarding();
        adjuster
    }

    pub fn set_node_presence(&mut self, out_node: NodeID, presence: PresenceGroups<T>) {
        let owner = self.get_owner_id(out_node);

        // Create events for removal of the old node (connections) and images
        let owner_out = from_sourced(Either::Left(owner));
        self.add_remove_node_events(owner_out);
        let maybe_images = (*self.node_data).borrow().images.get_vec(&owner).cloned();
        if let Some(images) = maybe_images.clone() {
            for image in images {
                self.add_remove_node_events(from_sourced(Either::Right(image)));
            }
        }

        // Delete the old images
        if let Some(images) = maybe_images.clone() {
            for image in images {
                self.delete_replacement(image);
            }
        }

        // Determine the new images of the node
        {
            {
                let mut data = (*self.node_data).borrow_mut();
                data.adjustments.insert(owner, presence.clone());
            }

            // This automatically creates events for the created replacements
            for group in presence.groups {
                self.create_replacement(group, owner);
            }

            // Make sure that for all possible parents, the children are determined (and hence replacements are calculated if needed)
            let source_parents = self.graph.get_known_parents(owner);
            let parents = source_parents
                .iter()
                .flat_map(|(_, parent)| get_all_copies((*self.node_data).borrow(), *parent))
                .collect_vec();
            for parent in parents {
                self.update_children(parent);
            }
        }

        // Create an event for the replaced node
        if presence.remainder == PresenceRemainder::Show {
            self.add_insert_node_events(owner_out, owner_out);
        }
        // if let Some(images) = maybe_images {
        //     for image in images {
        //         self.add_insert_node_events(from_sourced(Either::Right(image)), owner_out);
        //     }
        // }

        // Emit the events
        (*self.node_data).borrow_mut().listeners.dispatch_change();
    }

    fn setup_listener_forwarding(&mut self) {
        let node_data = self.node_data.clone();
        self.listener_handle = Some(self.graph.on_change(Box::new(move |events| {
            // let data = (*node_data).borrow();

            let mapped_events = events
                .iter()
                .flat_map(|event| match event {
                    Change::NodeLabelChange { node } => {
                        get_all_copies((*node_data).borrow(), *node)
                            .into_iter()
                            .map(|node| Change::NodeLabelChange { node })
                            .collect_vec()
                    }
                    Change::LevelChange { node } => get_all_copies((*node_data).borrow(), *node)
                        .into_iter()
                        .map(|node| Change::LevelChange { node })
                        .collect_vec(),
                    Change::LevelLabelChange { level } => {
                        vec![Change::LevelLabelChange { level: *level }]
                    }
                    Change::NodeConnectionsChange { node } => {
                        get_all_copies((*node_data).borrow(), *node)
                            .into_iter()
                            .map(|node| Change::NodeConnectionsChange { node })
                            .collect_vec()
                    }
                    Change::NodeRemoval { node } => get_all_copies((*node_data).borrow(), *node)
                        .into_iter()
                        .map(|node| Change::NodeRemoval { node })
                        .collect_vec(),
                    Change::NodeInsertion { node, source } => {
                        get_all_copies((*node_data).borrow(), *node)
                            .into_iter()
                            .map(|node| Change::NodeInsertion {
                                node,
                                source: source.clone(),
                            })
                            .collect_vec()
                    }
                })
                .collect_vec();

            let listeners = &mut (*node_data).borrow_mut().listeners;
            listeners.dispatch_changes(&mapped_events);
        })));
    }

    fn add_neighbor_connection_change_events(&mut self, out_node: NodeID) {
        let parents = self.get_known_parents(out_node);
        let children = self.get_children(out_node);
        let listeners = &mut (*self.node_data).borrow_mut().listeners;
        for (_edge, parent) in parents {
            listeners.add_change(Change::NodeConnectionsChange { node: parent });
        }

        for (_edge, child) in children {
            listeners.add_change(Change::NodeConnectionsChange { node: child });
        }
    }

    fn add_remove_node_events(&mut self, out_node: NodeID) {
        self.add_neighbor_connection_change_events(out_node);

        let listeners = &mut (*self.node_data).borrow_mut().listeners;
        listeners.add_change(Change::NodeRemoval { node: out_node });
    }

    fn add_insert_node_events(&mut self, out_node: NodeID, source: NodeID) {
        self.add_neighbor_connection_change_events(out_node);

        let listeners = &mut (*self.node_data).borrow_mut().listeners;
        listeners.add_change(Change::NodeInsertion {
            node: out_node,
            source: Some(source),
        });
    }

    fn get_owner_id(&self, id: NodeID) -> NodeID {
        match to_sourced(id) {
            Either::Left(id) => id,
            Either::Right(id) => {
                let data = (*self.node_data).borrow();
                let Some(original_id) = data.sources.get(&id) else {
                    return 0; // Case should not be reachable
                };
                *original_id
            }
        }
    }

    fn create_replacement(
        &mut self,
        parents: Vec<(EdgeConstraint<T>, NodeID)>,
        child_to_be_replaced: NodeID,
    ) -> NodeID {
        let id = {
            let mut data = (*self.node_data).borrow_mut();
            let id = data.free_id.get_next();

            // Store the mapping
            data.sources.insert(id, child_to_be_replaced);
            data.images.insert(child_to_be_replaced, id);
            for (constraint, parent) in &parents {
                data.replacements
                    .insert((*parent, constraint.clone(), child_to_be_replaced), id);
            }

            // Store the parents
            data.parent_nodes
                .insert(id, parents.iter().map(|(_, parent)| *parent).collect());

            id
        };

        // Calculate the connections
        self.update_parents(id);
        let out_id = from_sourced(Either::Right(id));
        self.update_children(out_id);

        // Create a creation event
        self.add_insert_node_events(out_id, from_sourced(Either::Left(child_to_be_replaced)));

        id
    }

    fn delete_replacement(&mut self, node: NodeID) {
        let parents = self.get_known_parents(node);
        let mut data = (*self.node_data).borrow_mut();
        let Some(&source) = data.sources.get(&node) else {
            return;
        };

        for (edge, parent) in parents {
            data.replacements
                .remove(&(parent, EdgeConstraint::Exact(edge), source));
            data.replacements
                .remove(&(parent, EdgeConstraint::Any, source));
        }

        data.sources.remove(&node);
        if let Some(images) = data.images.get_vec_mut(&source) {
            images.remove(node);
            if images.len() == 0 {
                data.images.remove(&source);
            }
        }
        data.children.remove(&node);
        data.parent_nodes.remove(&node);
        data.known_parents.remove(&node);
        data.free_id.make_available(node);
    }

    fn update_parents(&mut self, right_node_id: NodeID) {
        let source_id = self.get_owner_id(from_sourced(Either::Right(right_node_id)));

        let parent_images: MultiMap<NodeID, NodeID> = {
            let data = (*self.node_data).borrow();
            let parent_nodes = data.parent_nodes.get(&right_node_id).unwrap();
            parent_nodes
                .iter()
                .map(|&parent| (self.get_owner_id(parent), parent))
                .sorted()
                .dedup()
                .collect()
        };

        let mut data = (*self.node_data).borrow_mut();
        let source_parents = self.graph.get_known_parents(source_id);
        let mut out_parents = Vec::new();
        for (edge, source_parent) in source_parents {
            let Some(parent_images) = parent_images.get_vec(&source_parent) else {
                continue;
            };
            for &parent in parent_images {
                if data
                    .replacements
                    .get(&(parent, EdgeConstraint::Exact(edge), source_id))
                    == Some(&right_node_id)
                    || data
                        .replacements
                        .get(&(parent, EdgeConstraint::Any, source_id))
                        == Some(&right_node_id)
                {
                    out_parents.push((edge, parent));
                }
            }
        }

        // console::log!(
        //     "update parents for {}: {}",
        //     right_node_id,
        //     out_parents.iter().map(|(_, k)| k.to_string()).join(", ")
        // );

        data.known_parents.insert(right_node_id, out_parents);
    }

    fn update_children(&mut self, out_node_id: NodeID) {
        let source_id = self.get_owner_id(out_node_id);

        // This is the only place that graph.get_children is called. Here we should also update our own "known_parents" accordingly
        let children = self.graph.get_children(source_id);

        let mut out = Vec::new();
        // Analyze the children and store them for future use
        for (edge_type, child) in children {
            let out_child = from_sourced(Either::Left(child));
            let remainder = {
                let data = (*self.node_data).borrow();
                if let Some(&replacement) =
                    data.replacements
                        .get(&(out_node_id, EdgeConstraint::Exact(edge_type), child))
                {
                    drop(data);
                    self.update_parents(replacement);
                    out.push((edge_type, from_sourced(Either::Right(replacement))));
                    continue;
                }

                if let Some(&replacement) =
                    data.replacements
                        .get(&(out_node_id, EdgeConstraint::Any, child))
                {
                    drop(data);
                    self.update_parents(replacement);
                    out.push((edge_type, from_sourced(Either::Right(replacement))));
                    continue;
                }

                let Some(adjustment) = data.adjustments.get(&child) else {
                    out.push((edge_type, out_child));
                    continue;
                };
                adjustment.remainder.clone()
            };

            match remainder {
                PresenceRemainder::Show => out.push((edge_type, out_child)),
                PresenceRemainder::Hide => {}
                PresenceRemainder::Duplicate => out.push((
                    edge_type,
                    from_sourced(Either::Right(self.create_replacement(
                        Vec::from([(EdgeConstraint::Exact(edge_type), out_node_id)]),
                        child,
                    ))),
                )),
                PresenceRemainder::DuplicateParent => out.push((
                    edge_type,
                    from_sourced(Either::Right(self.create_replacement(
                        Vec::from([(EdgeConstraint::Any, out_node_id)]),
                        child,
                    ))),
                )),
            }
        }

        // let has0 = out.iter().find(|p| p.1 == 0);
        // if has0.is_some() {
        //     console::log!("node0: {} ", out_node_id);
        // }
        let mut data = (*self.node_data).borrow_mut();
        data.children.insert(out_node_id, out);
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct PresenceLabel<LL> {
    original_label: LL,
    original_id: NodeID,
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>>
    GraphStructure<T, PresenceLabel<NL>, LL> for NodePresenceAdjuster<T, NL, LL, G>
{
    fn get_root(&self) -> NodeID {
        from_sourced(Either::Left(self.graph.get_root()))
    }
    fn get_terminals(&self) -> Vec<NodeID> {
        self.graph
            .get_terminals()
            .iter()
            .flat_map(|t| get_all_copies((*self.node_data).borrow(), *t))
            .collect()
    }

    fn get_known_parents(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)> {
        let parents = match to_sourced(node) {
            Either::Left(id) => {
                let data = (*self.node_data).borrow();
                let known_parents = self.graph.get_known_parents(id);
                // Filter parents to remove any parents that use a replacement node instead
                known_parents
                    .into_iter()
                    .map(|(edge, parent)| (edge, from_sourced(Either::Left(parent))))
                    .filter(|&(edge, out_parent)| {
                        let replaced = data.replacements.contains_key(&(
                            out_parent,
                            EdgeConstraint::Exact(edge.clone()),
                            id,
                        )) || data.replacements.contains_key(&(
                            out_parent,
                            EdgeConstraint::Any,
                            id,
                        ));
                        !replaced
                    })
                    .collect()
            }
            Either::Right(id) => {
                let data = (*self.node_data).borrow();
                data.known_parents
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| Vec::new())
            }
        };
        // console::log!(
        //     "{} parents: {}",
        //     node,
        //     parents.iter().map(|(_, v)| v.to_string()).join(", ")
        // );
        parents
    }

    fn get_children(&mut self, node: NodeID) -> Vec<(EdgeType<T>, NodeID)> {
        if let Some(children) = (*self.node_data).borrow().children.get(&node) {
            // console::log!(
            //     "{} children: {}",
            //     node,
            //     children.iter().map(|(_, v)| v.to_string()).join(", ")
            // );
            // if let Either::Left(_) = to_sourced(node) {
            //     let data = (*self.node_data).borrow();
            //     console::log!(
            //         "{} children: {}",
            //         node,
            //         data.children
            //             .get(&node)
            //             .unwrap()
            //             .iter()
            //             .map(|(_, v)| v.to_string())
            //             .join(", ")
            //     );
            // }
            return children.clone();
        }

        match to_sourced(node) {
            Either::Left(_) => {
                self.update_children(node);

                let mut data = (*self.node_data).borrow_mut();
                data.listeners.dispatch_change();
                // console::log!(
                //     "{} children: {}",
                //     node,
                //     data.children
                //         .get(&node)
                //         .unwrap()
                //         .iter()
                //         .map(|(_, v)| v.to_string())
                //         .join(", ")
                // );
                return data.children.get(&node).cloned().unwrap();
            }
            Either::Right(_) => {
                // This should not be able to happen, since any such node should have registered children
                return Vec::new();
            }
        }
    }

    fn get_level(&mut self, node: NodeID) -> LevelNo {
        let id = self.get_owner_id(node);
        // if self.graph.get_level(id) == 0 {
        //     console::log!(
        //         "node: {}, id: {}, level: {}",
        //         node,
        //         id,
        //         self.graph.get_level(id)
        //     );
        // }
        // if let Either::Right(_) = to_sourced(node) {
        //     console::log!(
        //         "node: {}, id: {}, level: {}",
        //         node,
        //         id,
        //         self.graph.get_level(id)
        //     );
        // }

        // TODO: store custom levels for terminals
        self.graph.get_level(id)
    }

    fn get_node_label(&self, node: NodeID) -> PresenceLabel<NL> {
        let id = self.get_owner_id(node);
        PresenceLabel {
            original_id: id,
            original_label: self.graph.get_node_label(id),
        }
    }

    fn get_level_label(&self, level: LevelNo) -> LL {
        self.graph.get_level_label(level)
    }

    fn on_change(&mut self, listener: Box<GraphListener>) -> usize {
        let listeners = &mut (*self.node_data).borrow_mut().listeners;
        listeners.add(listener)
    }

    fn off_change(&mut self, listener: usize) {
        let listeners = &mut (*self.node_data).borrow_mut().listeners;
        listeners.remove(listener)
    }
}

impl<T: DrawTag, NL: Clone, LL: Clone, G: GraphStructure<T, NL, LL>> Drop
    for NodePresenceAdjuster<T, NL, LL, G>
{
    fn drop(&mut self) {
        if let Some(listener_handle) = self.listener_handle {
            self.graph.off_change(listener_handle);
        }
    }
}
