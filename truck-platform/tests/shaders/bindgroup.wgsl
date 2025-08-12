struct Camera {
    camera_matrix: mat4x4<f32>,
    projection: mat4x4<f32>,
}

struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
    light_type: vec4<u32>,
}

struct Lights {
    lights: array<Light, 255>,
}

struct SceneInfo {
    bk_color: vec4<f32>,
    resolution: vec2<u32>,
    time: f32,
    nlights: u32,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(0)
@binding(1)
var<uniform> lights: Lights;

@group(0)
@binding(2)
var<uniform> info: SceneInfo;

const acm0: vec4<f32> = vec4<f32>(1.0, 2.1, 3.2, 4.3);
const acm1: vec4<f32> = vec4<f32>(5.4, 6.5, 7.6, 8.7);
const acm2: vec4<f32> = vec4<f32>(9.8, 10.9, 11.0, 12.0);
const acm3: vec4<f32> = vec4<f32>(13.0, 14.0, 15.0, 16.23);
    
const acp0: vec4<f32> = vec4<f32>(11.714964805291158, -20.083602331793195, 0.6148440931060913, 1.0);
const acp1: vec4<f32> = vec4<f32>(-7.919279773964541, 12.919469128453738, -1.6688625384307627, -2.0);
const acp2: vec4<f32> = vec4<f32>(-23.645933626763625, 36.55672789512988, 1.4931927975432673, 1.0);
const acp3: vec4<f32> = vec4<f32>(19.301563694896643, -28.84390979125007, -0.4391743522185905, 0.0);

const alp0: vec4<f32> = vec4<f32>(0.1, 0.2, 0.3, 1.0);
const alc0: vec4<f32> = vec4<f32>(0.4, 0.5, 0.6, 1.0);
const alt0: vec4<u32> = vec4<u32>(0u, 0u, 0u, 0u);
const alp1: vec4<f32> = vec4<f32>(1.1, 1.2, 1.3, 1.0);
const alc1: vec4<f32> = vec4<f32>(1.4, 1.5, 1.6, 1.0);
const alt1: vec4<u32> = vec4<u32>(1u, 0u, 0u, 0u);
const asnl: u32 = 2u;
const abk: vec4<f32> = vec4<f32>(0.1, 0.2, 0.3, 0.4);
const arsl: vec2<u32> = vec2<u32>(256u, 256u);
 
const EPS: f32 = 1.0e-5;
const e: vec2<f32> = vec2<f32>(1.0, 0.0);

@vertex
fn vs_main(@location(0) idx: u32) -> @builtin(position) vec4<f32> {
    var vertex: array<vec2<f32>, 4>;
    vertex[0] = vec2<f32>(-1.0, -1.0);
    vertex[1] = vec2<f32>(1.0, -1.0);
    vertex[2] = vec2<f32>(-1.0, 1.0);
    vertex[3] = vec2<f32>(1.0, 1.0);
    if (distance(camera.camera_matrix * e.xyyy, acm0) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.camera_matrix * e.yxyy, acm1) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.camera_matrix * e.yyxy, acm2) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.camera_matrix * e.yyyx, acm3) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.projection * e.xyyy, acp0) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.projection * e.yxyy, acp1) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.projection * e.yyxy, acp2) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.projection * e.yyyx, acp3) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(lights.lights[0].position, alp0) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(lights.lights[0].color, alc0) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (any(lights.lights[0].light_type != alt0)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(lights.lights[1].position, alp1) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(lights.lights[1].color, alc1) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (any(lights.lights[1].light_type != alt1)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (info.nlights != asnl) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(info.bk_color, abk) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (any(info.resolution != arsl)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else {
        return vec4<f32>(vertex[idx], 0.0, 1.0);
    }
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    if (distance(camera.camera_matrix * e.xyyy, acm0) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.camera_matrix * e.yxyy, acm1) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.camera_matrix * e.yyxy, acm2) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.camera_matrix * e.yyyx, acm3) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.projection * e.xyyy, acp0) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } else if (distance(camera.projection * e.yxyy, acp1) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } else if (distance(camera.projection * e.yyxy, acp2) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } else if (distance(camera.projection * e.yyyx, acp3) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } else if (distance(lights.lights[0].position, alp0) > EPS) {
        return vec4<f32>(0.2, 0.2, 0.2, 1.0);
    } else if (distance(lights.lights[0].color, alc0) > EPS) {
        return vec4<f32>(0.3, 0.3, 0.3, 1.0);
    } else if (any(lights.lights[0].light_type != alt0)) {
        return vec4<f32>(0.4, 0.4, 0.4, 1.0);
    } else if (distance(lights.lights[1].position, alp1) > EPS) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } else if (distance(lights.lights[1].color, alc1) > EPS) {
        return vec4<f32>(0.6, 0.6, 0.6, 1.0);
    } else if (any(lights.lights[1].light_type != alt1)) {
        return vec4<f32>(0.7, 0.7, 0.7, 1.0);
    } else if (info.nlights != asnl) {
        return vec4<f32>(0.8, 0.8, 0.8, 1.0);
    } else if (distance(info.bk_color, abk) > EPS) {
        return vec4<f32>(0.9, 0.9, 0.9, 1.0);
    } else if (any(info.resolution != arsl)) {
        return vec4<f32>(0.9, 0.9, 0.9, 1.0);
    } else {
        return vec4<f32>(0.2, 0.4, 0.6, 0.8);
    }
}

@fragment
fn fs_main_anti() -> @location(0) vec4<f32> {
    if (distance(camera.camera_matrix * e.xyyy, acm0) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.camera_matrix * e.yxyy, acm1) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.camera_matrix * e.yyxy, acm2) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.camera_matrix * e.yyyx, acm3) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else if (distance(camera.projection * e.xyyy, acp0) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } else if (distance(camera.projection * e.yxyy, acp1) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } else if (distance(camera.projection * e.yyxy, acp2) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } else if (distance(camera.projection * e.yyyx, acp3) > EPS) {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    } else if (distance(lights.lights[0].position, alp0) > EPS) {
        return vec4<f32>(0.2, 0.2, 0.2, 1.0);
    } else if (distance(lights.lights[0].color, alc0) > EPS) {
        return vec4<f32>(0.3, 0.3, 0.3, 1.0);
    } else if (any(lights.lights[0].light_type != alt0)) {
        return vec4<f32>(0.4, 0.4, 0.4, 1.0);
    } else if (distance(lights.lights[1].position, alp1) > EPS) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } else if (distance(lights.lights[1].color, alc1) > EPS) {
        return vec4<f32>(0.6, 0.6, 0.6, 1.0);
    } else if (any(lights.lights[1].light_type == alt1)) {
        return vec4<f32>(0.7, 0.7, 0.7, 1.0);
    } else if (info.nlights != asnl) {
        return vec4<f32>(0.8, 0.8, 0.8, 1.0);
    } else if (distance(info.bk_color, abk) > EPS) {
        return vec4<f32>(0.9, 0.9, 0.9, 1.0);
    } else if (any(info.resolution != arsl)) {
        return vec4<f32>(0.9, 0.9, 0.9, 1.0);
    } else {
        return vec4<f32>(0.2, 0.4, 0.6, 0.8);
    }
}
