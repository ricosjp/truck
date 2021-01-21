use truck_polymesh::prelude::*;

fn main() {
    let file = std::fs::File::open("examples/data/irregular_sphere.obj").unwrap();
    let mut mesh = obj::read(file).unwrap();
    mesh.put_together_same_attrs()
        .remove_degenerate_faces()
        .add_smooth_normals(std::f64::consts::PI / 6.0, true)
        .remove_unused_attrs();
    let file = std::fs::File::create("regular_sphere.obj").unwrap();
    obj::write(&mesh, file).unwrap();
}
