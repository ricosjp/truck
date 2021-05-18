// cf: https://thebookofshaders.com/10/
fn random(uv: vec2<f32>) -> f32 {
    let c = vec2<f32>(12.9898, 78.233) * 43758.5453123;
    return fract(sin(dot(uv, c)));
}

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
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
    output.uv = vertex[idx];
    return output;
}

[[stage(fragment)]]
fn unicolor() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.2, 0.4, 0.6, 0.8);
}

[[stage(fragment)]]
fn random_texture([[location(0)]] uv: vec2<f32>) -> [[location(0)]] vec4<f32> {
    let r = random(uv);
    let g = random(uv.yx);
    let b = random(vec2<f32>(r, g));
    return vec4<f32>(r, g, b, 1.0);
}

[[stage(fragment)]]
fn gradation_texture([[location(0)]] uv: vec2<f32>) -> [[location(0)]] vec4<f32> {
    let r = length(uv) / sqrt(2.0);
    let l = 1.0 - r;
    let col0 = vec3<f32>(r, r * r, r * r * r);
    let col1 = vec3<f32>(l * l * l, l, l * l);
    return vec4<f32>(clamp(col0 + col1, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
