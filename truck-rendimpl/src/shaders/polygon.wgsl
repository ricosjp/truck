struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2)]] normal: vec3<f32>; 
};

struct Camera {
    matrix: mat4x4<f32>;
    projection: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: Camera;

struct Lights {
    lights: array<Light, 255>;
};

[[group(0), binding(1)]]
var<uniform> lights: Lights;

struct SceneInfo {
    bk_color: vec4<f32>;
    time: f32;
    nlights: u32;
};

[[group(0), binding(2)]]
var<uniform> info: SceneInfo;

struct ModelMatrix {
    matrix: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> model_matrix: ModelMatrix;

struct ModelMaterial {
    material: Material;
};

[[group(1), binding(1)]]
var<uniform> material: ModelMaterial;

[[group(1), binding(2)]]
var r_color: texture_2d<f32>;

[[group(1), binding(3)]]
var r_sampler: sampler;

struct VertexOutput {
    [[builtin(position)]] gl_position: vec4<f32>;
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2)]] normal: vec3<f32>; 
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
    return out;
}

let e: vec2<f32> = vec2<f32>(1.0, 0.0);

[[stage(fragment)]]
fn nontex_main(in: VertexInput) -> [[location(0)]] vec4<f32> {
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
    pre_color = background_correction(pre_color, info.bk_color.xyz, material.material);
    pre_color = ambient_correction(pre_color, material.material);

    return vec4<f32>(pre_color, material.material.albedo.a);
}

[[stage(fragment)]]
fn tex_main(in: VertexInput) -> [[location(0)]] vec4<f32> {
    var matr: Material = material.material;
    matr.albedo = textureSample(r_color, r_sampler, in.uv);
    let camera_dir = normalize((camera.matrix * e.yyyx).xyz - in.position);
    let normal = normalize(in.normal);
    var pre_color: vec3<f32> = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < info.nlights; i = i + 1u) {
        pre_color = pre_color + microfacet_color(
            in.position,
            normal,
            lights.lights[i],
            camera_dir,
            matr,
        );
    }
    pre_color = clamp(pre_color, vec3<f32>(0.0), vec3<f32>(1.0));
    pre_color = background_correction(pre_color, info.bk_color.xyz, material.material);
    pre_color = ambient_correction(pre_color, matr);

    return vec4<f32>(pre_color, matr.albedo.a);
}
