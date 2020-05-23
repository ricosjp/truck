extern crate truck_io as io;
use truck_polymesh::*;

fn main() {
    let file = std::fs::File::open("tests/data/sample.obj").unwrap();
    let mut mesh = io::obj::read(file).unwrap();
    mesh.uv_coords = Vec::new();
    mesh.normals = Vec::new();
    let mut handler = MeshHandler::new(mesh);
    handler.add_smooth_normal(std::f64::consts::PI / 3.0);

//    let (planes, others) = handler.extract_planes(0.01);
//    let planes = handler.create_mesh_by_face_indices(&planes);
//    let others = handler.create_mesh_by_face_indices(&others);
//    let planes_handler = MeshHandler::new(planes);
//    let others_handler = MeshHandler::new(others);
//    let planes_parts = planes_handler.clustering_face();
//    let others_parts = others_handler.clustering_face();
    let parts = handler.clustering_face();

    std::fs::DirBuilder::new().recursive(true).create("output").unwrap();
//    for (i, faces) in planes_parts.into_iter().enumerate() {
//        let mesh = planes_handler.create_mesh_by_face_indices(&faces);
//        let file = std::fs::File::create(&format!("output/planes_parts_{}.obj", i)).unwrap();
//        io::obj::write(&mesh, file).unwrap();
//    }
//    for (i, faces) in others_parts.into_iter().enumerate() {
//        let mesh = others_handler.create_mesh_by_face_indices(&faces);
//        let file = std::fs::File::create(&format!("output/others_parts_{}.obj", i)).unwrap();
//        io::obj::write(&mesh, file).unwrap();
//    }
    for (i, faces) in parts.into_iter().enumerate() {
        let mesh = handler.create_mesh_by_face_indices(&faces);
        let file = std::fs::File::create(&format!("output/parts_{}.obj", i)).unwrap();
        io::obj::write(&mesh, file).unwrap();
    }
}
