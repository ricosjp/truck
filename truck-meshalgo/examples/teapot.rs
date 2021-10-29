//! Adds smooth normals to and quadrangulate the famous teapot.
//!
//! - Input: teapot.obj
//! - Output: quaded_pot.obj

use truck_meshalgo::filters::*;
use truck_polymesh::*;

fn main() {
    const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/obj/teapot.obj",);
    std::fs::copy(PATH, "teapot.obj").unwrap();
    let file = std::fs::File::open(PATH).unwrap();
    let mut mesh = obj::read(file).unwrap();

    mesh.put_together_same_attrs()
        .add_smooth_normals(std::f64::consts::PI / 3.0, true)
        .quadrangulate(0.1, 1.0);
    let file = std::fs::File::create("quaded_pot.obj").unwrap();
    obj::write(&mesh, file).unwrap()
}
