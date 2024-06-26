#version 300 es
precision highp float;

uniform sampler2D characters;
in vec2 charCoordOut;
in float curExists;

uniform vec3 color;

out vec4 outColor;
void main() {
    outColor = vec4(color, texture(characters, charCoordOut).a) * curExists;
    // outColor += (1.0-outColor.a) * vec4(charCoord, 1.0, 1.0);
    // outColor = vec4(1.0, 0.0, 1.0, 1.0);
}