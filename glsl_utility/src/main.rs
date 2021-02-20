extern crate glsl_to_spirv;
use glsl_to_spirv::ShaderType;
use std::io::Read;

fn resolve_include<S: AsRef<str>>(code: S) -> String {
    let mut res = String::new();
    for line in code.as_ref().split("\n") {
        let words: Vec<_> = line.split_whitespace().collect();
        if !words.is_empty() {
            if words[0] == "#include" {
                let filename = words[1].trim_matches('\"');
                res += &std::fs::read_to_string(filename).unwrap();
            } else {
                res += &line;
            }
        }
        res += "\n";
    }
    res
}

fn save_spirv(filename: &str, shadertype: ShaderType) {
    let code = resolve_include(std::fs::read_to_string(filename).unwrap());
    let mut spirv = glsl_to_spirv::compile(&code, shadertype).unwrap();
    let mut compiled = Vec::new();
    spirv.read_to_end(&mut compiled).unwrap();
    let output_name = filename.to_string() + ".spv";
    std::fs::write(&output_name, &compiled).unwrap();
}

fn main() {
    std::env::set_current_dir("truck-rendimpl/src/shaders").unwrap();
    save_spirv("polygon.vert", ShaderType::Vertex);
    save_spirv("polygon.frag", ShaderType::Fragment);
    save_spirv("textured-polygon.frag", ShaderType::Fragment);
    save_spirv("face.vert", ShaderType::Vertex);
    save_spirv("face.frag", ShaderType::Fragment);
    save_spirv("textured-face.frag", ShaderType::Fragment);
    save_spirv("line.vert", ShaderType::Vertex);
    save_spirv("line.frag", ShaderType::Fragment);
}
