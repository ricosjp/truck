extern crate truck_io as io;
use truck_polymesh::*;

fn main() {
    let file = std::fs::File::open("tests/data/bunny.obj").unwrap();
    let mesh = io::obj::read(file).unwrap();
    let mut handler = MeshHandler::new(mesh);

    handler
        .put_together_same_attrs()
        .add_smooth_normal(std::f64::consts::PI / 3.0)
        .quadrangulate(0.1);
    let file = std::fs::File::create("quaded_pot.obj").unwrap();
    io::obj::write(&handler.into(), file).unwrap()
}
