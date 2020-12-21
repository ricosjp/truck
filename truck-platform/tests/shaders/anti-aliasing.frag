#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 color;

const vec4 BLACK = vec4(vec3(0.0), 1.0);
const vec4 WHITE = vec4(1.0);

void main() {
    if (uv.y > 2.0 * uv.x + 0.002) color = WHITE;
    else if (uv.y < 2.0 * uv.x - 0.002) color = WHITE;
    else color = BLACK;
}
