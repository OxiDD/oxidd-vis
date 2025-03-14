use core::f32;

use itertools::Itertools;

use crate::{
    types::util::{
        drawing::{
            diagram_layout::{DiagramLayout, LayerStyle, NodeStyle},
            renderer::{GroupSelection, Renderer},
        },
        graph_structure::graph_structure::DrawTag,
    },
    util::{logging::console, transformation::Transformation},
};

pub struct LatexRenderer<T: DrawTag, S: NodeStyle, LS: LayerStyle> {
    output: String,
    layout: Option<DiagramLayout<T, S, LS>>,
}

impl<T: DrawTag, S: LatexNodeStyle, LS: LatexLayerStyle> LatexRenderer<T, S, LS> {
    pub fn new() -> LatexRenderer<T, S, LS> {
        LatexRenderer {
            output: "".into(),
            layout: None,
        }
    }

    pub fn get_output(&self) -> String {
        self.output.clone()
    }
}

impl<T: DrawTag, S: LatexNodeStyle, LS: LatexLayerStyle> Renderer<T, S, LS>
    for LatexRenderer<T, S, LS>
{
    fn set_transform(&mut self, transform: Transformation) {
        todo!()
    }

    fn update_layout(&mut self, layout: &DiagramLayout<T, S, LS>) {
        self.layout = Some(layout.clone());
    }

    fn render(&mut self, time: u32) {
        let Some(layout) = &self.layout else {
            return;
        };

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let nodes = layout
            .groups
            .iter()
            .filter_map(|(id, group)| {
                if group.exists.get(time) < 1.0 {
                    return None;
                }
                let pos = group.position.get(time);
                let size = group.size.get(time);
                if pos.x - 0.5*size.x < min_x {
                    min_x = pos.x - 0.5*size.x;
                }
                if pos.x + 0.5*size.x > max_x {
                    max_x = pos.x + 0.5*size.x;
                }
                let style = group.style.get(time);
                let label = sanitize(style.get_label().unwrap_or_default());

                let x = pos.x;
                let y = pos.y + 0.5 * size.y;
                if let Some((terminal_type, terminal_label)) = style.is_terminal() {
                    Some(format!(
                        "\\node[{}] (n{}) at ({}, {}) {{{}}};",
                        terminal_type, id, x, y, terminal_label.unwrap_or_else(|| format!("\\pgfkeysvalueof{{/tikz/{}/label}}", terminal_type))
                    ))
                } else if style.is_group() {
                    Some(format!(
                        "\\node[group, minimum width={}*\\unit cm, minimum height={}*\\unit cm] (n{}) at ({}, {}) {{{}}};",
                        size.x, size.y, id, x, y, label
                    ))
                } else if (size.y - size.x).abs() <= f32::EPSILON {
                    Some(format!(
                        "\\node[inner, minimum size={}*\\unit cm] (n{}) at ({}, {}) {{{}}};",
                        size.y, id, x, y,  label
                    ))
                } else {
                    Some(format!(
                        "\\node[innerSized, minimum width={}*\\unit cm, minimum height={}*\\unit cm] (n{}) at ({}, {}) {{{}}};",
                        size.x, size.y, id, x, y, label
                    ))
                }
            })
            .join("\n    ");

        let edges = layout
            .groups
            .iter()
            .flat_map(|(group_id, group)| {
                for (_, edge) in group.edges.iter() {
                    for p in edge.points.iter() {
                        let p = p.point.get(time);
                        if p.x > max_x {
                            max_x = p.x;
                        }
                        if p.x < min_x {
                            min_x = p.x;
                        }
                    }
                }
                group.edges.iter().filter_map(move |(edge_data, edge)| {
                    if edge.exists.get(time) < 1.0 {
                        return None;
                    }
                    let Some(target_group) = layout.groups.get(&edge_data.to) else {
                        return None;
                    };
                    let start_pos = group.position.get(time) + edge.start_offset.get(time);
                    let end_pos = target_group.position.get(time) + edge.end_offset.get(time);
                    let start_next_pos = edge
                        .points
                        .get(0)
                        .map(|e| e.point.get(time))
                        .unwrap_or(end_pos);
                    let end_previous_pos = Some(edge.points.len())
                        .filter(|&v| v > 0)
                        .and_then(|v| edge.points.get(v - 1))
                        .map(|e| e.point.get(time))
                        .unwrap_or(start_pos);

                    let start_delta = start_next_pos - start_pos;
                    let (is_start_side, start_side) = if group.level_range.0 == group.level_range.1
                    {
                        (false, "")
                    } else if start_delta.x > 0. {
                        (true, ".east")
                    } else if start_delta.x < 0. {
                        (true, ".west")
                    } else {
                        (false, ".south")
                    };
                    let start_offset = if is_start_side {
                        edge.start_offset.get(time).y - 0.5 * group.size.get(time).y
                    } else {
                        0.0
                    };
                    let start_offset = if start_offset == 0.0 {
                        ""
                    } else {
                        &format!("[yshift={}*\\unit cm] ", start_offset)
                    };

                    let end_delta = end_pos - end_previous_pos;
                    let (is_end_side, end_side) =
                        if target_group.level_range.0 == target_group.level_range.1 {
                            (false, "")
                        } else if end_delta.x > 0. {
                            (true, ".west")
                        } else if end_delta.x < 0. {
                            (true, ".east")
                        } else {
                            (false, ".north")
                        };
                    let end_offset = if is_end_side {
                        edge.end_offset.get(time).y - 0.5 * target_group.size.get(time).y
                    } else {
                        0.0
                    };
                    let end_offset = if end_offset == 0.0 {
                        ""
                    } else {
                        &format!("[yshift={}*\\unit cm] ", end_offset)
                    };

                    let intermediate_points = edge
                        .points
                        .iter()
                        .filter_map(|p| {
                            if p.exists.get(time) < 1.0 {
                                return None;
                            }
                            let p = p.point.get(time);
                            Some(format!("({}, {}) to ", p.x, p.y))
                        })
                        .join("");

                    Some(format!(
                        "\\draw[choice{}] ({}n{}{}) to[bend left={}] {}({}n{}{});",
                        edge_data.edge_type.index,
                        start_offset,
                        group_id,
                        start_side,
                        edge.curve_offset.get(time) * 45.0,
                        intermediate_points,
                        end_offset,
                        edge_data.to,
                        end_side
                    ))
                })
            })
            .join("\n    ");

        let layers = layout
            .layers
            .iter()
            .enumerate()
            .filter_map(|(index, layer)| {
                if layer.exists.get(time) < 1.0 {
                    return None;
                }
                let top = layer.top.get(time);
                let bottom = layer.bottom.get(time);
                let style = layer.style.get(time);
                console::log!("minX: {}, maxX: {}", min_x, max_x);
                let label = format!(
                    "\\node[layerLabel] (l-{}) at ({}-\\ts, {}) {{{}}};",
                    index,
                    min_x,
                    0.5 * (top + bottom),
                    sanitize(style.get_label())
                );
                let bottom_divider = format!(
                    "\\draw[layerDivider] ({}-\\margin-\\ts, {}) -- ({}+\\margin, {});",
                    min_x, bottom, max_x, bottom
                );
                let top_divider = format!(
                    "\\draw[layerDivider] ({}-\\margin-\\ts, {}) -- ({}+\\margin, {});",
                    min_x, top, max_x, top
                );
                if index == 0 {
                    Some(format!(
                        "{}\n    {}\n    {}",
                        top_divider, label, bottom_divider
                    ))
                } else {
                    Some(format!("{}\n    {}", label, bottom_divider))
                }
            })
            .join("\n    ");

        let out = format!(
            "\\begin{{tikzpicture}}\n    \
            \\pgfmathsetmacro{{\\margin}}{{0.5}} % spacing around diagram on left and right \n    \
            \\pgfmathsetmacro{{\\ts}}{{2}} % the spacing available for variables \n    \
            \n    \
            \\pgfmathsetmacro{{\\unit}}{{veclen(0,1)}}\n    \
            \n    \
            % Layers \n    \
            {}\n    \
            \n    \
            % Nodes \n    \
            {}\n    \
            \n    \
            % Edges \n    \
            {}\n\
            \\end{{tikzpicture}}",
            layers, nodes, edges
        );

        self.output = out;
    }

    fn select_groups(&mut self, selection: GroupSelection, old_selection: GroupSelection) {
        todo!()
    }
}

fn sanitize(text: String) -> String {
    text.replace("_", "\\_")
}

pub trait LatexNodeStyle: NodeStyle {
    /// Retrieves whether the given node is a terminal, and if so: retrieves the terminal type, and optionally a label
    fn is_terminal(&self) -> Option<(String, Option<String>)>;
    fn is_group(&self) -> bool;
    fn get_label(&self) -> Option<String>;
}
pub trait LatexLayerStyle: LayerStyle {
    fn get_label(&self) -> String;
}
