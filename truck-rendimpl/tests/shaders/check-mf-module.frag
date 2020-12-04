layout(location = 0) out vec4 color;

const float EPS = 1.0e-6;

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
    vec3 alpha = 0.25;
    float result = microfacet_distribution(middle, normal, alpha);
    float answer = pow(16.0 / 19.0, 2.0);
    return abs(result - answer) < EPS;
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
    } else {
        color = vec4(0.2, 0.4, 0.6, 0.8);
    }
}
