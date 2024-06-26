#version 300 es
precision highp float;

in float yPosition;
in float yPositionOld;
in float positionStartTime;
in float positionDuration;

in float type;
in float typeOld;
in float typeStartTime;
in float typeDuration;

uniform mat4 transform;
uniform float time;

out float curType;

void main() {
    float positionPer = min((time - positionStartTime) / positionDuration, 1.0f);
    float curYPosition = positionPer * yPosition + (1.0f - positionPer) * yPositionOld;

    float typePer = min((time - typeStartTime) / typeDuration, 1.0f);
    curType = typePer * type + (1.0f - typePer) * typeOld;

    int corner = gl_VertexID % 6; // two triangles
    vec2 cornerPos = (
    /**/corner == 0 || corner == 3 ?  /**/ vec2(1.f, 1.f)  //
    /**/: corner == 1 ?               /**/ vec2(1.f, -1.f) //
    /**/: corner == 2 || corner == 4 ?/**/ vec2(-1.f, -1.f) //
    /**/:                             /**/ vec2(-1.f, 1.f));

    float transformedYPos = (transform * vec4(0.0f, curYPosition, 0.0f, 1.0f) *
        vec4(vec3(2.0f), 1.0f)).y;
    gl_Position = vec4(cornerPos.x, transformedYPos, 0.0f, 1.0f);
    // gl_Position = vec4(cornerPos, 0.0f, 1.0f);
}