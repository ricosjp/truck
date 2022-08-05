#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]
#![warn(
    //missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use std::io::Write;
use std::process::Command;

const EXAMPLES: &[&str] = &[
    "wgsl-sandbox",
    "bsp-animation",
    "collision-sphere",
    "material-samples",
    "rotate-objects",
    "simple-obj-viewer",
    "simple-shape-viewer",
    "textured-cube",
];

fn main() {
    let output = Command::new("cargo")
        .args(&[
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--examples",
            "--release",
        ])
        .output()
        .unwrap_or_else(|e| panic!("{}", e));
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success(), "build failed");
    let mut sum = String::new();
    for dir in EXAMPLES {
        let output_dir = format!("dist/{}", dir);
        std::fs::create_dir_all(&output_dir).unwrap_or_else(|e| panic!("{}", e));
        let output = Command::new("wasm-bindgen")
            .args(&[
                "--target",
                "web",
                "--out-dir",
                &output_dir,
                &format!(
                    "target/wasm32-unknown-unknown/release/examples/{}.wasm",
                    dir
                ),
            ])
            .output()
            .unwrap_or_else(|e| panic!("{}", e));
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        assert!(output.status.success(), "wasm-bindgen failed");
        std::fs::write(
            format!("{}/index.html", output_dir),
            include_str!("example-index.html").replace("{example}", dir),
        )
        .unwrap_or_else(|e| panic!("{}", e));
        std::fmt::Write::write_fmt(
            &mut sum,
            format_args!("<li><a href=\"{dir}/index.html\">{dir}</a></li>"),
        )
        .unwrap_or_else(|e| panic!("{}", e));
    }
    std::fs::write(
        "dist/index.html",
        include_str!("index.html").replace("<!-- index -->", &sum),
    )
    .unwrap_or_else(|e| panic!("{}", e));
}
