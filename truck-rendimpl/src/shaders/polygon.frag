#version 450

#include "microfacet_module.frag"

layout(location = 0) in vec3 position;
layout(location = 2) in vec3 vertex_normal;

layout(set = 0, binding = 0) uniform Camera {
    mat4 camera_matrix;
    mat4 camera_projection;
};

layout(set = 0, binding = 1) buffer Lights {
    Light lights[];
};

layout(set = 0, binding = 2) uniform Scene {
    float _time;
    uint nlights;
};

layout(set = 1, binding = 1) uniform ModelMaterial {
    Material material;
};

layout(location = 0) out vec4 color;

void main() {
    vec3 camera_dir = normalize(camera_matrix[3].xyz - position);
    vec3 normal = normalize(vertex_normal);
    vec3 pre_color = vec3(0.0, 0.0, 0.0);
    for (uint i = 0; i < nlights; i++) {
        pre_color += microfacet_color(position, normal, lights[i], camera_dir, material);
    }
    pre_color = ambient_correction(pre_color, material);
    color = vec4(pre_color, 1.0);
}
