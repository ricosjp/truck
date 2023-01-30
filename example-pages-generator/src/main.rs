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

use std::io::{BufRead, BufReader};
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
    let mut child = Command::new("cargo")
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--examples",
            "--release",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("{}", e));
    let stdout = BufReader::new(child.stdout.take().expect("no stdout"));
    let stderr = BufReader::new(child.stderr.take().expect("no stderr"));
    let _thread0 = std::thread::spawn(|| {
        stdout
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| println!("{line}"))
    });
    let _thread1 = std::thread::spawn(|| {
        stderr
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| println!("{line}"))
    });
    child.wait().unwrap_or_else(|e| panic!("{}", e));
    let mut sum = String::new();
    for dir in EXAMPLES {
        let output_dir = format!("dist/{dir}");
        std::fs::create_dir_all(&output_dir).unwrap_or_else(|e| panic!("{}", e));
        let mut child = Command::new("wasm-bindgen")
            .args([
                "--target",
                "web",
                "--out-dir",
                &output_dir,
                &format!(
                    "target/wasm32-unknown-unknown/release/examples/{dir}.wasm",
                ),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("{}", e));
        let stdout = BufReader::new(child.stdout.take().expect("no stdout"));
        let stderr = BufReader::new(child.stderr.take().expect("no stderr"));
        let _thread0 = std::thread::spawn(|| {
            stdout
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| println!("{line}"))
        });
        let _thread1 = std::thread::spawn(|| {
            stderr
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| println!("{line}"))
        });
        child.wait().unwrap_or_else(|e| panic!("{}", e));
        std::fs::write(
            format!("{output_dir}/index.html"),
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
