struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2)]] normal: vec3<f32>; 
    [[location(3)]] boundary_range: vec2<u32>;
};

[[block]]
struct Camera {
    matrix: mat4x4<f32>;
    projection: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: Camera;

[[block]]
struct Lights {
    lights: [[stride(48)]] array<Light>;
};

[[group(0), binding(1)]]
var<storage> lights: [[access(read)]] Lights;

[[block]]
struct SceneInfo {
    time: f32;
    nlights: u32;
};

[[group(0), binding(2)]]
var<uniform> info: SceneInfo;

[[block]]
struct Boundary {
    boundary: [[stride(16)]] array<vec4<f32>>;
};

[[group(1), binding(0)]]
var<storage> boundary: [[access(read)]] Boundary;

[[block]]
struct ModelMatrix {
    matrix: mat4x4<f32>;
};

[[group(1), binding(1)]]
var<uniform> model_matrix: ModelMatrix;

[[block]]
struct ModelMaterial {
    material: Material;
};

[[group(1), binding(2)]]
var<uniform> material: ModelMaterial;

[[group(1), binding(3)]]
var r_color: texture_2d<f32>;

[[group(1), binding(4)]]
var r_sampler: sampler;

struct VertexOutput {
    [[builtin(position)]] gl_position: vec4<f32>;
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2)]] normal: vec3<f32>; 
    [[location(3)]] boundary_range: vec2<u32>;
};

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model_matrix.matrix * vec4<f32>(in.position, 1.0);
    let world_normal = model_matrix.matrix * vec4<f32>(in.normal, 0.0);
    out.gl_position = camera.projection * world_position;
    out.position = world_position.xyz;
    out.uv = in.uv;
    out.normal = normalize(world_normal.xyz);
    out.boundary_range = in.boundary_range;
    return out;
}

let e: vec2<f32> = vec2<f32>(1.0, 0.0);
fn in_domain(in: VertexInput) -> bool {
    var score: i32 = 0;
    for (var i: u32 = u32(in.boundary_range[0]); i < u32(in.boundary_range[1]); i = i + 1u) {
        let start = boundary.boundary[i].xy - in.uv;
        let end = boundary.boundary[i].zw - in.uv;
        if (start[1] * end[1] >= 0.0) {
            continue;
        }
        let abs_s = abs(start[1]);
        let abs_e = abs(end[1]);
        let x = (abs_e * start[0] + abs_s * end[0]) / (abs_s + abs_e);
        if (x > 0.0) {
            if (end[1] > 0.0) {
                score = score + 1;
            } else {
                score = score - 1;
            }
        }
    }
    return score > 0;
}

[[stage(fragment)]]
fn nontex_main(in: VertexInput) -> [[location(0)]] vec4<f32> {
    if (!in_domain(in)) {
        discard;
    }
    let camera_dir = normalize((camera.matrix * e.yyyx).xyz - in.position);
    let normal = normalize(in.normal);
    var pre_color: vec3<f32> = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < info.nlights; i = i + 1u) {
        pre_color = pre_color + microfacet_color(
            in.position,
            normal,
            lights.lights[i],
            camera_dir,
            material.material,
        );
    }
    pre_color = clamp(pre_color, vec3<f32>(0.0), vec3<f32>(1.0));
    pre_color = ambient_correction(pre_color, material.material);

    return vec4<f32>(pre_color, 1.0);
}

[[stage(fragment)]]
fn tex_main(in: VertexInput) -> [[location(0)]] vec4<f32> {
    var mat: Material = material.material;
    mat.albedo = textureSample(r_color, r_sampler, in.uv);
    if (!in_domain(in)) {
        discard;
    }
    let camera_dir = normalize((camera.matrix * e.yyyx).xyz - in.position);
    let normal = normalize(in.normal);
    var pre_color: vec3<f32> = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < info.nlights; i = i + 1u) {
        pre_color = pre_color + microfacet_color(
            in.position,
            normal,
            lights.lights[i],
            camera_dir,
            mat,
        );
    }
    pre_color = clamp(pre_color, vec3<f32>(0.0), vec3<f32>(1.0));
    pre_color = ambient_correction(pre_color, material.material);

    return vec4<f32>(pre_color, 1.0);
}

