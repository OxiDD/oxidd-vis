#version 300 es
precision highp float;

in float yPosition;
in float yPositionOld;
in vec2 yPositionTransition;

in float type;
in float typeOld;
in vec2 typeTransition;

in float exists;
in float existsOld;
in vec2 existsTransition;

uniform mat4 transform;
uniform float time;

out float curType;
out float curExists;

float getPer(vec2 transition) {
    return max(0.0f, min((time - transition.x) / transition.y, 1.0f));
}

void main() {
    float positionPer = getPer(yPositionTransition);
    float curYPosition = positionPer * yPosition + (1.0f - positionPer) * yPositionOld;

    float typePer = getPer(typeTransition);
    curType = typePer * type + (1.0f - typePer) * typeOld;

    float existsPer = getPer(existsTransition);
    curExists = existsPer * exists + (1.0f - existsPer) * existsOld;

    int corner = gl_VertexID % 6; // two triangles
    vec2 cornerPos = (
    /**/corner == 0 || corner == 3 ?  /**/ vec2(1.f, 1.f)  //
    /**/: corner == 1 ?               /**/ vec2(1.f, -1.f) //
    /**/: corner == 2 || corner == 4 ?/**/ vec2(-1.f, -1.f) //
    /**/:                             /**/ vec2(-1.f, 1.f));

    float transformedYPos = (transform * vec4(0.0f, curYPosition, 0.0f, 1.0f) *
        vec4(vec3(2.0f), 1.0f)).y;
    gl_Position = vec4(cornerPos.x, transformedYPos, -0.1f, 1.0f);
    // gl_Position = vec4(cornerPos, 0.0f, 1.0f);
}