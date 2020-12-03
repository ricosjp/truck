extern crate glsl_to_spirv;
use glsl_to_spirv::ShaderType;
use std::io::Read;

fn save_vertex_spirv(filename: &str) {
    let code = std::fs::read_to_string(filename).unwrap();
    let mut spirv = glsl_to_spirv::compile(&code, ShaderType::Vertex).unwrap();
    let mut compiled = Vec::new();
    spirv.read_to_end(&mut compiled).unwrap();
    let output_name = filename.to_string() + ".spv";
    std::fs::write(&output_name, &compiled).unwrap();
}

fn save_fragment_spirv(filename: &str) {
    let mut code = std::fs::read_to_string("microfacet_module.frag").unwrap();
    code += &std::fs::read_to_string(filename).unwrap();
    let mut spirv = glsl_to_spirv::compile(&code, ShaderType::Fragment).unwrap();
    let mut compiled = Vec::new();
    spirv.read_to_end(&mut compiled).unwrap();
    let output_name = filename.to_string() + ".spv";
    std::fs::write(&output_name, &compiled).unwrap();
}

fn main() {
    std::env::set_current_dir("src/shaders").unwrap();
    save_vertex_spirv("polygon.vert");
    save_fragment_spirv("polygon.frag");
    save_fragment_spirv("textured-polygon.frag");
    save_fragment_spirv("face.frag");
    save_fragment_spirv("textured-face.frag");
}
