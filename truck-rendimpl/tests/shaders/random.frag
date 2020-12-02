#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 color;

// cf: https://thebookofshaders.com/10/
float random(vec2 uv) {
    vec2 c = vec2(12.9898, 78.233) * 43758.5453123;
    return fract(sin(dot(uv, c)));
}

void main() {
    float r = random(uv);
    float g = random(uv.yx);
    float b = random(vec2(r, g));
    color = vec4(r, g, b, 1.0);
}
