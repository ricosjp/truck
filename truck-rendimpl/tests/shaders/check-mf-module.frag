layout(location = 0) out vec4 color;

const float EPS = 1.0e-5;

bool light_direction_test() {
    vec3 position = vec3(1.0, 0.0, 0.0);
    Light light;
    vec3 result, answer;

    // point light
    light.position = vec4(-1.0, 0.0, 1.0, 1.0);
    light.light_type = uvec4(0);
    result = light_direction(light, position);
    answer = vec3(-2.0, 0.0, 1.0) / sqrt(5.0);
    if (distance(result, answer) > EPS) return false;

    // uniform light
    light.light_type[0] = 1;
    result = light_direction(light, position);
    answer = vec3(-1.0, 0.0, 1.0);
    if (distance(result, answer) > EPS) return false;
    return true;
}

bool irradiance_test() {
    vec3 position = vec3(1.0, 0.0, 0.0);
    vec3 normal = vec3(0.0, 0.0, 1.0);
    Light light;
    vec3 result, answer;
    
    // point light
    light.position = vec4(-1.0, 0.0, 1.0, 1.0);
    light.color = vec4(0.01, 0.1, 1.0, 1.0);
    light.light_type = uvec4(0);
    result = irradiance(light, position, normal);
    answer = vec3(0.01, 0.1, 1.0) / sqrt(5.0);
    if (distance(result, answer) > EPS) return false;

    // uniform light
    light.position /= sqrt(2.0);
    light.light_type[0] = 1;
    result = irradiance(light, position, normal);
    answer = vec3(0.01, 0.1, 1.0) / sqrt(2.0);
    if (distance(result, answer) > EPS) return false;

    // from backside
    light.position = vec4(-1.0, 0.0, -1.0, 1.0);
    light.light_type = uvec4(0);
    result = irradiance(light, position, normal);
    answer = vec3(0.0);
    if (distance(result, answer) > EPS) return false;
    return true;
}

bool diffuse_brdf_test() {
    Material material;
    material.albedo = vec4(0.1, 0.2, 0.3, 1.0);
    material.reflectance = 0.8;
    vec3 result = diffuse_brdf(material);
    vec3 answer = vec3(0.02, 0.04, 0.06);
    return distance(result, answer) < EPS;
}

bool microfacet_distribution_test() {
    vec3 middle = vec3(0.5, 0.0, sqrt(3.0) / 2.0);
    vec3 normal = vec3(0.0, 0.0, 1.0);
    float alpha = 0.25;
    float result = microfacet_distribution(middle, normal, alpha);
    float answer = pow(16.0 / 19.0, 2.0);
    return abs(result - answer) < EPS;
}

bool schlick_approxy_test() {
    vec3 v = vec3(0.5, 0.0, sqrt(3.0) / 2.0);
    vec3 normal = vec3(0.0, 0.0, 1.0);
    float k = 0.25;
    float result = schlick_approxy(v, normal, k);
    float answer = 12.0 / (9.0 + 2.0 * sqrt(3.0));
    return abs(result - answer) < EPS;
}

bool geometric_decay_test() {
    vec3 light_dir = normalize(vec3(-1.0, 0.0, 1.0));
    vec3 camera_dir = normalize(vec3(1.0, 0.0, 1.0));
    vec3 normal = vec3(0.0, 0.0, 1.0);
    float alpha = 0.25;
    float result = geometric_decay(light_dir, camera_dir, normal, alpha);
    float answer = 64.0 / (51.0 + 14.0 * sqrt(2.0));
    return abs(result - answer) < EPS;
}

bool fresnel_test() {
    vec3 middle = vec3(0.0, 0.0, 1.0);
    vec3 camera_dir = vec3(1.0, 1.0, 1.0) / sqrt(3.0);
    vec3 f0 = vec3(0.1, 0.2, 0.3);
    vec3 result = fresnel(f0, middle, camera_dir);
    float c = (44.0 * sqrt(3.0) - 76.0) / (9.0 * sqrt(3.0));
    vec3 answer = f0 + (1.0 - f0) * c;
    return distance(result, answer) < EPS;
}

bool specular_brdf_test() {
    Material material;
    material.albedo = vec4(0.01, 0.1, 1.0, 1.0);
    material.roughness = 0.5;
    material.reflectance = 0.3;
    vec3 camera_dir = vec3(1.0, 0.0, 1.0) / sqrt(2.0);
    vec3 light_dir = vec3(-1.0, 0.0, 1.0) / sqrt(2.0);
    vec3 normal = vec3(0.0, 0.0, 1.0);
    vec3 result = specular_brdf(material, camera_dir, light_dir, normal);
    float a = 64.0 / (51.0 + 14.0 * sqrt(2.0));
    float b = 41.0 * sqrt(2.0) - 50.0;
    float c = 58.0 - 41.0 * sqrt(2.0);
    vec3 answer = a * (vec3(0.003, 0.03, 0.3) * b + c);
    return distance(result, answer) < EPS;
}

bool microfacet_color_test() {
    vec3 position = vec3(0.0, 0.0, 0.0);
    vec3 normal = vec3(0.0, 0.0, 1.0);
    Light light;
    light.position = vec4(-1.0, 0.0, 1.0, 1.0);
    light.color = vec4(0.1, 0.2, 0.3, 1.0);
    light.light_type[0] = 0;
    vec3 camera_dir = vec3(1.0, 0.0, 1.0) / sqrt(2.0);
    Material material;
    material.albedo = vec4(0.01, 0.1, 1.0, 1.0);
    material.roughness = 0.5;
    material.reflectance = 0.3;
    vec3 result = microfacet_color(position, normal, light, camera_dir, material);
    vec3 diffuse = vec3(0.007, 0.07, 0.7);
    float a = 64.0 / (51.0 + 14.0 * sqrt(2.0));
    float b = 41.0 * sqrt(2.0) - 50.0;
    float c = 58.0 - 41.0 * sqrt(2.0);
    vec3 specular = a * (vec3(0.003, 0.03, 0.3) * b + c);
    vec3 irradiance = vec3(0.1, 0.2, 0.3) / sqrt(2.0);
    vec3 answer = (diffuse + specular) * irradiance;
    return distance(result, answer) < EPS;
}

bool ambient_correction_test() {
    vec3 pre_color = vec3(0.1, 0.2, 0.3);
    Material material;
    material.albedo = vec4(0.01, 0.1, 1.0, 1.0);
    material.ambient_ratio = 0.02;
    vec3 result = ambient_correction(pre_color, material);
    vec3 answer = vec3(0.0982, 0.198, 0.314);
    return distance(result, answer) < EPS;
}

void main() {
    if (!light_direction_test()) {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    } else if (!irradiance_test()) {
        color = vec4(0.0, 1.0, 0.0, 1.0);
    } else if (!diffuse_brdf_test()) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else if (!microfacet_distribution_test()) {
        color = vec4(1.0, 1.0, 0.0, 1.0);
    } else if (!schlick_approxy_test()) {
        color = vec4(1.0, 0.0, 1.0, 1.0);
    } else if (!geometric_decay_test()) {
        color = vec4(0.0, 1.0, 1.0, 1.0);
    } else if (!fresnel_test()) {
        color = vec4(0.25, 0.25, 0.25, 1.0);
    } else if (!specular_brdf_test()) {
        color = vec4(0.5, 0.5, 0.5, 1.0);
    } else if (!microfacet_color_test()) {
        color = vec4(0.75, 0.75, 0.75, 1.0);
    } else if (!ambient_correction_test()) {
        color = vec4(1.0, 1.0, 1.0, 1.0);
    } else {
        color = vec4(0.2, 0.4, 0.6, 0.8);
    }
}
