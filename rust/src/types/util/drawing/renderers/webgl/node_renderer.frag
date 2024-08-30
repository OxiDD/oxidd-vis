#version 300 es
precision highp float;

out vec4 outColor;
in vec2 cornerPos;
in vec2 curSize;
in vec3 curColor;
in float curExists;

uniform mat4 transform;
float cornerSize = 0.3;
float fuzziness = 0.003; // A form of anti-aliasing by making the circle border a slight gradient

void main() {
    float alpha = 1.0;
    float cornerSize2 = cornerSize * cornerSize;
    float scaledFuzziness = fuzziness / transform[0][0];

    float xBoundary = curSize.x / 2.0 - cornerSize;
    float yBoundary = curSize.y / 2.0 - cornerSize;
    float absX = abs(cornerPos.x);
    float absY = abs(cornerPos.y);
    if (absX > xBoundary && absY > yBoundary) {
        float dx = xBoundary - absX;
        float dy = yBoundary - absY;
        float distance2 = dx*dx + dy*dy;
        if(distance2 >= cornerSize2)
        //     alpha = 1.0 - max(0.0, (sqrt(distance2) - cornerSize) / scaledFuzziness);
            alpha = 0.0;
    }

    outColor = vec4(curColor, curExists * alpha);
}