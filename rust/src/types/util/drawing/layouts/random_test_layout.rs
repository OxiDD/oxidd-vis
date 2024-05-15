use std::collections::HashMap;

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
    group_manager::GroupManager,
};

pub struct RandomTestLayout;

impl<T: Tag> LayoutRules<T> for RandomTestLayout {
    fn layout(
        &mut self,
        groups: &GroupManager<T>,
        old: &DiagramLayout<T>,
        time: u32,
    ) -> DiagramLayout<T> {
        let groups = groups.get_groups();
        DiagramLayout {
            groups: groups
                .iter()
                .map(|(&id, group)| {
                    (id, {
                        let x: f32 = (random() * 20. - 10.) as f32;
                        let y: f32 = (random() * 20. - 10.) as f32;
                        let width: f32 = (random() * 1. + 0.5) as f32;
                        let height: f32 = (random() * 1. + 0.5) as f32;

                        NodeGroupLayout {
                            label: id.to_string(),
                            // center_position: Transition::plain(Point { x, y }),
                            center_position: Transition {
                                old: Point { x: 0.0, y: 0.0 },
                                new: Point { x, y },
                                old_time: time,
                                duration: 1000,
                            },
                            // size: Transition::plain(Point {
                            //     x: width,
                            //     y: height,
                            // }),
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
                            edges: group
                                .out_edges
                                .iter()
                                .map(|(&to, edges)| {
                                    (
                                        to,
                                        edges
                                            .iter()
                                            .map(|(&edge_type, _)| {
                                                (
                                                    edge_type,
                                                    EdgeLayout {
                                                        points: (vec![
                                                            0;
                                                            (random() * 3.0) as usize
                                                        ])
                                                        .iter()
                                                        .map(|_| EdgePoint {
                                                            point: Transition {
                                                                old: Point { x: 0.0, y: 0.0 },
                                                                new: Point {
                                                                    x: (random() * 20. - 10.)
                                                                        as f32,
                                                                    y: (random() * 20. - 10.)
                                                                        as f32,
                                                                },
                                                                old_time: time,
                                                                duration: 1000,
                                                            },
                                                            exists: Transition::plain(1.),
                                                        })
                                                        .collect(),
                                                        exists: Transition::plain(1.),
                                                    },
                                                )
                                            })
                                            .collect(),
                                    )
                                })
                                .collect(),
                        }
                    })
                })
                .collect(),
            layers: HashMap::new(),
        }
    }
}
