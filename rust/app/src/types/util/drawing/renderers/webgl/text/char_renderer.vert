#version 300 es
precision highp float;

in vec2 position;
out vec2 pos;

void main() {
    pos = position;
    gl_Position = vec4(2.0f * position - vec2(1.0f), 0.0f, 1.0f);
}