#version 400

in vec3 position;
in vec3 normal;
uniform mat4 camera_projection;

out vec3 vertex_position;
out vec3 vertex_normal;

void main() {
    gl_Position = camera_projection * vec4(position, 1.0);
    vertex_position = position;
    vertex_normal = normal;
}