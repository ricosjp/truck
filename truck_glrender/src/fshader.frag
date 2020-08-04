#version 400

in vec3 vertex_position;
in vec3 vertex_normal;

uniform mat4 camera_matrix;
uniform vec3 light_position;
uniform float light_strength;
uniform vec3 reflect_ratio;
uniform vec3 material;
uniform int light_type;

out vec3 color;

float radiance() {
    if (light_type == 0) {
        vec3 tmp = light_position - vertex_position;
        return light_strength / dot(tmp, tmp);
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
    color = material * radiance() * (ambient() + diffuse() + specular());
}
