#version 300 es
precision highp float;

in vec2 position;
in vec2 positionOld;
in vec2 positionTransition;

in vec2 size;
in vec2 sizeOld;
in vec2 sizeTransition;

in vec3 color;
in vec3 colorOld;
in vec2 colorTransition;

uniform mat4 transform;
uniform float time;

out vec2 cornerPos;
out vec2 curSize;
out vec3 curColor;

float getPer(vec2 transition) {
    return min((time - transition.x) / transition.y, 1.0f);
}

void main() {
    float positionPer = getPer(positionTransition);
    vec2 curPosition = positionPer * position + (1.0f - positionPer) * positionOld;

    float sizePer = getPer(sizeTransition);
    curSize = sizePer * size + (1.0f - sizePer) * sizeOld;

    float colorPer = getPer(colorTransition);
    curColor = sqrt(mix(colorOld * colorOld, color * color, colorPer));

    int corner = gl_VertexID % 6; // two triangles
    cornerPos = curSize * (
    /**/corner == 0 || corner == 3 ?  /**/ vec2(0.5f, 0.5f)  //
    /**/: corner == 1 ?               /**/ vec2(0.5f, -0.5f) //
    /**/: corner == 2 || corner == 4 ?/**/ vec2(-0.5f, -0.5f) //
    /**/:                             /**/ vec2(-0.5f, 0.5f));
    gl_Position = transform * vec4(curPosition + cornerPos, 0.0f, 1.0f) * vec4(vec3(2.0f), 1.0f); // 2 to to make the default width and height of the screen 1, instead of 2
}