use std::collections::HashMap;

use js_sys::Math::random;
use oxidd::{Edge, Function, InnerNode, Manager};
use oxidd_core::{DiagramRules, Tag};

use crate::types::util::{
    drawing::{
        diagram_layout::{DiagramLayout, EdgeLayout, NodeGroupLayout, Point, Transition},
        layout_rules::LayoutRules,
    },
    group_manager::GroupManager,
};

pub struct RandomTestLayout;

impl<ET: Tag, T, E: Edge<Tag = ET>, N: InnerNode<E>, R: DiagramRules<E, N, T>, F: Function>
    LayoutRules<ET, F> for RandomTestLayout
where
    for<'id> F::Manager<'id>:
        Manager<EdgeTag = ET, Edge = E, InnerNode = N, Rules = R, Terminal = T>,
{
    fn layout(
        &mut self,
        groups: &GroupManager<ET, F>,
        old: &DiagramLayout<ET>,
    ) -> DiagramLayout<ET> {
        let groups = groups.get_groups();
        DiagramLayout {
            groups: groups
                .iter()
                .map(|(&id, group)| {
                    (id, {
                        let x: f32 = (random() * 40. - 20.) as f32;
                        let y: f32 = (random() * 40. - 20.) as f32;
                        let width: f32 = (random() * 1. + 0.5) as f32;
                        let height: f32 = (random() * 1. + 0.5) as f32;

                        NodeGroupLayout {
                            label: id.to_string(),
                            top_left: Transition::plain(Point { x, y }),
                            size: Transition::plain(Point {
                                x: width,
                                y: height,
                            }),
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
                                                (edge_type, EdgeLayout { points: Vec::new() })
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
