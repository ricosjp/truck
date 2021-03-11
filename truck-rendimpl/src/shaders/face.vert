#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv_coord;
layout(location = 2) in vec3 normal;
layout(location = 3) in uvec2 brange;

layout(set = 0, binding = 0) uniform Camera {
    mat4 camera_matrix;
    mat4 camera_projection;
};

layout(set = 1, binding = 0) uniform ModelMatrix {
    mat4 matrix;
};

layout(location = 0) out vec3 vertex_position;
layout(location = 1) out vec2 uv;
layout(location = 2) out vec3 vertex_normal;
layout(location = 3) out uvec2 boundary_range;

void main() {
    vec4 world_position = matrix * vec4(position, 1.0);
    vec4 world_normal = normalize(matrix * vec4(normal, 0.0));
    gl_Position = camera_projection * world_position;
    vertex_position = world_position.xyz;
    uv = uv_coord;
    vertex_normal = world_normal.xyz;
    boundary_range = brange;
}