use truck_polymesh::*;

fn main() {
    let file = std::fs::File::open("tests/data/teapot.obj").unwrap();
    let mesh = obj::read(file).unwrap();
    let mut handler = MeshHandler::new(mesh);

    handler
        .put_together_same_attrs()
        .add_smooth_normal(std::f64::consts::PI / 3.0)
        .quadrangulate(0.1, 1.0);
    let file = std::fs::File::create("quaded_pot.obj").unwrap();
    obj::write(&handler.into(), file).unwrap()
}
