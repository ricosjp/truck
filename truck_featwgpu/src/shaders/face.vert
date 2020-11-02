#version 450
#define BUF_SIZE 32
#define EPSILON 1.0e-7

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv_coord;
layout(location = 2) in vec3 normal;

layout(set = 0, binding = 0) uniform Camera {
    mat4 _camera_matrix;
    mat4 camera_projection;
};

layout(location = 0) out vec3 vertex_position;
layout(location = 1) out vec2 uv;
layout(location = 2) out vec3 vertex_normal;

void main() {
    gl_Position = camera_projection * vec4(position, 1.0);
    vertex_position = position;
    uv = uv_coord;
    vertex_normal = normal;
}
