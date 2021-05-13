[[block]]
struct Camera {
    matrix: mat4x4<f32>;
    projection: mat4x4<f32>;
};

struct Light {
    position: vec4<f32>;
    color: vec4<f32>;
    light_type: vec4<u32>;
};

[[block]]
struct Lights {
    lights: [[stride(48)]] array<Light>;
};

[[block]]
struct SceneInfo {
    time: f32;
    nlights: u32;
};

[[group(0), binding(0)]]
var<uniform> camera: Camera;

[[group(0), binding(1)]]
var<storage> lights: [[access(read)]] Lights;

[[group(0), binding(2)]]
var<uniform> info: SceneInfo;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] cm: mat4x4<f32>;
    [[location(4)]] cp: mat4x4<f32>;
    [[location(8)]] lp0: vec4<f32>;
    [[location(9)]] lc0: vec4<f32>;
    [[location(10)]] lt0: vec4<u32>;
    [[location(11)]] lp1: vec4<f32>;
    [[location(12)]] lc1: vec4<f32>;
    [[location(13)]] lt1: vec4<u32>;
    [[location(14)]] st: f32;
    [[location(15)]] snl: u32;
};

[[stage(vertex)]]
fn vs_main([[location(0)]] idx: u32) -> VertexOutput {
    var vertex: array<vec2<f32>, 4>;
    vertex[0] = vec2<f32>(-1.0, -1.0);
    vertex[1] = vec2<f32>(1.0, -1.0);
    vertex[2] = vec2<f32>(-1.0, 1.0);
    vertex[3] = vec2<f32>(1.0, 1.0);
    
    var output: VertexOutput;
    output.position = vec4<f32>(vertex[idx], 0.0, 1.0);
    output.cm = camera.matrix;
    output.cp = camera.projection;
    output.lp0 = lights.lights[0].position;
    output.lc0 = lights.lights[0].color;
    output.lt0 = lights.lights[0].light_type;
    output.lp1 = lights.lights[1].position;
    output.lc1 = lights.lights[1].color;
    output.lt1 = lights.lights[1].light_type;
    output.st = info.time;
    output.snl = info.nlights;
    return output;
}

let EPS: f32 = 1.0e-5;
let e: vec2<f32> = vec2<f32>(1.0, 0.0);

[[stage(fragment)]]
fn fs_main(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    let acm0 = vec4<f32>(1.0, 2.1, 3.2, 4.3);
    let acm1 = vec4<f32>(5.4, 6.5, 7.6, 8.7);
    let acm2 = vec4<f32>(9.8, 10.9, 11.0, 12.0);
    let acm3 = vec4<f32>(13.0, 14.0, 15.0, 16.23);

    let acp0 = vec4<f32>(11.714964805291158, -20.083602331793195, 0.2296881862103637, 1.0000000000018192);
    let acp1 = vec4<f32>(-7.919279773964541, 12.919469128453738, -1.3377250768579256, -2.0000000000036002);
    let acp2 = vec4<f32>(-23.645933626763625, 36.55672789512988, 1.9863855950847729, 1.0000000000017617);
    let acp3 = vec4<f32>(19.301563694896643, -28.84390979125007, -0.8783487044372066, 0.000000000000025531539193725142);

    let alp0 = vec4<f32>(0.1, 0.2, 0.3, 1.0);
    let alc0 = vec4<f32>(0.4, 0.5, 0.6, 1.0);
    let alt0 = vec4<u32>(0u, 0u, 0u, 0u);
    let alp1 = vec4<f32>(1.1, 1.2, 1.3, 1.0);
    let alc1 = vec4<f32>(1.4, 1.5, 1.6, 1.0);
    let alt1 = vec4<u32>(1u, 0u, 0u, 0u);
    let asnl = 2u;
    
    if (any(input.cm * e.xyyy != camera.matrix * e.xyyy)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (any(input.cm * e.yxyy != camera.matrix * e.yxyy)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (any(input.cm * e.yyxy != camera.matrix * e.yyxy)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (any(input.cm * e.yyyx != camera.matrix * e.yyyx)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(input.cm * e.xyyy, acm0) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(input.cm * e.yxyy, acm1) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(input.cm * e.yyxy, acm2) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(input.cm * e.yyyx, acm3) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (any(input.cp * e.xyyy != camera.projection * e.xyyy)) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (any(input.cp * e.yxyy != camera.projection * e.yxyy)) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (any(input.cp * e.yyxy != camera.projection * e.yyxy)) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (any(input.cp * e.yyyx != camera.projection * e.yyyx)) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (distance(input.cp * e.xyyy, acp0) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (distance(input.cp * e.yxyy, acp1) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (distance(input.cp * e.yyxy, acp2) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (distance(input.cp * e.yyyx, acp3) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (all(input.lp0 != lights.lights[0].position)) {
        return vec4<f32>(0.2, 0.2, 0.2, 1.0);
    } elseif (distance(input.lp0, alp0) > EPS) {
        return vec4<f32>(0.2, 0.2, 0.2, 1.0);
    } elseif (all(input.lc0 != lights.lights[0].color)) {
        return vec4<f32>(0.3, 0.3, 0.3, 1.0);
    } elseif (distance(input.lc0, alc0) > EPS) {
        return vec4<f32>(0.3, 0.3, 0.3, 1.0);
    } elseif (any(input.lt0 != lights.lights[0].light_type)) {
        return vec4<f32>(0.4, 0.4, 0.4, 1.0);
    } elseif (any(input.lt0 != alt0)) {
        return vec4<f32>(0.4, 0.4, 0.4, 1.0);
    } elseif (any(input.lp1 != lights.lights[1].position)) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } elseif (distance(input.lp1, alp1) > EPS) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } elseif (any(input.lc1 != lights.lights[1].color)) {
        return vec4<f32>(0.6, 0.6, 0.6, 1.0);
    } elseif (distance(input.lc1, alc1) > EPS) {
        return vec4<f32>(0.6, 0.6, 0.6, 1.0);
    } elseif (any(input.lt1 != lights.lights[1].light_type)) {
        return vec4<f32>(0.7, 0.7, 0.7, 1.0);
    } elseif (any(input.lt1 != alt1)) {
        return vec4<f32>(0.7, 0.7, 0.7, 1.0);
    } elseif (input.st != info.time) {
        return vec4<f32>(0.8, 0.8, 0.8, 1.0);
    } elseif (input.snl != info.nlights) {
        return vec4<f32>(0.9, 0.9, 0.9, 1.0);
    } elseif (input.snl != asnl) {
        return vec4<f32>(0.9, 0.9, 0.9, 1.0);
    } else {
        return vec4<f32>(0.2, 0.4, 0.6, 0.8);
    }
}

[[stage(fragment)]]
fn fs_main_anti(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    let acm0 = vec4<f32>(1.0, 2.1, 3.2, 4.3);
    let acm1 = vec4<f32>(5.4, 6.5, 7.6, 8.7);
    let acm2 = vec4<f32>(9.8, 10.9, 11.0, 12.0);
    let acm3 = vec4<f32>(13.0, 14.0, 15.0, 16.23);

    let acp0 = vec4<f32>(11.714964805291158, -20.083602331793195, 0.2296881862103637, 1.0000000000018192);
    let acp1 = vec4<f32>(-7.919279773964541, 12.919469128453738, -1.3377250768579256, -2.0000000000036002);
    let acp2 = vec4<f32>(-23.645933626763625, 36.55672789512988, 1.9863855950847729, 1.0000000000017617);
    let acp3 = vec4<f32>(19.301563694896643, -28.84390979125007, -0.8783487044372066, 0.000000000000025531539193725142);

    let alp0 = vec4<f32>(0.1, 0.2, 0.3, 1.0);
    let alc0 = vec4<f32>(0.4, 0.6, 0.6, 1.0);
    let alt0 = vec4<u32>(0u, 0u, 0u, 0u);
    let alp1 = vec4<f32>(1.1, 1.2, 1.3, 1.0);
    let alc1 = vec4<f32>(1.4, 1.5, 1.6, 1.0);
    let alt1 = vec4<u32>(1u, 0u, 0u, 0u);
    let asnl = 2u;
    
    if (any(input.cm * e.xyyy != camera.matrix * e.xyyy)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (any(input.cm * e.yxyy != camera.matrix * e.yxyy)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (any(input.cm * e.yyxy != camera.matrix * e.yyxy)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (any(input.cm * e.yyyx != camera.matrix * e.yyyx)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(input.cm * e.xyyy, acm0) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(input.cm * e.yxyy, acm1) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(input.cm * e.yyxy, acm2) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(input.cm * e.yyyx, acm3) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (any(input.cp * e.xyyy != camera.projection * e.xyyy)) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (any(input.cp * e.yxyy != camera.projection * e.yxyy)) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (any(input.cp * e.yyxy != camera.projection * e.yyxy)) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (any(input.cp * e.yyyx != camera.projection * e.yyyx)) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (distance(input.cp * e.xyyy, acp0) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (distance(input.cp * e.yxyy, acp1) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (distance(input.cp * e.yyxy, acp2) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (distance(input.cp * e.yyyx, acp3) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } elseif (all(input.lp0 != lights.lights[0].position)) {
        return vec4<f32>(0.2, 0.2, 0.2, 1.0);
    } elseif (distance(input.lp0, alp0) > EPS) {
        return vec4<f32>(0.2, 0.2, 0.2, 1.0);
    } elseif (all(input.lc0 != lights.lights[0].color)) {
        return vec4<f32>(0.3, 0.3, 0.3, 1.0);
    } elseif (distance(input.lc0, alc0) > EPS) {
        return vec4<f32>(0.3, 0.3, 0.3, 1.0);
    } elseif (any(input.lt0 != lights.lights[0].light_type)) {
        return vec4<f32>(0.4, 0.4, 0.4, 1.0);
    } elseif (any(input.lt0 != alt0)) {
        return vec4<f32>(0.4, 0.4, 0.4, 1.0);
    } elseif (any(input.lp1 != lights.lights[1].position)) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } elseif (distance(input.lp1, alp1) > EPS) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } elseif (any(input.lc1 != lights.lights[1].color)) {
        return vec4<f32>(0.6, 0.6, 0.6, 1.0);
    } elseif (distance(input.lc1, alc1) > EPS) {
        return vec4<f32>(0.6, 0.6, 0.6, 1.0);
    } elseif (any(input.lt1 != lights.lights[1].light_type)) {
        return vec4<f32>(0.7, 0.7, 0.7, 1.0);
    } elseif (any(input.lt1 != alt1)) {
        return vec4<f32>(0.7, 0.7, 0.7, 1.0);
    } elseif (input.st != info.time) {
        return vec4<f32>(0.8, 0.8, 0.8, 1.0);
    } elseif (input.snl != info.nlights) {
        return vec4<f32>(0.9, 0.9, 0.9, 1.0);
    } elseif (input.snl != asnl) {
        return vec4<f32>(0.9, 0.9, 0.9, 1.0);
    } else {
        return vec4<f32>(0.2, 0.4, 0.6, 0.8);
    }
}
