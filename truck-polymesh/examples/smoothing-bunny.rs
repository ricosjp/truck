use truck_polymesh::prelude::*;

fn main() {
    let file = std::fs::File::open("tests/data/bunny.obj").unwrap();
    let mut mesh = obj::read(file).unwrap();
    mesh.add_smooth_normals(std::f64::consts::PI / 3.0, true);

    let file = std::fs::File::create("smooth_bunny.obj").unwrap();
    obj::write(&mesh, file).unwrap();
}
