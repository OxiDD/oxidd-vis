#version 300 es
precision highp float;

in vec2 position;
in vec2 positionOld;
in float positionStartTime;
in float positionDuration;

in vec2 charCoord;
out vec2 charCoordOut;

uniform mat4 transform;
uniform float time;

void main() {
    float positionPer = min((time - positionStartTime) / positionDuration, 1.0f);
    vec2 curPosition = positionPer * position + (1.0f - positionPer) * positionOld;
    charCoordOut = charCoord;

    gl_Position = transform * vec4(curPosition, 0.0f, 1.0f) * vec4(vec3(2.0f), 1.0f); // 2 to to make the default width and height of the screen 1, instead of 2
}