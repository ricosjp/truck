#version 450

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 vertex_normal;

layout(set = 0, binding = 0) uniform Camera {
    mat4 camera_matrix;
    mat4 camera_projection;
};

layout(set = 0, binding = 1) uniform Light {
    vec3 light_position;
    float light_strength;
    vec3 light_color;
    int light_type;
};

layout(set = 1, binding = 1) uniform ModelColor {
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
    vec4 reflect_ratio;
};

layout(set = 1, binding = 2) uniform texture2D texture_view;
layout(set = 1, binding = 3) uniform sampler texture_sampler;

layout(location = 0) out vec4 color;

vec4 textured_ambient() {
    if (uv[0] < 0.0 || 1.0 < uv[0] || uv[1] < 0.0 || 1.0 < uv[1]) return ambient;
    else return texture(sampler2D(texture_view, texture_sampler), uv);
}

vec4 textured_diffuse() {
    if (uv[0] < 0.0 || 1.0 < uv[0] || uv[1] < 0.0 || 1.0 < uv[1]) return diffuse;
    else return texture(sampler2D(texture_view, texture_sampler), uv);
}

float radiance() {
    if (light_type == 0) {
        vec3 dir = light_position - vertex_position;
        return light_strength / dot(dir, dir);
    } else {
        return light_strength;
    }
}

float ambient_strength() {
    return reflect_ratio[0];
}

float diffuse_strength() {
    vec3 normal = normalize(vertex_normal);
    vec3 dir;
    if (light_type == 0) {
        dir = normalize(light_position - vertex_position);
    } else {
        dir = light_position;
    }
    return clamp(dot(dir, normal), 0.0, 1.0) * reflect_ratio[1];
}

float specular_strength() {
    vec3 normal = normalize(vertex_normal);
    vec3 light_dir;
    if (light_type == 0) {
        light_dir = normalize(vertex_position - light_position);
    } else {
        light_dir = -light_position;
    }
    vec3 reflect_dir = reflect(light_dir, normal);
    vec3 camera_position = vec3(
        camera_matrix[3][0],
        camera_matrix[3][1],
        camera_matrix[3][2]
    );
    vec3 camera_dir = normalize(camera_position - vertex_position);
    float cos_alpha = clamp(dot(camera_dir, reflect_dir), 0.0, 1.0);
    return pow(cos_alpha, 5) * reflect_ratio[2];
}

void main() {
    vec4 material = radiance() * (
        ambient_strength() * textured_ambient()
        + diffuse_strength() * textured_diffuse()
        + specular_strength() * specular
        );
    color = vec4(light_color, 1.0) * vec4(material.xyz, 1.0);
}
