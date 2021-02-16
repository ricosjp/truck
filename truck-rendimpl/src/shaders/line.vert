#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Camera {
    mat4 camera_matrix;
    mat4 camera_projection;
};

layout(set = 1, binding = 0) uniform ModelMatrix {
    mat4 matrix;
};

void main() {
    vec4 world_position = matrix * vec4(position, 1.0);
    gl_Position = camera_projection * world_position;
}