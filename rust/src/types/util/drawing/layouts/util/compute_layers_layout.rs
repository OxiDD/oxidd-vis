use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use itertools::Itertools;
use oxidd::LevelNo;
use oxidd_core::Tag;
use priority_queue::PriorityQueue;

use crate::{
    types::util::{
        drawing::diagram_layout::{LayerLayout, Point, Transition},
        grouped_graph_structure::GroupedGraphStructure,
    },
    util::{logging::console, rectangle::Rectangle},
};

pub fn compute_layers_layout<
    T: Tag,
    G: GroupedGraphStructure<T>,
    I: Iterator<Item = (usize, Rectangle)>,
>(
    graph: &G,
    node_positions: I,
) -> Vec<LayerLayout> {
    let mut layer_start_positions = HashMap::<LevelNo, f32>::new();
    let mut layer_end_positions = HashMap::<LevelNo, f32>::new();
    for (group_id, point) in node_positions {
        let (start, end) = graph.get_level_range(group_id);
        let start_y = point.y + point.height;
        layer_start_positions
            .entry(start)
            .and_modify(|v| *v = v.max(start_y))
            .or_insert(start_y);
        let end_y = point.y;
        layer_end_positions
            .entry(end)
            .and_modify(|v| *v = v.min(end_y))
            .or_insert(end_y);
    }
    let layer_positions = layer_start_positions
        .keys()
        .cloned()
        .chain(layer_end_positions.keys().map(|layer| layer + 1))
        .sorted()
        .dedup()
        .map(|level| {
            let end = if level > 0 {
                layer_end_positions.get(&(level - 1))
            } else {
                None
            };
            let start = layer_start_positions.get(&level);
            (
                level,
                match (start, end) {
                    (Some(s), Some(e)) => (s + e) / 2.,
                    (Some(&s), None) => s,
                    (None, Some(&e)) => e,
                    _ => 0.,
                },
            )
        });

    let mut layout: Vec<LayerLayout> = Vec::new();
    let mut min: Option<f32> = None;
    let mut index = 0;
    for ((start_layer, start_y), (end_layer, end_y)) in
        layer_positions.clone().zip(layer_positions.skip(1))
    {
        let start_y = min.map(|m| m.min(start_y)).unwrap_or(start_y);
        let end_y = end_y.min(start_y);
        min = Some(end_y);

        layout.push(LayerLayout {
            start_layer,
            end_layer,
            index: Transition::plain(index as f32),
            label: (start_layer..end_layer)
                .map(|level| graph.get_level_label(level))
                .join(", \n"),
            top: Transition::plain(start_y),
            bottom: Transition::plain(end_y),
            exists: Transition::plain(1.),
        });
        index += 1;
    }
    layout
}
