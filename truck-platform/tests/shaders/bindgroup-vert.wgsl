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

