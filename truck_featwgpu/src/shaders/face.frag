#version 450

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 vertex_normal;

layout(set = 0, binding = 0) uniform Camera {
    mat4 camera_matrix;
    mat4 camera_projection;
};

struct Light {
    vec4 position;
    vec4 color;
    uvec4 light_type;
};

layout(set = 0, binding = 1) buffer Lights {
    Light lights[];
};

layout(set = 0, binding = 2) uniform Scene {
    float _time;
    uint nlights;
};

layout(set = 1, binding = 0) buffer Boundary {
    vec4 boundary[];
};

layout(set = 1, binding = 1) uniform BoundaryLength {
    uint boundary_length;
};

layout(location = 0) out vec4 color;

vec4 albedo = vec4(1.0, 1.0, 1.0, 3.0) / 3.0;
float roughness = 0.5;
float reflectance = 0.5;
vec3 normal = normalize(vertex_normal);

bool in_domain() {
    int score = 0;
    for (int i = 0; i < boundary_length; i++) {
        vec2 start = boundary[i].xy - uv;
        vec2 end = boundary[i].zw - uv;
        if (start[1] * end[1] >= 0) continue;
        float as = abs(start[1]);
        float ae = abs(end[1]);
        float x = (ae * start[0] + as * end[0]) / (as + ae);
        if (x > 0) {
            if (end[1] > 0) score += 1;
            else score -= 1;
        }
    }
    return score > 0;
}

vec3 light_direction(vec3 light_position, uint light_type) {
    if (light_type == 0) {
        return normalize(light_position - vertex_position);
    } else {
        return light_position;
    }
}

vec3 light_irradiance(vec3 light_dir, vec3 light_color) {
    return light_color * clamp(dot(light_dir, vertex_normal), 0.0, 1.0);
}

vec3 diffuse_brdf() {
    return albedo.xyz * (1.0 - reflectance);
}

float microfacet_distribution(vec3 middle, float alpha) {
    float dotNH = clamp(dot(normal, middle), 0.0, 1.0);
    float alpha2 = alpha * alpha;
    float sqrt_denom = dotNH * dotNH * (alpha2 - 1.0) + 1.0;
    return alpha2 / (sqrt_denom * sqrt_denom);
}

float schlick_approxy(vec3 vec, float k) {
    float dotNV = clamp(dot(normal, vec), 0.0, 1.0);
    return dotNV / (dotNV * (1.0 - k) + k);
}

float geometric_decay(vec3 light_dir, vec3 camera_dir, float alpha) {
    float k = alpha / 2.0;
    return schlick_approxy(light_dir, k) * schlick_approxy(camera_dir, k);
}

float fresnel(float f0, vec3 middle, vec3 camera_dir) {
    return f0 + (1.0 - f0) * pow(1.0 - clamp(dot(middle, camera_dir), 0.0, 1.0), 5);
}

vec3 specular_brdf(vec3 camera_dir, vec3 light_dir) {
    vec3 specular_color = albedo.xyz * reflectance;
    vec3 middle = normalize((camera_dir + light_dir) / 2.0);
    float alpha = roughness * roughness;
    float distribution = microfacet_distribution(middle, alpha);
    float decay = geometric_decay(light_dir, camera_dir, alpha);
    vec3 fresnel_color = vec3(
        fresnel(specular_color[0], middle, camera_dir),
        fresnel(specular_color[1], middle, camera_dir),
        fresnel(specular_color[2], middle, camera_dir)
    );
    float dotCN = clamp(dot(camera_dir, normal), 0.0, 1.0);
    float dotLN = clamp(dot(light_dir, normal), 0.0, 1.0);
    float denom = 4.0 * dotCN * dotLN;
    if (denom == 0.0) {
        return vec3(0.0, 0.0, 0.0);
    }
    return distribution * decay / denom * fresnel_color;
}

void main() {
    if (!in_domain()) discard;
    vec3 pre_color = vec3(0.0, 0.0, 0.0);
    for (uint i = 0; i < nlights; i++) {
        vec3 light_position = lights[i].position.xyz;
        vec3 light_color = lights[i].color.xyz;
        uint light_type = lights[i].light_type[0];
        vec3 camera_dir = normalize(camera_matrix[3].xyz - vertex_position);
        vec3 light_dir = light_direction(light_position, light_type);
        vec3 irradiance = light_irradiance(light_dir, light_color);
        vec3 diffuse = diffuse_brdf();
        vec3 specular = specular_brdf(camera_dir, light_dir);
        pre_color += (diffuse + specular) * irradiance * 0.98 + albedo.xyz * 0.02;
    }
    color = vec4(pre_color, 1.0);
}
