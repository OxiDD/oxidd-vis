use std::collections::HashMap;

use itertools::Itertools;
use js_sys::Math::random;
use oxidd::{Edge, Function, InnerNode, Manager};
use oxidd_core::{DiagramRules, Tag};

use crate::types::util::{
    drawing::{
        diagram_layout::{
            DiagramLayout, EdgeLayout, EdgePoint, NodeGroupLayout, Point, Transition,
        },
        layout_rules::LayoutRules,
    },
    graph_structure::{graph_structure::DrawTag, grouped_graph_structure::GroupedGraphStructure},
};

pub struct RandomTestLayout;

impl<T: DrawTag, GL, LL, G: GroupedGraphStructure<T, GL, LL>> LayoutRules<T, GL, LL, G>
    for RandomTestLayout
{
    fn layout(
        &mut self,
        graph: &G,
        old: &DiagramLayout<T>,
        sources: &G::Tracker,
        time: u32,
    ) -> DiagramLayout<T> {
        let groups = graph.get_all_groups();
        DiagramLayout {
            groups: groups
                .iter()
                .map(|&group_id| {
                    (group_id, {
                        let x: f32 = (random() * 20. - 10.) as f32;
                        let y: f32 = (random() * 20. - 10.) as f32;
                        let width: f32 = (random() * 1. + 0.5) as f32;
                        let height: f32 = (random() * 1. + 0.5) as f32;

                        NodeGroupLayout {
                            label: group_id.to_string(),
                            // center_position: Transition::plain(Point { x, y }),
                            position: Transition {
                                old: Point { x: 0.0, y: 0.0 },
                                new: Point { x, y },
                                old_time: time,
                                duration: 1000,
                            },
                            level_range: graph.get_level_range(group_id),
                            // size: Transition::plain(Point {
                            //     x: width,
                            //     y: height,
                            // }),
                            color: Transition {
                                old: (0., 0., 0.),
                                new: (random() as f32, random() as f32, random() as f32),
                                old_time: time,
                                duration: 1000,
                            },
                            size: Transition {
                                old: Point { x: 0.0, y: 0.0 },
                                new: Point {
                                    x: width,
                                    y: height,
                                },
                                old_time: time,
                                duration: 1000,
                            },
                            exists: Transition::plain(1.),
                            edges: graph
                                .get_children(group_id)
                                .into_iter()
                                .map(|edge_data| {
                                    (edge_data.drop_count(), {
                                        EdgeLayout {
                                            start_offset: Transition::plain(Point { x: 0., y: 0. }),
                                            end_offset: Transition::plain(Point { x: 0., y: 0. }),
                                            points: (vec![0; (random() * 3.0) as usize])
                                                .iter()
                                                .map(|_| EdgePoint {
                                                    point: Transition {
                                                        old: Point { x: 0.0, y: 0.0 },
                                                        new: Point {
                                                            x: (random() * 20. - 10.) as f32,
                                                            y: (random() * 20. - 10.) as f32,
                                                        },
                                                        old_time: time,
                                                        duration: 1000,
                                                    },
                                                    exists: Transition::plain(1.),
                                                })
                                                .collect(),
                                            exists: Transition::plain(1.),
                                            curve_offset: Transition::plain(0.),
                                        }
                                    })
                                })
                                .collect(),
                        }
                    })
                })
                .collect(),
            layers: Vec::new(),
        }
    }
}
