const PI: f32 = 3.141592653;
const TAU: f32 = 6.283185307;

fn inverse2(m: mat2x2<f32>) -> mat2x2<f32> {
    let det = determinant(m);
    return mat2x2<f32>(m[1][1] / det, -m[0][1] / det, -m[1][0] / det, m[0][0] / det);
}

fn inverse3(m0: mat3x3<f32>) -> mat3x3<f32> {
    let det = determinant(m0);
    let m = transpose(m0);
    return mat3x3<f32>(
        cross(m[1], m[2]) / det,
        cross(m[2], m[0]) / det,
        cross(m[0], m[1]) / det
    );
}

fn inverse4(m0: mat4x4<f32>) -> mat4x4<f32> {
    let det = determinant(m0);
    let m = transpose(m0);
    return mat4x4<f32>(
        determinant(mat3x3<f32>(m[1].yzw, m[2].yzw, m[3].yzw)) / det,
        -determinant(mat3x3<f32>(m[1].xzw, m[2].xzw, m[3].xzw)) / det,
        determinant(mat3x3<f32>(m[1].xyw, m[2].xyw, m[3].xyw)) / det,
        -determinant(mat3x3<f32>(m[1].xyz, m[2].xyz, m[3].xyz)) / det,
        -determinant(mat3x3<f32>(m[0].yzw, m[2].yzw, m[3].yzw)) / det,
        determinant(mat3x3<f32>(m[0].xzw, m[2].xzw, m[3].xzw)) / det,
        -determinant(mat3x3<f32>(m[0].xyw, m[2].xyw, m[3].xyw)) / det,
        determinant(mat3x3<f32>(m[0].xyz, m[2].xyz, m[3].xyz)) / det,
        determinant(mat3x3<f32>(m[0].yzw, m[1].yzw, m[3].yzw)) / det,
        -determinant(mat3x3<f32>(m[0].xzw, m[1].xzw, m[3].xzw)) / det,
        determinant(mat3x3<f32>(m[0].xyw, m[1].xyw, m[3].xyw)) / det,
        -determinant(mat3x3<f32>(m[0].xyz, m[1].xyz, m[3].xyz)) / det,
        -determinant(mat3x3<f32>(m[0].yzw, m[1].yzw, m[2].yzw)) / det,
        determinant(mat3x3<f32>(m[0].xzw, m[1].xzw, m[2].xzw)) / det,
        -determinant(mat3x3<f32>(m[0].xyw, m[1].xyw, m[2].xyw)) / det,
        determinant(mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz)) / det,
    );
}

fn rotate2D(angle: f32) -> mat2x2<f32> {
    let cs = vec2<f32>(cos(angle), sin(angle));
    return mat2x2<f32>(cs.x, cs.y, -cs.y, cs.x);
}

fn rotate3D(angle: f32, axis: vec3<f32>) -> mat3x3<f32> {
    let a = normalize(axis);
    let s = sin(angle);
    let c = cos(angle);
    let r = 1.0 - c;
    return mat3x3<f32>(
        a.x * a.x * r + c,
        a.y * a.x * r + a.z * s,
        a.z * a.x * r - a.y * s,
        a.x * a.y * r - a.z * s,
        a.y * a.y * r + c,
        a.z * a.y * r + a.x * s,
        a.x * a.z * r + a.y * s,
        a.y * a.z * r - a.x * s,
        a.z * a.z * r + c
    );
}
