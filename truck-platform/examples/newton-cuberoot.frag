const vec3 COLORS[3] = vec3[](
    vec3(174.0, 37.0, 137.0) / 255.0,
    vec3(0.0, 175.0, 132.0) / 255.0,
    vec3(209.0, 86.0, 36.0) / 255.0
);

const vec2 ONE = vec2(1.0, 0.0);

const vec2 ROOTS[3] = vec2[](
    ONE,
    vec2(1.0, sqrt(3.0)) / 2.0,
    vec2(1.0, -sqrt(3.0)) / 2.0
);

vec2 cmult(vec2 z, vec2 w) {
    return vec2(
        z.x * w.x - z.y * w.y,
        z.x * w.y + z.y * w.x
    );
}

vec2 cinv(vec2 z) {
    return vec2(z.x, -z.y) / dot(z, z);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    float asp = iResolution.x / iResolution.y;
    vec2 uv = (2.0 * fragCoord / iResolution.y - vec2(asp, 1.0)) * 1.2;
    if (length(uv) < 1.0e-6) {
        fragColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }

    vec2 z = uv;
    for (int i = 0; i < 20; i++) {
        vec2 z2 = cmult(z, z);
        vec2 z3 = cmult(z2, z);
        z = z - cmult(z3 - ONE, cinv(3.0 * z2));
    }

    int idx = length(z - ROOTS[0]) < length(z - ROOTS[1]) ? 0 : 1;
    idx = length(z - ROOTS[idx]) < length(z - ROOTS[2]) ? idx : 2;
    fragColor = vec4(COLORS[idx], 1.0);
}
