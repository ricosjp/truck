extern crate truck_io as io;
use truck_polymesh::*;

fn main() {
    let mesh = io::obj::read(&"tests/data/sample.obj").unwrap();
    let mut handler = MeshHandler::new(mesh);
    handler.add_smooth_normal(std::f64::consts::PI / 6.0);
    let parts = handler.clustering_face();
    let mesh: PolygonMesh = handler.into();

    for (i, faces) in parts.into_iter().enumerate() {
        let mut new_mesh = mesh.clone();
        new_mesh.tri_faces = faces.iter().map(|i| mesh.tri_faces[*i].clone()).collect();
        let mut handler = MeshHandler::new(new_mesh);
        handler.remove_unused_attrs();
        let file = std::fs::File::create(&format!("parts_{}.obj", i)).unwrap();
        io::obj::write(&handler.into(), file).unwrap()
    }
}