#version 300 es
precision highp float;

out vec4 outColor;
in vec2 cornerPos;
in vec2 curSize;
in vec3 curColor;
in float curExists;

uniform float cornerSize;

void main() {
    float alpha = 1.0f;
    float cornerSize2 = cornerSize * cornerSize;

    float xCornerBoundary = curSize.x / 2.0f - cornerSize;
    float yCornerBoundary = curSize.y / 2.0f - cornerSize;
    float absX = abs(cornerPos.x);
    float absY = abs(cornerPos.y);
    if(absX > xCornerBoundary && absY > yCornerBoundary) {
        float dx = xCornerBoundary - absX;
        float dy = yCornerBoundary - absY;
        float distance2 = dx * dx + dy * dy;
        if(distance2 >= cornerSize2)
            alpha = 0.0f;
    }

    float a = max(0.0f, curExists * alpha);
    outColor = vec4(curColor * a, a);
}