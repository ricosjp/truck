use std::process::Command;
use std::io::Write;

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
        .args(&["build", "--target", "wasm32-unknown-unknown", "--examples", "--release"])
        .output()
        .unwrap_or_else(|e| panic!("{}", e));
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
    if !output.status.success() {
        println!("build failed");
        return;
    }
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
                &format!("target/wasm32-unknown-unknown/release/examples/{}.wasm", dir),
            ])
            .output()
            .unwrap_or_else(|e| panic!("{}", e));
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        if !output.status.success() {
            println!("wasm-bindgen failed");
            return;
        }
        std::fs::write(
            format!("{}/index.html", output_dir),
            include_str!("example-index.html").replace("{example}", dir),
        )
        .unwrap_or_else(|e| panic!("{}", e));
        sum += &format!("<li><a href=\"{}/index.html\">{0}</a></li>", dir);
    }
    std::fs::write(
        format!("dist/index.html"),
        include_str!("index.html").replace("<!-- index -->", &sum),
    )
    .unwrap_or_else(|e| panic!("{}", e));
}
