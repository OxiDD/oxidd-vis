#version 300 es
precision highp float;

out vec4 outColor;
in vec2 cornerPos;
in vec2 curSize;
in vec4 curColor;
in float curExists;

uniform float cornerSize;
uniform float offset;
uniform float width;

void main() {
    float outerScale = (1.0f - offset * 2.0f);
    float innerScale = (1.0f - (offset + width) * 2.0f);

    float outerCornerSize = outerScale * cornerSize;
    float outerCornerSize2 = outerCornerSize * outerCornerSize;
    float innerCornerSize = innerScale * cornerSize;
    float innerCornerSize2 = innerCornerSize * innerCornerSize;

    float absX = abs(cornerPos.x);
    float absY = abs(cornerPos.y);
    float alpha = 0.0f;

    // outer boundary corner
    float xOuterBoundary = curSize.x / 2.0f - offset;
    float yOuterBoundary = curSize.y / 2.0f - offset;
    if(absX < xOuterBoundary && absY < yOuterBoundary) {
        alpha = 1.0f;
        float xCornerBoundary = curSize.x / 2.0f - outerCornerSize - offset;
        float yCornerBoundary = curSize.y / 2.0f - outerCornerSize - offset;

        if(absX > xCornerBoundary && absY > yCornerBoundary) {
            float dx = xCornerBoundary - absX;
            float dy = yCornerBoundary - absY;
            float distance2 = dx * dx + dy * dy;
            if(distance2 >= outerCornerSize2)
                alpha = 0.0f;
        }
    }

    // inner boundary
    float xInnerBoundary = curSize.x / 2.0f - width - offset;
    float yInnerBoundary = curSize.y / 2.0f - width - offset;
    if(absX < xInnerBoundary && absY < yInnerBoundary) {
        alpha = 0.0f;

        float xInnerCornerBoundary = curSize.x / 2.0f - width - innerCornerSize - offset;
        float yInnerCornerBoundary = curSize.y / 2.0f - width - innerCornerSize - offset;
        if(absX > xInnerCornerBoundary && absY > yInnerCornerBoundary) {
            float dx = xInnerCornerBoundary - absX;
            float dy = yInnerCornerBoundary - absY;
            float distance2 = dx * dx + dy * dy;
            if(distance2 >= innerCornerSize2)
                alpha = 1.0f;
        }
    }

    float a = max(0.0f, curColor.a * curExists * alpha);
    outColor = vec4(curColor.rgb * a, a);
}
