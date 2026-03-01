#version 300 es
precision highp float;

out vec4 outColor;
in vec2 pos;

void main() {
    // outColor = vec4(0, 0, 0, 1.0);
    outColor = vec4(0.5*(pos)+vec2(0.5), 0, 1.0);
    // outColor = vec4(0.0, 1.0, 0.0, 1.0);
}