use std::{collections::HashMap, usize};

use itertools::Itertools;
use num_rational::Ratio;
use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::layouts::{
            layered_layout::LayerGroupSorting,
            util::layered::layer_orderer::{EdgeMap, Order},
        },
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
    },
    util::logging::console,
    wasm_interface::NodeGroupID,
};

pub struct OrderingGroupAlignment;

/// Sorts each layer according to the group order in the previous layer. This attempts to retain the original ordering, while fulfilling the requirement: No two groups can cross (it's always either fully before or fully after another group, or does not occur on the same layer).
impl<T: DrawTag, GL, LL> LayerGroupSorting<T, GL, LL> for OrderingGroupAlignment {
    fn align_cross_layer_nodes(
        &self,
        _graph: &impl GroupedGraphStructure<T, GL, LL>,
        layers: &Vec<Order>,
        _edges: &EdgeMap,
        // The ID such that any ID in the range [dummy_group_start_id, dummy_edge_start_id) represents a dummy node of a group
        _dummy_group_start_id: NodeGroupID,
        // The ID such that any ID greater or equal represents a dummy node of an edge
        _dummy_edge_start_id: NodeGroupID,
        // The owner of a given dummy node, such that multiple nodes derived from the same data can be considered as a group
        owners: &HashMap<NodeGroupID, NodeGroupID>,
    ) -> Vec<Order> {
        let mut out = Vec::new();

        let mut group_index = HashMap::<NodeGroupID, usize>::new();
        for layer in layers {
            // Obtain the nextGroup for any node
            let mut layer_next_groups = Vec::new();
            let mut next_group: Option<NodeGroupID> = None;
            for (&node, index) in layer
                .iter()
                .map(|(node, &index)| (node, index))
                .sorted_by_key(|(_, index)| *index)
                .rev()
            {
                let self_group = owners.get(&node).and_then(|owner| {
                    if group_index.contains_key(owner) {
                        Some(*owner)
                    } else {
                        None
                    }
                });
                let g = if let Some(group) = self_group {
                    let new_next = next_group.is_none()
                        || (group_index.get(&group).unwrap()
                            < group_index.get(&next_group.unwrap()).unwrap());
                    if new_next {
                        next_group = Some(group);
                    }
                    Some(group)
                } else {
                    next_group
                };
                layer_next_groups.push((node, index, g));
            }

            // Sort the nodes for this layer
            let sorted = layer_next_groups
                .iter()
                .sorted_by(|a, b| {
                    let ag =
                        a.2.map(|g| group_index.get(&g).unwrap())
                            .unwrap_or(&usize::MAX);
                    let bg =
                        b.2.map(|g| group_index.get(&g).unwrap())
                            .unwrap_or(&usize::MAX);
                    if ag != bg {
                        ag.cmp(bg)
                    } else {
                        a.1.cmp(&b.1)
                    }
                })
                .enumerate()
                .map(|(index, (node, _, _))| (*node, index));

            // Update the indices
            group_index.clear();
            group_index.extend(
                sorted
                    .clone()
                    .filter_map(|(node, index)| owners.get(&node).map(|&group| (group, index))),
            );

            // Store the output layer
            out.push(sorted.map(|(node, index)| (node, index)).collect());
        }

        out
    }
}

/*
  Formalization:
  Let a node n belong to a group N = group(n) which may be null, have a integer layer l = layer(n), and an index in the layer i = index(n). Let a group be an ordered sequence of nodes, and let nodes(l) be the nodes of a given layer.

  For group g and layer l, we define function index(g, l) = {
    g = null -> Infinity
    g != null -> {
        let n ∈ g where layer(n) = l
        index of n in ordering (nodes(l), <=) // A relationship we will define below
    }
  }

  Define function nextGroup(n) = {
    let after = {m ∈ nodes(layer(n)) | index(m) >= index(n)}
    let group_after = {m ∈ after | group(m) != null && group(m)[0] != m} // The group exists before this layer

    group(n) ∈ group_after    -> group(n)
    !(group(n) ∈ group_after) -> arg_min {g ∈ group_after} (index(g, layer(n)-1))  // May be null
  }

  For nodes n and m on the same layer l:
  n <= m iff S(n, m) with S(a, b) = {
    let ag = index(nextGroup(a), l-1)
    let bg = index(nextGroup(b), l-1)
    ag <= bg || (ag = bg && index(a) <= index(b))
  }

  Then each layer will be sorted according to the <= relationship. This is clearly a total ordering on a layer, since it is a lexicographical ordering on indices of the groups and then the element indices. The element indices are not shared between elements and hence make up a total ordering.
  These definitions contain some cyclic dependencies on one and another, so it is not immediately obvious that they are well defined. However any reference to `index(g, l)` is made with a layer that's smaller than the current argument, and hence these expressions do not cyclically depend on each-other.

  This ordering fulfills our criteria:
  No group crossings: Assume there is a crossing between two distinct groups A and B, between layers l-1 and l.
    Then there are nodes a ∈ A with layer(a) = l-1, a' ∈ A with layer(a') = l,
                         b ∈ B with layer(b) = l-1, b' ∈ B with layer(b') = l
    such that (a <= b && b' <= a') or (b <= a && a' <= b'). WLOG assume a <= b && b' <= a'.
    Since b' <= a', nextGroup(a) = group(a) = A, nextGroup(b) = group(b) = B, and index(A, l-1) != index(B, l-1) (distinct groups) we know index(B, l-1) <= index(A, l-1).
    index(A, l-1) is equal to the index of a in the ordering (nodes(l-1), <=), and index(B, l-1) is equal to the index of b in the ordering (nodes(l-1), <=). Hence we must therefor have that b <= a, which contradicts that a <= b which is required for obtaining a crossing.
*/
