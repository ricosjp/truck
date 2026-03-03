struct Camera {
    _matrix: mat4x4<f32>,
    projection: mat4x4<f32>,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

struct ModelMatrix {
    model_matrix: mat4x4<f32>,
}

@group(1)
@binding(0)
var<uniform> model_matrix: ModelMatrix;

struct Color {
    color: vec4<f32>,
}

@group(1)
@binding(1)
var<uniform> color: Color;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    var res: vec4<f32> = camera.projection * model_matrix.model_matrix * vec4<f32>(position, 1.0);
    res.z = res.z - 1.0e-4;
    return res;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(pow(color.color.rgb, vec3<f32>(0.4545)), color.color.a);
}
