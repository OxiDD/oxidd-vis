use web_sys::WebGl2RenderingContext;

use crate::{
    types::util::drawing::diagram_layout::{Point, Transition},
    util::{logging::console, matrix4::Matrix4},
};

use super::vertex_renderer::VertexRenderer;

pub struct EdgeRenderer {
    vertex_renderer: VertexRenderer,
    edge_types: Vec<EdgeRenderingType>,
}

pub struct Edge {
    pub start: Transition<Point>,
    pub points: Vec<Transition<Point>>,
    pub end: Transition<Point>,
    pub edge_type: usize,
    pub shift: Transition<f32>, // Some sideways shift
}

#[derive(Clone)]
pub struct EdgeRenderingType {
    pub color: (f32, f32, f32),
    pub width: f32,
    pub dash_solid: f32, // The distance per period over which this dash should be solid
    pub dash_transparent: f32, // The distance per
}
type Segment = (
    Transition<Point>,
    Transition<Point>,
    f32,             /* type*/
    Transition<f32>, /* curvature */
);

impl EdgeRenderer {
    pub fn new(
        context: &WebGl2RenderingContext,
        edge_types: Vec<EdgeRenderingType>,
    ) -> EdgeRenderer {
        let type_count = edge_types.len();
        console::log!("uniform EdgeType types[{type_count}];");
        let vertex_renderer = VertexRenderer::new(
            context,
            &format!(r##"#version 300 es
                struct EdgeType {{
                    vec3 color;
                    float width;
                    float dashSolid;
                    float dashTransparent;
                }};
                
                in vec2 start;
                in vec2 startOld;
                in float startStartTime;
                in float startDuration;

                in vec2 end;
                in vec2 endOld;
                in float endStartTime;
                in float endDuration;
                
                in float curveOffset;
                in float curveOffsetOld;
                in float curveOffsetStartTime;
                in float curveOffsetDuration;
                
                in float type;
                out float outType;
                
                out vec2 curStart;
                out vec2 curEnd;
                out vec2 outPos;
                out float curCurveOffset;
                out float radius;
                out vec2 center;
                
                uniform EdgeType edgeTypes[{type_count}];
                uniform mat4 transform;
                uniform float time;

                void main() {{
                    outType = type;
                    
                    float startPer = min((time - startStartTime) / startDuration, 1.0);
                    curStart = startPer * start + (1.0 - startPer) * startOld;
                    float halfWidth = 0.5 * edgeTypes[int(type)].width;

                    float endPer = min((time - endStartTime) / endDuration, 1.0);
                    curEnd = endPer * end + (1.0 - endPer) * endOld;

                    float curvePer = min((time - curveOffsetStartTime) / curveOffsetDuration, 1.0);
                    curCurveOffset = curvePer * curveOffset + (1.0 - curvePer) * curveOffsetOld;

                    vec2 delta = curEnd - curStart;
                    vec2 dir = normalize(delta);
                    vec2 dirOrth = vec2(-dir.y, dir.x);

                    bool p = curCurveOffset > 0.; // Whether the curvature is to the right
                    float halfLength = 0.5 * length(delta);
                    float curveWidth = min(abs(curCurveOffset), halfLength);
                    float centerDeltaX = ((curveWidth * curveWidth) - (halfLength * halfLength)) / (2.0 * curveWidth);
                    center = 0.5 * (curEnd + curStart) + dirOrth * centerDeltaX * (p ? 1. : -1.);
                    radius = abs(centerDeltaX) + curveWidth;

                    int corner = gl_VertexID % 6; // two triangles
                    outPos = (
                        corner == 0                  ? curStart + (-dir * halfWidth - dirOrth * (halfWidth + (p ? 0. : curveWidth))) 
                        : corner == 1 || corner == 3 ? curStart + (-dir * halfWidth + dirOrth * (halfWidth + (p ? curveWidth : 0.))) 
                        : corner == 2 || corner == 4 ? curEnd + (- dirOrth * (halfWidth + (p ? 0. : curveWidth))) 
                        :                              curEnd + (+ dirOrth * (halfWidth + (p ? curveWidth : 0.))) 
                    );
                    gl_Position = transform * vec4(outPos, 0.0, 1.0) * vec4(vec3(2.0), 1.0); // 2 to to make the default width and height of the screen 1, instead of 2
                }}
                "##),
                &format!(r##"#version 300 es
                precision highp float;
                
                #define M_PI 3.1415926535897932384626433832795

                struct EdgeType {{
                    vec3 color;
                    float width;
                    float dashSolid;
                    float dashTransparent;
                }};

                out vec4 outColor;


                in vec2 curStart;
                in vec2 curEnd;
                in vec2 outPos;

                in float outType;
                in float curCurveOffset;
                in float radius;
                in vec2 center;

                uniform EdgeType edgeTypes[{type_count}];
                uniform mat4 transform;

                float fuzziness = 0.003; // A form of anti-aliasing by making the circle border a slight gradient
                
                // Ensures that the output angle is specified such that it's greater than the reference angle
                float getAngle(vec2 point, float refAngle) {{
                    vec2 delta = point - center;
                    float angle = atan(delta.y, delta.x);
                    return mod(angle - refAngle + 2.*M_PI, 2.*M_PI) + refAngle;
                }}

                void main() {{
                    EdgeType typeData = edgeTypes[int(outType)];
                    float halfWidth = 0.5 * typeData.width;
                    float alpha = 1.0;
                    float scaledFuzziness = fuzziness / transform[0][0];
                    float cor = 0.5 * scaledFuzziness;
                    float halfWidthSquared = (halfWidth - cor)*(halfWidth - cor);

                    float proj; 
                    float projPer;
                    bool onLine;
                    
                    if (abs(curCurveOffset) > 0.0) {{
                        vec2 centerDelta = outPos - center;
                        float dist = length(centerDelta);
                        float distDelta = abs(dist - radius);
                        
                        float startAngle;
                        float endAngle;
                        float pointAngle;
                        if (curCurveOffset > 0.) {{
                            endAngle = getAngle(curEnd, 0.0);
                            startAngle = getAngle(curStart, endAngle);
                            pointAngle = getAngle(outPos, endAngle);
                        }} else {{
                            startAngle = getAngle(curStart, 0.0);
                            endAngle = getAngle(curEnd, startAngle);
                            pointAngle = getAngle(outPos, startAngle);
                        }}

                        float arcLength = abs(endAngle - startAngle) * radius;
                        proj = abs(pointAngle - startAngle) * radius;

                        projPer = proj / arcLength;
                        onLine = projPer >= 0.0 && projPer <= 1.0;
                        
                        // if(startAngle < pointAngle) {{
                        //     onLine = false;
                        // }}
                        if(distDelta > halfWidth) {{
                            onLine = false;
                        }}
                    }} else {{
                        vec2 line = curEnd - curStart;
                        vec2 point = outPos - curStart;

                        proj = dot(point, normalize(line));
                        projPer = proj / length(line);
                        onLine = projPer >= 0.0 && projPer <= 1.0;
                    }}

                    if(!onLine) {{
                        // Only draw half circle from one side
                        if(projPer >= 1.0)
                            alpha = 0.0;
                        else {{
                            vec2 delta1 = curStart - outPos;
                            vec2 delta2 = curEnd - outPos;
                            float distSquared = min(dot(delta1, delta1), dot(delta2, delta2));
                            
                            if(distSquared >= halfWidthSquared) 
                                // alpha = 1.0 - max(0.0, (sqrt(distSquared) - (width - cor)) / scaledFuzziness);
                                alpha = 0.0;
                        }}
                    }} else {{
                        float period = typeData.dashSolid + typeData.dashTransparent;
                        float offset = mod(proj, period);
                        if (offset > typeData.dashSolid)
                            alpha = 0.0;
                    }}

                    outColor = vec4(typeData.color, alpha);
                }}
                "##),
        )
        .unwrap();
        EdgeRenderer {
            vertex_renderer,
            edge_types,
        }
    }

    pub fn set_edges(&mut self, context: &WebGl2RenderingContext, edges: &Vec<Edge>) {
        fn map<const LEN: usize>(
            edges: &Vec<Segment>,
            map: impl Fn(&Segment) -> [f32; LEN],
        ) -> Box<[f32]> {
            edges
                .iter()
                .flat_map(|edge| map(edge).repeat(6))
                // .flat_map(|node| [0.0, 0.0].repeat(4))
                .collect()
        }

        let segments = edges
            .iter()
            .flat_map(|edge| {
                let mut points = edge.points.clone();
                let edge_type = edge.edge_type;
                let curve_offset = edge.shift;
                points.push(edge.end);
                let edge_segments = points
                    .iter()
                    .scan(edge.start, |prev, item| {
                        let out = (*prev, *item, edge_type as f32, curve_offset);
                        *prev = *item;
                        Some(out)
                    })
                    .collect::<Vec<Segment>>();
                edge_segments
            })
            .collect::<Vec<Segment>>();

        // Start
        let old_starts = map(&segments, |(start, _, _, _)| [start.old.x, start.old.y]);
        self.vertex_renderer
            .set_data(context, "startOld", &old_starts, 2);

        let starts = map(&segments, |(start, _, _, _)| [start.new.x, start.new.y]);
        self.vertex_renderer.set_data(context, "start", &starts, 2);

        let start_old_times = map(&segments, |(start, _, _, _)| [start.old_time as f32]);
        self.vertex_renderer
            .set_data(context, "startStartTime", &start_old_times, 1);

        let start_durations = map(&segments, |(start, _, _, _)| [start.duration as f32]);
        self.vertex_renderer
            .set_data(context, "startDuration", &start_durations, 1);

        // End
        let old_ends = map(&segments, |(_, end, _, _)| [end.old.x, end.old.y]);
        self.vertex_renderer
            .set_data(context, "endOld", &old_ends, 2);

        let ends = map(&segments, |(_, end, _, _)| [end.new.x, end.new.y]);
        self.vertex_renderer.set_data(context, "end", &ends, 2);

        let end_old_times = map(&segments, |(_, end, _, _)| [end.old_time as f32]);
        self.vertex_renderer
            .set_data(context, "endStartTime", &end_old_times, 1);

        let end_durations = map(&segments, |(_, end, _, _)| [end.duration as f32]);
        self.vertex_renderer
            .set_data(context, "endDuration", &end_durations, 1);

        // Curve
        let old_curve_offsets = map(&segments, |(_, _, _, curve_offset)| [curve_offset.old]);
        self.vertex_renderer
            .set_data(context, "curveOffsetOld", &old_curve_offsets, 1);

        let curve_offsets = map(&segments, |(_, _, _, curve_offset)| [curve_offset.new]);
        self.vertex_renderer
            .set_data(context, "curveOffset", &curve_offsets, 1);

        let curve_offset_old_times = map(&segments, |(_, _, _, curve_offset)| {
            [curve_offset.old_time as f32]
        });
        self.vertex_renderer
            .set_data(context, "curveOffsetStartTime", &curve_offset_old_times, 1);

        let curve_offset_durations = map(&segments, |(_, _, _, curve_offset)| {
            [curve_offset.duration as f32]
        });
        self.vertex_renderer
            .set_data(context, "curveOffsetDuration", &curve_offset_durations, 1);

        // Type
        let edge_types = map(&segments, |&(_, _, edge_type, _)| [edge_type]);
        self.vertex_renderer
            .set_data(context, "type", &edge_types, 1);

        self.vertex_renderer.update_data(context);
    }

    pub fn set_transform(&mut self, context: &WebGl2RenderingContext, transform: &Matrix4) {
        self.vertex_renderer.set_uniform(context, "transform", |u| {
            context.uniform_matrix4fv_with_f32_array(u, true, &transform.0)
        });
    }

    pub fn render(
        &mut self,
        context: &WebGl2RenderingContext,
        time: u32,
        selected_ids: &[u32],
        hovered_ids: &[u32],
    ) {
        self.vertex_renderer
            .set_uniform(context, "time", |u| context.uniform1f(u, time as f32));
        for (index, edge_type) in self.edge_types.iter().enumerate() {
            let c = edge_type.color;
            self.vertex_renderer
                .set_uniform(context, &format!("edgeTypes[{index}].color"), |u| {
                    context.uniform3f(u, c.0, c.1, c.2)
                });
            self.vertex_renderer
                .set_uniform(context, &format!("edgeTypes[{index}].width"), |u| {
                    context.uniform1f(u, edge_type.width)
                });
            self.vertex_renderer.set_uniform(
                context,
                &format!("edgeTypes[{index}].dashSolid"),
                |u| context.uniform1f(u, edge_type.dash_solid),
            );
            self.vertex_renderer.set_uniform(
                context,
                &format!("edgeTypes[{index}].dashTransparent"),
                |u| context.uniform1f(u, edge_type.dash_transparent),
            );
        }

        self.vertex_renderer
            .render(context, WebGl2RenderingContext::TRIANGLES);
    }

    pub fn dispose(&self, context: &WebGl2RenderingContext) {
        self.vertex_renderer.dispose(context);
    }
}
