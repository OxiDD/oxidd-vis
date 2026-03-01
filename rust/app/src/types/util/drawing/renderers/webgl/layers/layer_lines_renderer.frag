#version 300 es
precision highp float;

in float curExists;

uniform vec4 color;

out vec4 outColor;

void main() {
    outColor = vec4(vec3(1.0), pow(curExists, 0.2)) * color;
}