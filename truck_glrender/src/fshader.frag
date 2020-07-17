#version 400

in vec3 vertex_position;
in vec3 vertex_normal;

uniform mat4 camera_matrix;
uniform vec3 light_position;
uniform float light_strength;
uniform vec3 reflect_ratio;
uniform vec3 material;

out vec3 color;

float radiance() {
    vec3 tmp = light_position - vertex_position;
    return light_strength / dot(tmp, tmp);     
}

float ambient() {
    return reflect_ratio[0];
}

float diffuse() {
    vec3 normal = normalize(vertex_normal);
    vec3 dir = normalize(light_position - vertex_position);
    float res = dot(dir, normal) * reflect_ratio[1];
    return max(res, 0.0);
}

float specular() {
    vec3 normal = normalize(vertex_normal);
    vec3 light_dir = normalize(vertex_position - light_position);
    vec3 reflect_dir = reflect(light_dir, normal);
    vec3 camera_position = vec3(
        camera_matrix[3][0],
        camera_matrix[3][1],
        camera_matrix[3][2]
    );
    vec3 camera_dir = normalize(camera_position - vertex_position);
    float alpha = dot(camera_dir, reflect_dir);
    float res = pow(alpha, 1) * reflect_ratio[2];
    return max(res, 0.0);
}

void main() {
    color = material * radiance() * (ambient() + diffuse() + specular());
}
