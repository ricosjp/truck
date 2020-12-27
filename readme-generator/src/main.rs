use std::process::Command;
use std::io::Write;

const WORKSPACES: [&str; 7] = [
	"truck-base",
	"truck-geometry",
	"truck-modeling",
	"truck-platform",
	"truck-polymesh",
	"truck-rendimpl",
	"truck-topology",
];

fn create_readme() {
    let mut readme = std::fs::File::create("README.md").unwrap();
    let output = Command::new("cargo").args(&["readme"]).output().unwrap();
    readme.write(&output.stdout).unwrap();
    let dir = match std::fs::read_dir("examples") {
        Ok(got) => got,
        Err(_) => return,
    };
    readme.write_fmt(format_args!("\n# Sample Codes\n")).unwrap();
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
    //std::env::set_current_dir("..").unwrap();
    for path in &WORKSPACES {
        std::env::set_current_dir(path).unwrap();
        create_readme();
        std::env::set_current_dir("..").unwrap();
    }
}
