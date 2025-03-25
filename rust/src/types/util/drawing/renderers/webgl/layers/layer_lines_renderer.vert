#version 300 es
precision highp float;

in float yPosition;

in float exists;
in float existsOld;
in float existsStartTime;
in float existsDuration;

uniform mat4 transform;
uniform float time;

out float curExists;

void main() {
    float existsPer = min((time - existsStartTime) / existsDuration, 1.0f);
    curExists = existsPer * exists + (1.0f - existsPer) * existsOld;

    float side = gl_VertexID % 2 == 0 ? -1.f : 1.f;

    float transformedYPos = (transform * vec4(0.0f, yPosition, 0.0f, 1.0f) *
        vec4(vec3(2.0f), 1.0f)).y;
    gl_Position = vec4(side, transformedYPos, 0.0f, 1.0f);
}