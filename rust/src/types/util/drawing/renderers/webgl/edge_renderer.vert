#version 300 es
struct EdgeType {
    vec3 color;
    vec3 hoverColor;
    vec3 selectColor;
    float width;
    float dashSolid;
    float dashTransparent;
};

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
in float state;
out float outType;
out float outState;

out vec2 curStart;
out vec2 curEnd;
out vec2 outPos;
out float curCurveOffset;
out float radius;
out vec2 center;

uniform EdgeType edgeTypes[/*$type_count {*/1/*}*/];
uniform mat4 transform;
uniform float time;

void main() {
    outType = type;
    outState = state;

    float startPer = min((time - startStartTime) / startDuration, 1.0f);
    curStart = startPer * start + (1.0f - startPer) * startOld;
    float halfWidth = 0.5f * edgeTypes[int(type)].width;

    float endPer = min((time - endStartTime) / endDuration, 1.0f);
    curEnd = endPer * end + (1.0f - endPer) * endOld;

    float curvePer = min((time - curveOffsetStartTime) / curveOffsetDuration, 1.0f);
    curCurveOffset = curvePer * curveOffset + (1.0f - curvePer) * curveOffsetOld;

    vec2 delta = curEnd - curStart;
    vec2 dir = normalize(delta);
    vec2 dirOrth = vec2(-dir.y, dir.x);

    bool p = curCurveOffset > 0.f; // Whether the curvature is to the right
    float halfLength = 0.5f * length(delta);
    float curveWidth = min(abs(curCurveOffset), halfLength);
    float centerDeltaX = ((curveWidth * curveWidth) - (halfLength * halfLength)) / (2.0f * curveWidth);
    center = 0.5f * (curEnd + curStart) + dirOrth * centerDeltaX * (p ? 1.f : -1.f);
    radius = abs(centerDeltaX) + curveWidth;

    int corner = gl_VertexID % 6; // two triangles
    outPos = (corner == 0 ? curStart + (-dir * halfWidth - dirOrth * (halfWidth + (p ? 0.f : curveWidth))) : corner == 1 || corner == 3 ? curStart + (-dir * halfWidth + dirOrth * (halfWidth + (p ? curveWidth : 0.f))) : corner == 2 || corner == 4 ? curEnd + (-dirOrth * (halfWidth + (p ? 0.f : curveWidth))) : curEnd + (+dirOrth * (halfWidth + (p ? curveWidth : 0.f))));
    gl_Position = transform * vec4(outPos, 0.0f, 1.0f) * vec4(vec3(2.0f), 1.0f); // 2 to to make the default width and height of the screen 1, instead of 2
}
