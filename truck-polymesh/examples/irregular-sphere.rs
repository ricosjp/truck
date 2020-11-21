use truck_polymesh::*;

fn main() {
    let file = std::fs::File::open("tests/data/irregular_sphere.obj").unwrap();
    let mut mesh = obj::read(file).unwrap();
    mesh.normals = Vec::new();
    let mut handler = MeshHandler::new(mesh);
    handler
        .put_together_same_attrs()
        .remove_degenerate_faces()
        .add_smooth_normal(std::f64::consts::PI / 6.0)
        .remove_unused_attrs();
    let file = std::fs::File::create("regular_sphere.obj").unwrap();
    obj::write(&handler.into(), file).unwrap();
}
