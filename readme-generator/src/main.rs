use std::io::Write;
use std::process::Command;

const WORKSPACES: [&str; 11] = [
    "truck-base",
    "truck-geometry",
    "truck-geotrait",
    "truck-meshalgo",
    "truck-modeling",
    "truck-platform",
    "truck-polymesh",
    "truck-rendimpl",
    "truck-shapeops",
    "truck-topology",
    "truck-stepio",
];

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
    let output = Command::new("cargo").args(["readme"]).output().unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    println!("{}", output);
    let lines: Vec<_> = output.split('\n').collect();
    readme
        .write_fmt(format_args!(
            "{}\n\n{}\n\n{}\n",
            lines[0],
            badge_url(path),
            lines[2]
        ))
        .unwrap();
    let dir = match std::fs::read_dir("examples") {
        Ok(got) => got,
        Err(_) => return,
    };
    let mut dir = dir.map(|file| file.unwrap().path()).collect::<Vec<_>>();
    dir.sort();

    readme
        .write_fmt(format_args!("\n## Sample Codes\n"))
        .unwrap();
    for path in dir {
        let extension = path.extension();
        if extension
            .map(|e| e.to_str().unwrap() != "rs")
            .unwrap_or(true)
        {
            continue;
        }
        let filestem = path.file_stem().unwrap().to_str().unwrap();
        readme
            .write_fmt(format_args!("\n### {}\n\n", filestem))
            .unwrap();
        let output = Command::new("cargo")
            .args(["readme", "--no-license", "--no-title"])
            .arg("-i")
            .arg(path.to_str().unwrap())
            .output()
            .unwrap();
        readme.write_all(&output.stdout).unwrap();
    }
}

fn main() {
    for path in &WORKSPACES {
        println!("{}", path);
        std::env::set_current_dir(path).unwrap();
        create_readme(path);
        std::env::set_current_dir("..").unwrap();
    }
}
