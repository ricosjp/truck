#version 450

layout(location = 0) in uint idx;
layout(location = 0) out vec2 uv;

const vec2 vertex[4] = vec2[](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-0.8, 1.0),
    vec2(0.8, 1.0)
);

void main() {
    uv = vertex[idx];
    gl_Position = vec4(uv, 0.0, 1.0);
}
