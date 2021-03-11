use std::io::Write;
use std::process::Command;

const WORKSPACES: [&str; 7] = [
    "truck-base",
    "truck-geometry",
    "truck-modeling",
    "truck-polymesh",
    "truck-platform",
    "truck-rendimpl",
    "truck-topology",
];

const DEVDEPENDS_CMAKE: [&str; 3] = ["truck-rendimpl", "truck-modeling", "truck-platform"];

const DEVDEPENDS_CMAKE_MESSAGE: &str = "## Dependencies
The dev-dependencies of this crate includes [CMake](https://cmake.org).
";

fn cmake_flag(path: &&str) -> usize {
    if DEVDEPENDS_CMAKE.iter().any(|s| s == path) {
        1
    } else {
        0
    }
}

fn badge_url(path: &str) -> String {
    format!(
        "[![Crates.io](https://img.shields.io/crates/v/{0}.svg)]\
(https://crates.io/crates/{0}) \
[![Docs.rs](https://docs.rs/{0}/badge.svg)]\
(https://docs.rs/{0})",
        path
    )
}

fn create_readme(path: &str) {
    let mut readme = std::fs::File::create("README.md").unwrap();
    let output = Command::new("cargo").args(&["readme"]).output().unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = output.split("\n").collect();
    readme
        .write_fmt(format_args!(
            "{}\n{}\n\n{}\n",
            lines[0],
            badge_url(path),
            lines[2]
        ))
        .unwrap();
    let dir = match std::fs::read_dir("examples") {
        Ok(got) => got,
        Err(_) => return,
    };

    match cmake_flag(&path) {
        1 => {
            readme.write(DEVDEPENDS_CMAKE_MESSAGE.as_bytes()).unwrap();
        }
        _ => {}
    }

    readme
        .write_fmt(format_args!("\n# Sample Codes\n"))
        .unwrap();
    for file in dir {
        let path = file.unwrap().path();
        let extension = path.extension();
        if extension
            .map(|e| e.to_str().unwrap() != "rs")
            .unwrap_or(true)
        {
            continue;
        }
        let filestem = path.file_stem().unwrap().to_str().unwrap();
        readme.write_fmt(format_args!("## {}\n", filestem)).unwrap();
        let output = Command::new("cargo")
            .args(&["readme", "--no-license", "--no-title"])
            .arg("-i")
            .arg(&path.to_str().unwrap())
            .output()
            .unwrap();
        readme.write(&output.stdout).unwrap();
    }
}

fn main() {
    for path in &WORKSPACES {
        std::env::set_current_dir(path).unwrap();
        create_readme(path);
        std::env::set_current_dir("..").unwrap();
    }
}
