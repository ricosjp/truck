let EPS: f32 = 1.0e-5;

fn light_direction_test() -> bool {
    let position = vec3<f32>(1.0, 0.0, 0.0);
    var light: Light;
    var result: vec3<f32>;
    var answer: vec3<f32>;

    // point light
    light.position = vec4<f32>(-1.0, 0.0, 1.0, 1.0);
    light.light_type = vec4<u32>(0u);
    result = light_direction(light, position);
    answer = vec3<f32>(-2.0, 0.0, 1.0) / sqrt(5.0);
    if (distance(result, answer) > EPS) {
        return false;
    }

    // uniform light
    light.light_type[0] = 1u;
    result = light_direction(light, position);
    answer = vec3<f32>(-1.0, 0.0, 1.0);
    if (distance(result, answer) > EPS) {
        return false;
    }
    return true;
}

fn irradiance_test() -> bool {
    let position = vec3<f32>(1.0, 0.0, 0.0);
    let normal = vec3<f32>(0.0, 0.0, 1.0);
    var light: Light;
    var result: vec3<f32>;
    var answer: vec3<f32>;
    
    // point light
    light.position = vec4<f32>(-1.0, 0.0, 1.0, 1.0);
    light.color = vec4<f32>(0.01, 0.1, 1.0, 1.0);
    light.light_type = vec4<u32>(0u);
    result = irradiance(light, position, normal);
    answer = vec3<f32>(0.01, 0.1, 1.0) / sqrt(5.0);
    if (distance(result, answer) > EPS) {
        return false;
    }

    // uniform light
    light.position = light.position / sqrt(2.0);
    light.light_type[0] = 1u;
    result = irradiance(light, position, normal);
    answer = vec3<f32>(0.01, 0.1, 1.0) / sqrt(2.0);
    if (distance(result, answer) > EPS) {
        return false;
    }

    // from backside
    light.position = vec4<f32>(-1.0, 0.0, -1.0, 1.0);
    light.light_type = vec4<u32>(0u);
    result = irradiance(light, position, normal);
    answer = vec3<f32>(0.0, 0.0, 0.0);
    if (distance(result, answer) > EPS) {
        return false;
    }
    return true;
}

fn diffuse_brdf_test() -> bool {
    var material: Material;
    material.albedo = vec4<f32>(0.1, 0.2, 0.3, 1.0);
    material.reflectance = 0.8;
    let result = diffuse_brdf(material);
    let answer = vec3<f32>(0.02, 0.04, 0.06);
    return distance(result, answer) < EPS;
}

fn microfacet_distribution_test() -> bool {
    let middle = vec3<f32>(0.5, 0.0, sqrt(3.0) / 2.0);
    let normal = vec3<f32>(0.0, 0.0, 1.0);
    let alpha = 0.25;
    let result = microfacet_distribution(middle, normal, alpha);
    let answer = pow(16.0 / 19.0, 2.0);
    return abs(result - answer) < EPS;
}

fn schlick_approxy_test() -> bool {
    let v = vec3<f32>(0.5, 0.0, sqrt(3.0) / 2.0);
    let normal = vec3<f32>(0.0, 0.0, 1.0);
    let k = 0.25;
    let result = schlick_approxy(v, normal, k);
    let answer = 12.0 / (9.0 + 2.0 * sqrt(3.0));
    return abs(result - answer) < EPS;
}

fn geometric_decay_test() -> bool {
    let light_dir = normalize(vec3<f32>(-1.0, 0.0, 1.0));
    let camera_dir = normalize(vec3<f32>(1.0, 0.0, 1.0));
    let normal = vec3<f32>(0.0, 0.0, 1.0);
    let alpha = 0.25;
    let result = geometric_decay(light_dir, camera_dir, normal, alpha);
    let answer = 64.0 / (51.0 + 14.0 * sqrt(2.0));
    return abs(result - answer) < EPS;
}

fn fresnel_test() -> bool {
    let middle = vec3<f32>(0.0, 0.0, 1.0);
    let camera_dir = vec3<f32>(1.0, 1.0, 1.0) / sqrt(3.0);
    let f0 = vec3<f32>(0.1, 0.2, 0.3);
    let result = fresnel(f0, middle, camera_dir);
    let c = (44.0 * sqrt(3.0) - 76.0) / (9.0 * sqrt(3.0));
    let answer = f0 + (1.0 - f0) * c;
    return distance(result, answer) < EPS;
}

fn specular_brdf_test() -> bool {
    var material: Material;
    material.albedo = vec4<f32>(0.01, 0.1, 1.0, 1.0);
    material.roughness = 0.5;
    material.reflectance = 0.3;
    let camera_dir = vec3<f32>(1.0, 0.0, 1.0) / sqrt(2.0);
    let light_dir = vec3<f32>(-1.0, 0.0, 1.0) / sqrt(2.0);
    let normal = vec3<f32>(0.0, 0.0, 1.0);
    let result = specular_brdf(material, camera_dir, light_dir, normal);
    let a = 64.0 / (51.0 + 14.0 * sqrt(2.0));
    let b = 41.0 * sqrt(2.0) - 50.0;
    let c = 58.0 - 41.0 * sqrt(2.0);
    let answer = a * (vec3<f32>(0.003, 0.03, 0.3) * b + c);
    return distance(result, answer) < EPS;
}

fn microfacet_color_test() -> bool {
    let position = vec3<f32>(0.0, 0.0, 0.0);
    let normal = vec3<f32>(0.0, 0.0, 1.0);
    var light: Light;
    light.position = vec4<f32>(-1.0, 0.0, 1.0, 1.0);
    light.color = vec4<f32>(0.1, 0.2, 0.3, 1.0);
    light.light_type[0] = 0u;
    let camera_dir = vec3<f32>(1.0, 0.0, 1.0) / sqrt(2.0);
    var material: Material;
    material.albedo = vec4<f32>(0.01, 0.1, 1.0, 1.0);
    material.roughness = 0.5;
    material.reflectance = 0.3;
    let result = microfacet_color(position, normal, light, camera_dir, material);
    let diffuse = vec3<f32>(0.007, 0.07, 0.7);
    let a = 64.0 / (51.0 + 14.0 * sqrt(2.0));
    let b = 41.0 * sqrt(2.0) - 50.0;
    let c = 58.0 - 41.0 * sqrt(2.0);
    let specular = a * (vec3<f32>(0.003, 0.03, 0.3) * b + c);
    let irr = vec3<f32>(0.1, 0.2, 0.3) / sqrt(2.0);
    let answer = (diffuse + specular) * irr;
    return distance(result, answer) < EPS;
}

fn ambient_correction_test() -> bool {
    let pre_color = vec3<f32>(0.1, 0.2, 0.3);
    var material: Material;
    material.albedo = vec4<f32>(0.01, 0.1, 1.0, 1.0);
    material.ambient_ratio = 0.02;
    let result = ambient_correction(pre_color, material);
    let answer = vec3<f32>(0.0982, 0.198, 0.314);
    return distance(result, answer) < EPS;
}

[[stage(vertex)]]
fn vs_main([[location(0)]] idx: u32) -> [[builtin(position)]] vec4<f32> {
    var vertex: array<vec2<f32>, 4>;
    vertex[0] = vec2<f32>(-1.0, -1.0);
    vertex[1] = vec2<f32>(1.0, -1.0);
    vertex[2] = vec2<f32>(-1.0, 1.0);
    vertex[3] = vec2<f32>(1.0, 1.0);

    return vec4<f32>(vertex[idx], 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    if (!light_direction_test()) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    } else if (!irradiance_test()) {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    } else if (!diffuse_brdf_test()) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    } else if (!microfacet_distribution_test()) {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0);
    } else if (!schlick_approxy_test()) {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    } else if (!geometric_decay_test()) {
        return vec4<f32>(0.0, 1.0, 1.0, 1.0);
    } else if (!fresnel_test()) {
        return vec4<f32>(0.25, 0.25, 0.25, 1.0);
    } else if (!specular_brdf_test()) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } else if (!microfacet_color_test()) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } else if (!ambient_correction_test()) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else {
        return vec4<f32>(0.2, 0.4, 0.6, 0.8);
    }
}

[[stage(fragment)]]
fn fs_main_anti() -> [[location(0)]] vec4<f32> {
    if (!light_direction_test()) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    } else if (!irradiance_test()) {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
    } else if (!diffuse_brdf_test()) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    } else if (!microfacet_distribution_test()) {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0);
    } else if (!schlick_approxy_test()) {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    } else if (!geometric_decay_test()) {
        return vec4<f32>(0.0, 1.0, 1.0, 1.0);
    } else if (!fresnel_test()) {
        return vec4<f32>(0.25, 0.25, 0.25, 1.0);
    } else if (specular_brdf_test()) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } else if (!microfacet_color_test()) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } else if (!ambient_correction_test()) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else {
        return vec4<f32>(0.2, 0.4, 0.6, 0.8);
    }
}
