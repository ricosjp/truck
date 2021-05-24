//! Add the normal to the sphere containing
//! the irregular normal generated from the NURBS containing the critical point.
//!
//! - Input: irregular_sphere.obj
//! - Output: regular_sphere.obj

use truck_polymesh::*;
use truck_meshalgo::filters::*;

fn main() {
    let file = std::fs::File::open("examples/data/irregular_sphere.obj").unwrap();
    let mut mesh = obj::read(file).unwrap();
    mesh.normalize_normals()
        .remove_unused_attrs()
        .put_together_same_attrs()
        .remove_degenerate_faces()
        .add_smooth_normals(std::f64::consts::PI / 6.0, true)
        .remove_unused_attrs();
    let file = std::fs::File::create("regular_sphere.obj").unwrap();
    obj::write(&mesh, file).unwrap();
}
