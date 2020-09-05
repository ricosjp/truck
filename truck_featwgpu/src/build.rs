extern crate glsl_to_spirv;
use glsl_to_spirv::ShaderType;
use std::fs::File;
use std::io::{Read, Write};

fn save_spirv(filename: &str, shadertype: ShaderType) {
    let mut source = File::open(filename).unwrap();
    let mut code = String::new();
    source.read_to_string(&mut code).unwrap();
    let mut spirv = glsl_to_spirv::compile(&code, shadertype).unwrap();
    let mut compiled = Vec::new();
    spirv.read_to_end(&mut compiled).unwrap();
    let output_name = filename.to_string() + ".spv";
    let mut output = File::create(&output_name).unwrap();
    output.write(&compiled).unwrap();
}

fn main() {
    std::env::set_current_dir("src").unwrap();
    save_spirv("shaders/polygon.vert", ShaderType::Vertex);
    save_spirv("shaders/polygon.frag", ShaderType::Fragment);
    //save_spirv("shaders/textured_polygon.frag", ShaderType::Fragment);
}