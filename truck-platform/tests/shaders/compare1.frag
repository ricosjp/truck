#version 450

layout(location = 0) in vec2 st;
layout(location = 0) out vec4 color;

void main() {
    vec2 uv = (st + 1.0) / 2.0;
    color = vec4(
        uv.x,
        clamp(1.0 - distance(vec2(0.5, 1.0), uv), 0.0, 1.0),
        1.0 - uv.x,
        1.0
    );
    if (length(st) < 0.01) color = vec4(1.0, 1.0, 1.0, 1.0);
}
