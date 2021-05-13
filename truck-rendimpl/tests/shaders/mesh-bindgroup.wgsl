[[block]]
struct ModelMatrix {
    matrix: mat4x4<f32>;
};

[[block]]
struct Material {
    albedo: vec4<f32>;
    roughness: f32;
    reflectance: f32;
    ambient_ratio: f32;
};

[[group(1), binding(0)]]
var<uniform> model_matrix: ModelMatrix;

[[group(1), binding(1)]]
var<uniform> material: Material;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2)]] normal: vec3<f32>;
};

let EPS: f32 = 1.0e-6;
let e: vec2<f32> = vec2<f32>(1.0, 0.0);

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> [[builtin(position)]] vec4<f32> {
    if (distance(in.position, vec3<f32>(in.uv.x, 2.0, in.uv.y)) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(in.normal, vec3<f32>(in.uv.y, 0.2, in.uv.x)) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.xyyy, vec4<f32>(1.0, 2.0, 3.0, 4.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yxyy, vec4<f32>(5.0, 6.0, 7.0, 8.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yyxy, vec4<f32>(9.0, 10.0, 11.0, 12.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yyyx, vec4<f32>(13.0, 14.0, 15.0, 16.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);  
    } else {
        return vec4<f32>(in.uv, 0.0, 1.0);
    } 
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    if (distance(model_matrix.matrix * e.xyyy, vec4<f32>(1.0, 2.0, 3.0, 4.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yxyy, vec4<f32>(5.0, 6.0, 7.0, 8.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yyxy, vec4<f32>(9.0, 10.0, 11.0, 12.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yyyx, vec4<f32>(13.0, 14.0, 15.0, 16.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0); 
    } elseif (distance(material.albedo, vec4<f32>(0.2, 0.4, 0.6, 1.0)) > EPS) {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0);
    } elseif (abs(material.roughness - 0.31415) > EPS) {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    } elseif (abs(material.reflectance - 0.29613) > EPS) {
        return vec4<f32>(0.0, 1.0, 1.0, 1.0);
    } elseif (abs(material.ambient_ratio - 0.92) > EPS) {
        return vec4<f32>(0.25, 0.25, 0.25, 1.0);
    } else {
        return vec4<f32>(0.2, 0.4, 0.6, 0.8);
    }
}

[[stage(fragment)]]
fn fs_main_anti() -> [[location(0)]] vec4<f32> {
    if (distance(model_matrix.matrix * e.xyyy, vec4<f32>(1.0, 2.0, 3.0, 4.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yxyy, vec4<f32>(5.0, 6.0, 7.0, 8.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yyxy, vec4<f32>(9.0, 10.0, 11.0, 12.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yyyx, vec4<f32>(13.0, 14.0, 15.0, 16.0)) > EPS) {
        return vec4<f32>(0.0, 0.0, 1.0, 1.0); 
    } elseif (distance(material.albedo, vec4<f32>(0.2, 0.4, 0.6, 1.0)) > EPS) {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0);
    } elseif (abs(material.roughness - 0.31415) > EPS) {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    } elseif (abs(material.reflectance - 0.29613) < EPS) {
        return vec4<f32>(0.0, 1.0, 1.0, 1.0);
    } elseif (abs(material.ambient_ratio - 0.92) > EPS) {
        return vec4<f32>(0.25, 0.25, 0.25, 1.0);
    } else {
        return vec4<f32>(0.2, 0.4, 0.6, 0.8);
    }
}
