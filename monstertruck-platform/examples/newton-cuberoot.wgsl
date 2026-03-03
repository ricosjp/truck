fn cmult(z: vec2<f32>, w: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        z.x * w.x - z.y * w.y,
        z.x * w.y + z.y * w.x
    );
}

fn cinv(z: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(z.x, -z.y) / dot(z, z);
}

fn main_image(coord: vec2<f32>, env: Environment) -> vec4<f32> {
    var COLORS: array<vec3<f32>, 3>;
    COLORS[0] = vec3<f32>(174.0, 37.0, 137.0) / 255.0;
    COLORS[1] = vec3<f32>(0.0, 175.0, 132.0) / 255.0;
    COLORS[2] = vec3<f32>(209.0, 86.0, 36.0) / 255.0;

    let ONE = vec2<f32>(1.0, 0.0);

    var ROOTS: array<vec2<f32>, 3>;
    ROOTS[0] = ONE;
    ROOTS[1] = vec2<f32>(1.0, sqrt(3.0)) / 2.0;
    ROOTS[2] = vec2<f32>(1.0, -sqrt(3.0)) / 2.0;

    let uv = mat2x2<f32>(
        vec2<f32>(cos(env.time), -sin(env.time)),
        vec2<f32>(sin(env.time), cos(env.time))
    ) * (2.0 * coord - env.resolution) / env.resolution.y;

    if (length(uv) < 1.0e-5) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    var z: vec2<f32> = uv;
    for (var i: u32 = 0u; i < 20u; i = i + 1u) {
        let z2 = cmult(z, z);
        let z3 = cmult(z2, z);
        z = z - cmult(z3 - ONE, cinv(3.0 * z2));
    }

    var idx: u32;
    if (length(z - ROOTS[0]) < length(z - ROOTS[1])) {
        idx = 0u;
    } else {
        idx = 1u;
    }
    if (length(z - ROOTS[idx]) > length(z - ROOTS[2])) {
        idx = 2u;
    }
    return vec4<f32>(COLORS[idx], 1.0);
}
