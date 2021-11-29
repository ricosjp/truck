//! Adds smooth normals to the stanford bunny.
//!
//! - Input: bunny.obj
//! - Output: smooth_bunny.obj

use truck_meshalgo::filters::*;
use truck_polymesh::*;

fn main() {
    const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/obj/bunny.obj",);
    std::fs::copy(PATH, "bunny.obj").unwrap();
    let file = std::fs::File::open(PATH).unwrap();
    let mut mesh = obj::read(file).unwrap();
    mesh.add_smooth_normals(std::f64::consts::PI / 3.0, true);

    let file = std::fs::File::create("smooth_bunny.obj").unwrap();
    obj::write(&mesh, file).unwrap();
}
