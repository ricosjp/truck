use truck_polymesh::prelude::*;

fn main() {
    let file = std::fs::File::open("tests/data/teapot.obj").unwrap();
    let mut mesh = obj::read(file).unwrap();

    mesh
        .put_together_same_attrs()
        .add_smooth_normals(std::f64::consts::PI / 3.0, true)
        .quadrangulate(0.1, 1.0);
    let file = std::fs::File::create("quaded_pot.obj").unwrap();
    obj::write(&mesh, file).unwrap()
}
