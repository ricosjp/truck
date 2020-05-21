extern crate truck_io as io;
use truck_polymesh::*;

fn main() {
    let file = std::fs::File::open("tests/data/sample.obj").unwrap();
    let mut mesh = io::obj::read(file).unwrap();
    mesh.uv_coords = Vec::new();
    mesh.normals = Vec::new();
    let mut handler = MeshHandler::new(mesh);
    handler.add_smooth_normal(std::f64::consts::PI / 6.0);
    let parts = handler.clustering_face();

    for (i, faces) in parts.into_iter().enumerate() {
        let mesh = handler.create_mesh_by_face_indices(&faces);
        let file = std::fs::File::create(&format!("output0/parts_{}.obj", i)).unwrap();
        io::obj::write(&mesh, file).unwrap()
    }
}
