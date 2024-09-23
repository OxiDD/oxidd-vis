#version 300 es
precision highp float;

in float curType;
in float curExists;

uniform vec4 color1;
uniform vec4 color2;

out vec4 outColor;

void main() {
    outColor = vec4(sqrt(mix(color1.rgb * color1.rgb, color2.rgb * color2.rgb, curType)), curExists * mix(color1.a, color2.a, curType));
}