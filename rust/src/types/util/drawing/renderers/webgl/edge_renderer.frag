#version 300 es
precision highp float;

#define M_PI 3.1415926535897932384626433832795

struct EdgeType { 
    vec3 color;
    float width;
    float dashSolid;
    float dashTransparent;
};

out vec4 outColor;


in vec2 curStart;
in vec2 curEnd;
in vec2 outPos;

in float outType;
in float curCurveOffset;
in float radius;
in vec2 center;

uniform EdgeType edgeTypes[/*$type_count {*/1/*}*/];
uniform mat4 transform;

float fuzziness = 0.003; // A form of anti-aliasing by making the circle border a slight gradient

// Ensures that the output angle is specified such that it's greater than the reference angle
float getAngle(vec2 point, float refAngle) {{
    vec2 delta = point - center;
    float angle = atan(delta.y, delta.x);
    return mod(angle - refAngle + 2.*M_PI, 2.*M_PI) + refAngle;
}}

void main() {
    EdgeType typeData = edgeTypes[int(outType)];
    float halfWidth = 0.5 * typeData.width;
    float alpha = 1.0;
    float scaledFuzziness = fuzziness / transform[0][0];
    float cor = 0.5 * scaledFuzziness;
    float halfWidthSquared = (halfWidth - cor)*(halfWidth - cor);

    float proj; 
    float projPer;
    bool onLine;
    
    if (abs(curCurveOffset) > 0.0) {
        vec2 centerDelta = outPos - center;
        float dist = length(centerDelta);
        float distDelta = abs(dist - radius);
        
        float startAngle;
        float endAngle;
        float pointAngle;
        if (curCurveOffset > 0.) {
            endAngle = getAngle(curEnd, 0.0);
            startAngle = getAngle(curStart, endAngle);
            pointAngle = getAngle(outPos, endAngle);
        } else {
            startAngle = getAngle(curStart, 0.0);
            endAngle = getAngle(curEnd, startAngle);
            pointAngle = getAngle(outPos, startAngle);
        }

        float arcLength = abs(endAngle - startAngle) * radius;
        proj = abs(pointAngle - startAngle) * radius;

        projPer = proj / arcLength;
        onLine = projPer >= 0.0 && projPer <= 1.0;
        
        // if(startAngle < pointAngle) {{
        //     onLine = false;
        // }}
        if(distDelta > halfWidth) {
            onLine = false;
        }
    } else {
        vec2 line = curEnd - curStart;
        vec2 point = outPos - curStart;

        proj = dot(point, normalize(line));
        projPer = proj / length(line);
        onLine = projPer >= 0.0 && projPer <= 1.0;
    }

    if(!onLine) {
        // Only draw half circle from one side
        if(projPer >= 1.0)
            alpha = 0.0;
        else {
            vec2 delta1 = curStart - outPos;
            vec2 delta2 = curEnd - outPos;
            float distSquared = min(dot(delta1, delta1), dot(delta2, delta2));
            
            if(distSquared >= halfWidthSquared) 
                // alpha = 1.0 - max(0.0, (sqrt(distSquared) - (width - cor)) / scaledFuzziness);
                alpha = 0.0;
        }
    } else {
        float period = typeData.dashSolid + typeData.dashTransparent;
        float offset = mod(proj, period);
        if (offset > typeData.dashSolid)
            alpha = 0.0;
    }

    outColor = vec4(typeData.color, alpha);
}