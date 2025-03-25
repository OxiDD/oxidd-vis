use std::collections::HashMap;

use itertools::Itertools;
use num_rational::Ratio;
use oxidd_core::Tag;

use crate::{
    types::util::{
        drawing::layouts::{
            layered_layout_traits::LayerGroupSorting,
            util::layered::layer_orderer::{EdgeMap, Order},
        },
        graph_structure::{
            graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure,
        },
    },
    wasm_interface::NodeGroupID,
};

pub struct AverageGroupAlignment;

impl<T: DrawTag, GL, LL> LayerGroupSorting<T, GL, LL> for AverageGroupAlignment {
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
        let mut sums = HashMap::<NodeGroupID, (usize, usize)>::new();
        for (node, &index) in layers.iter().flatten() {
            let Some(&owner) = owners.get(node) else {
                continue;
            };
            sums.entry(owner)
                .and_modify(|(index_sum, count)| {
                    *index_sum += index;
                    *count += 1;
                })
                .or_insert((index, 1));
        }
        let averages: HashMap<NodeGroupID, Ratio<usize>> = sums
            .iter()
            .map(|(&owner, &(index_sum, count))| (owner, Ratio::new(index_sum, count)))
            .collect();

        layers
            .iter()
            .map(|layer| {
                layer
                    .iter()
                    .map(|(group, &index)| {
                        (
                            group,
                            owners
                                .get(group)
                                .map_or(Ratio::from_integer(index), |owner| {
                                    *averages.get(owner).unwrap()
                                }),
                        )
                    })
                    .sorted_by_key(|(&group, index)| (index.clone(), group))
                    .enumerate()
                    .map(|(index, (&group, _))| (group, index))
                    .collect()
            })
            .collect()
    }
}
