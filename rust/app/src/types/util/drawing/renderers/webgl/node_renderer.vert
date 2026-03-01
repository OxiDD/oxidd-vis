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

in float exists;
in float existsOld;
in vec2 existsTransition;

uniform mat4 transform;
uniform float time;

out vec2 cornerPos;
out vec2 curSize;
out vec3 curColor;
out float curExists;

float getPer(vec2 transition) {
    return max(0.0f, min((time - transition.x) / transition.y, 1.0f));
}

void main() {
    float positionPer = getPer(positionTransition);
    vec2 curPosition = positionPer * position + (1.0f - positionPer) * positionOld;

    float sizePer = getPer(sizeTransition);
    curSize = sizePer * size + (1.0f - sizePer) * sizeOld;

    float colorPer = getPer(colorTransition);
    curColor = sqrt(mix(colorOld * colorOld, color * color, colorPer));

    float existsPer = getPer(existsTransition);
    curExists = mix(existsOld, exists, existsPer);

    int corner = gl_VertexID % 6; // two triangles
    cornerPos = curSize * (
    /**/corner == 0 || corner == 3 ?  /**/ vec2(0.5f, 0.5f)  //
    /**/: corner == 1 ?               /**/ vec2(0.5f, -0.5f) //
    /**/: corner == 2 || corner == 4 ?/**/ vec2(-0.5f, -0.5f) //
    /**/:                             /**/ vec2(-0.5f, 0.5f));
    gl_Position = transform * vec4(curPosition + cornerPos, 0.0f, 1.0f) * vec4(vec3(2.0f), 1.0f); // 2 to to make the default width and height of the screen 1, instead of 2
    gl_Position.z = float(gl_VertexID) * 1e-10f;
}