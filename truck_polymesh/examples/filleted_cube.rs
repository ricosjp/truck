extern crate truck_io as io;
use truck_polymesh::*;

fn main() {
    let file = std::fs::File::open("tests/data/filleted_cube.obj").unwrap();
    let mesh = io::obj::read(file).unwrap();
    let mut handler = MeshHandler::new(mesh);
    handler
        .put_together_same_attrs()
        .quadrangulate(0.1)
        .add_smooth_normal(std::f64::consts::PI / 3.0);

    let (planes, others) = handler.extract_planes(0.01);
    let file = std::fs::File::create("planes.obj").unwrap();
    io::obj::write(&handler.create_mesh_by_face_indices(&planes), file).unwrap();

    let mesh = handler.create_mesh_by_face_indices(&others);
    let handler = MeshHandler::new(mesh);
    let (upper, lower) = handler.clustering_faces_by_gcurvature(0.1, false);
    let file = std::fs::File::create("lower.obj").unwrap();
    io::obj::write(&handler.create_mesh_by_face_indices(&lower), file).unwrap();
    let file = std::fs::File::create("upper.obj").unwrap();
    io::obj::write(&handler.create_mesh_by_face_indices(&upper), file).unwrap();
}
