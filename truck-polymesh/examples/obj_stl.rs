//! Converts OBJ and STL to each other.
//!
//! usage:
//!
//! ```bash
//! cargo run --example obj_stl <input-file>
//! ```

use stl::*;
use truck_polymesh::*;

fn main() {
    let args = &mut std::env::args();
    if args.len() < 2 {
        eprintln!("usage: obj_stl <input-file>");
        return;
    }
    args.next().unwrap();
    let arg = args.next().unwrap();
    let path: &std::path::Path = arg.as_ref();
    let ext = match path.extension() {
        Some(ext) => ext,
        None => {
            eprintln!("cannot infer file type");
            return;
        }
    };
    if ext == "obj" {
        let file = std::fs::File::open(path).unwrap();
        let polymesh = obj::read(file).unwrap();
        let mut outpath: String = path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        outpath += ".stl";
        let mut file = std::fs::File::create(&outpath).unwrap();
        stl::write(&polymesh, &mut file, StlType::Binary).unwrap();
    } else if ext == "stl" {
        let file = std::fs::File::open(path).unwrap();
        let polymesh = stl::read(file, StlType::Automatic).unwrap();
        let mut outpath: String = path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        outpath += ".obj";
        let mut file = std::fs::File::create(&outpath).unwrap();
        obj::write(&polymesh, &mut file).unwrap();
    } else {
        eprintln!("cannot infer file type");
    }
}
