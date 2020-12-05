// Based on the microfacet theory
// cf: https://qiita.com/mebiusbox2/items/e7063c5dfe1424e0d01a

struct Light {
    vec4 position;
    vec4 color;
    uvec4 light_type;
};

struct Material {
    vec4 albedo;
    float roughness;
    float reflectance;
    float ambient_ratio;
};

// light direction from point to light
vec3 light_direction(Light light, vec3 position) {
    switch(light.light_type[0]) {
    case 0:
        return normalize(light.position.xyz - position);
    default:
        return light.position.xyz;
    }
}

vec3 irradiance(Light light, vec3 position, vec3 normal) {
    vec3 light_dir = light_direction(light, position);
    return light.color.xyz * clamp(dot(light_dir, normal), 0.0, 1.0);
}

vec3 diffuse_brdf(Material material) {
    return material.albedo.xyz * (1.0 - material.reflectance);
}

float microfacet_distribution(vec3 middle, vec3 normal, float alpha) {
    float dotNH = dot(normal, middle);
    float alpha2 = alpha * alpha;
    float sqrt_denom = 1.0 - dotNH * dotNH * (1.0 - alpha2);
    return alpha2 / (sqrt_denom * sqrt_denom);
}

float schlick_approxy(vec3 vec, vec3 normal, float k) {
    float dotNV = dot(normal, vec);
    return dotNV / (dotNV * (1.0 - k) + k);
}

float geometric_decay(vec3 light_dir, vec3 camera_dir, vec3 normal, float alpha) {
    float k = alpha / 2.0;
    return schlick_approxy(light_dir, normal, k) * schlick_approxy(camera_dir, normal, k);
}

vec3 fresnel(vec3 f0, vec3 middle, vec3 camera_dir) {
    float c = 1.0 - dot(middle, camera_dir);
    c = c * c * c * c * c;
    return f0 + (1.0 - f0) * c;
}

vec3 specular_brdf(Material material, vec3 camera_dir, vec3 light_dir, vec3 normal) {
    vec3 specular_color = material.albedo.xyz * material.reflectance;
    vec3 middle = normalize(camera_dir + light_dir);
    float alpha = material.roughness * material.roughness;
    float distribution = microfacet_distribution(middle, normal, alpha);
    float decay = geometric_decay(light_dir, camera_dir, normal, alpha);
    vec3 fresnel_color = fresnel(specular_color, middle, camera_dir);
    float dotCN = clamp(dot(camera_dir, normal), 0.0, 1.0);
    float dotLN = clamp(dot(light_dir, normal), 0.0, 1.0);
    float denom = 4.0 * dotCN * dotLN;
    if (abs(denom) < 1.0e-6) {
        return vec3(0.0, 0.0, 0.0);
    }
    return distribution * decay / denom * fresnel_color;
}

vec3 microfacet_color(vec3 position, vec3 normal, Light light, vec3 camera_dir, Material material) {
    vec3 light_dir = light_direction(light, position);
    vec3 irradiance = irradiance(light, position, normal);
    vec3 diffuse = diffuse_brdf(material);
    vec3 specular = specular_brdf(material, camera_dir, light_dir, normal);
    return (diffuse + specular) * irradiance;
}

vec3 ambient_correction(vec3 pre_color, Material material) {
    return pre_color * (1.0 - material.ambient_ratio)
        + material.albedo.xyz * material.ambient_ratio;
}
