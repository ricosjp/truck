#version 450

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;

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

layout(set = 0, binding = 2) uniform ModelStatus {
    mat4 _matrix;
    vec4 material;
    vec3 reflect_ratio;
};

layout(location = 0) out vec4 color;

float radiance() {
    if (light_type == 0) {
        vec3 dir = light_position - vertex_position;
        return light_strength / dot(dir, dir);
    } else {
        return light_strength;
    }
}

float ambient() {
    return reflect_ratio[0];
}

float diffuse() {
    vec3 normal = normalize(vertex_normal);
    vec3 dir;
    if (light_type == 0) {
        dir = normalize(light_position - vertex_position);
    } else {
        dir = light_position;
    }
    return clamp(dot(dir, normal), 0.0, 1.0) * reflect_ratio[1];
}

float specular() {
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
    return pow(cos_alpha, 1) * reflect_ratio[2];
}

void main() {
    float strength = radiance() * (ambient() + diffuse()) + specular();
    strength = clamp(strength, 0.0, 1.0);
    color = vec4(light_color, 1.0) * material * strength;
}
