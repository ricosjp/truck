#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 color;

void main() {
    float r = length(uv) / sqrt(2.0);
    float l = 1.0 - r;
    vec3 col0 = vec3(r, r * r, r * r * r);
    vec3 col1 = vec3(l * l * l, l, l * l);
    color = vec4(clamp(col0 + col1, 0.0, 1.0), 1.0);
}
