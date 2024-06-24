#version 300 es
precision highp float;

uniform sampler2D characters;
in vec2 charCoordOut;

out vec4 outColor;
void main() {
    outColor = texture(characters, charCoordOut);
    // outColor += (1.0-outColor.a) * vec4(charCoord, 1.0, 1.0);
    // outColor = vec4(1.0, 0.0, 1.0, 1.0);
}