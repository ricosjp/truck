[[block]]
struct Camera {
    matrix: mat4x4<f32>;
    projection: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: Camera;

[[block]]
struct ModelMatrix {
    matrix: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> model_matrix: ModelMatrix;

[[block]]
struct Color {
    color: vec4<f32>;
};

[[group(1), binding(1)]]
var<uniform> color: Color;

[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec3<f32>) -> [[builtin(position)]] vec4<f32> {
    var res: vec4<f32> = camera.projection * model_matrix.matrix * vec4<f32>(position, 1.0);
    res.z = res.z - 1.0e-4;
    return res;
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return color.color;
}
