use espr::{ast::SyntaxTree, ir::IR};
use quote::ToTokens;
use std::{env, path::PathBuf};

const NUM_ERROR_LINES: usize = 10;

const EXPRESS: &[&str] = &["10303-201-aim-long.exp"];

fn build_express(src: &str) -> String {
    let st = match SyntaxTree::parse(src) {
        Ok(st) => st,
        Err(e) => {
            for (code, kind) in e.errors {
                eprintln!(
                    "Syntax Error occurred while parsing following line [{:?}]:",
                    kind
                );
                for line in code.lines().take(NUM_ERROR_LINES) {
                    eprintln!("> {}", line);
                }
                eprintln!();
            }
            panic!("Syntax Error");
        }
    };
    IR::from_syntax_tree(&st)
        .expect("Failed in semantic analysis phase")
        .to_token_stream()
        .to_string()
}

fn main() {
    let express_dir = PathBuf::from("express");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    for exp_rel in EXPRESS {
        let exp_rel = PathBuf::from(exp_rel);
        let exp_path = express_dir.join(&exp_rel);
        let exp_src = std::fs::read_to_string(exp_path).unwrap();
        let out_src = build_express(&exp_src);
        let mut out_rel = out_dir.join(&exp_rel);
        out_rel.set_extension("rs");
        std::fs::write(out_rel, &out_src).unwrap();
    }
}
