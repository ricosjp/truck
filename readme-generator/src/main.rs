use std::io::Write;
use std::process::Command;

const WORKSPACES: [&str; 6] = [
    "truck-base",
    "truck-geometry",
    "truck-modeling",
    "truck-platform",
    "truck-rendimpl",
    "truck-topology",
];

fn create_readme() {
    let mut readme = std::fs::File::create("README.md").unwrap();
    let output = Command::new("cargo").args(&["readme"]).output().unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = output.split("\n").collect();
    readme
        .write_fmt(format_args!("{}\n{}\n", lines[0], lines[2]))
        .unwrap();
    let dir = match std::fs::read_dir("examples") {
        Ok(got) => got,
        Err(_) => return,
    };
    readme
        .write_fmt(format_args!("\n# Sample Codes\n"))
        .unwrap();
    for file in dir {
        let path = file.unwrap().path();
        let extension = path.extension().unwrap().to_str().unwrap();
        if extension != "rs" {
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
        create_readme();
        std::env::set_current_dir("..").unwrap();
    }
}
