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

[[block]]
struct Boundary {
    boundary: [[stride(16)]] array<vec4<f32>>;
};
[[group(1), binding(0)]]
var<storage> boundary: [[access(read)]] Boundary;

[[group(1), binding(1)]]
var<uniform> model_matrix: ModelMatrix;

[[group(1), binding(2)]]
var<uniform> material: Material;

[[group(1), binding(3)]]
var r_color: texture_2d<f32>;

[[group(1), binding(4)]]
var r_sampler: sampler;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2)]] normal: vec3<f32>;
    [[location(3)]] boundary_range: vec2<u32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] inp: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2), interpolate(flat)]] boundary_range: vec2<u32>;
};

let EPS: f32 = 1.0e-6;
let e: vec2<f32> = vec2<f32>(1.0, 0.0);

fn fract_distance(x: f32, y: f32) -> f32 {
    let a = abs(x - y);
    let b = abs(1.0 + x - y);
    let c = abs(x - y - 1.0);
    return min(a, min(b, c));
}

fn current_normal(in: VertexInput) -> vec3<f32> {
    if (distance(in.uv, vec2<f32>(in.position.x, (1.0 + in.position.y) / 2.0)) > 0.5) {
        return vec3<f32>(0.0, 0.0, 1.0);
    } else {
        return vec3<f32>(-1.0, 0.0, 1.0) / sqrt(2.0);
    }
}

fn answer_range(inp: vec3<f32>) -> vec2<u32> {
    if (inp.x < 0.0) {
        return vec2<u32>(0u, 4u);
    } else {
        return vec2<u32>(4u, 8u);
    }
}

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.inp = vec3<f32>(0.0);
    out.uv = vec2<f32>(0.1);
    out.boundary_range = vec2<u32>(0u);
    if (fract_distance(fract(in.position.x), in.uv.x) > EPS) {
        out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (abs((1.0 + in.position.y) / 2.0 - in.uv.y) > EPS) {
        out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(in.normal, current_normal(in)) > EPS) {
        out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.xyyy, vec4<f32>(1.0, 2.0, 3.0, 4.0)) > EPS) {
        out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yxyy, vec4<f32>(5.0, 6.0, 7.0, 8.0)) > EPS) {
        out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yyxy, vec4<f32>(9.0, 10.0, 11.0, 12.0)) > EPS) {
        out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } elseif (distance(model_matrix.matrix * e.yyyx, vec4<f32>(13.0, 14.0, 15.0, 16.0)) > EPS) {
        out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);  
    } else {
        out.position = vec4<f32>(in.position.xy, 0.0, 1.0);
        out.inp = in.position;
        out.uv = in.uv;
        out.boundary_range = in.boundary_range;
    }
    return out;
}

[[stage(fragment)]]
fn nontex_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
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
    } elseif (any(in.boundary_range != answer_range(in.inp))) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    } elseif (distance(boundary.boundary[0], vec4<f32>(0.0, 0.0, 1.0, 0.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[1], vec4<f32>(1.0, 0.0, 1.0, 1.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[2], vec4<f32>(1.0, 1.0, 0.0, 1.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[3], vec4<f32>(0.0, 1.0, 0.0, 0.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0); 
    } else {
        return vec4<f32>(0.2, 0.4, 0.6, 0.8);
    }
}

[[stage(fragment)]]
fn nontex_main_anti(in: VertexOutput) -> [[location(0)]] vec4<f32> {
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
    } elseif (any(in.boundary_range == answer_range(in.inp))) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } elseif (distance(boundary.boundary[0], vec4<f32>(0.0, 0.0, 1.0, 0.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[1], vec4<f32>(1.0, 0.0, 1.0, 1.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[2], vec4<f32>(1.0, 1.0, 0.0, 1.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[3], vec4<f32>(0.0, 1.0, 0.0, 0.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0); 
    } else {
        return vec4<f32>(0.2, 0.4, 0.6, 0.8);
    }
}

[[stage(fragment)]]
fn tex_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let uv = vec2<f32>(1.0 + in.inp.x, 1.0 - in.inp.y) / 2.0;
    let col = textureSample(r_color, r_sampler, uv);
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
    } elseif (any(in.boundary_range != answer_range(in.inp))) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } elseif (distance(boundary.boundary[0], vec4<f32>(0.0, 0.0, 1.0, 0.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[1], vec4<f32>(1.0, 0.0, 1.0, 1.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[2], vec4<f32>(1.0, 1.0, 0.0, 1.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[3], vec4<f32>(0.0, 1.0, 0.0, 0.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0); 
    } else {
        return col;
    }
}

[[stage(fragment)]]
fn tex_main_anti(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let uv = vec2<f32>(1.0 + in.inp.x, 1.0 - in.inp.y) / 2.0;
    let col = textureSample(r_color, r_sampler, uv);
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
    } elseif (any(in.boundary_range == answer_range(in.inp))) {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0);
    } elseif (distance(boundary.boundary[0], vec4<f32>(0.0, 0.0, 1.0, 0.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[1], vec4<f32>(1.0, 0.0, 1.0, 1.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[2], vec4<f32>(1.0, 1.0, 0.0, 1.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0);
    } elseif (distance(boundary.boundary[3], vec4<f32>(0.0, 1.0, 0.0, 0.0)) > EPS) {
        return vec4<f32>(0.75, 0.75, 0.75, 1.0); 
    } else {
        return col;
    }
}
