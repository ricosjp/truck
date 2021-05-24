//! An experiment to decompose a mesh into elements for future NURBS-shape approximation of the mesh.
//!
//! - Input: sample.obj
//! - Output: planes_parts_#.obj, others_parts_#.obj

use truck_polymesh::*;
use truck_meshalgo::filters::*;

fn main() {
    let file = std::fs::File::open("examples/data/sample.obj").unwrap();
    let mut mesh = obj::read(file).unwrap();
    mesh.add_smooth_normals(std::f64::consts::PI / 3.0, true);

    let (planes, others) = mesh.extract_planes(0.01);
    let planes = mesh.create_mesh_by_face_indices(&planes);
    let others = mesh.create_mesh_by_face_indices(&others);
    let planes_parts = planes.into_components();
    let others_parts = others.into_components();

    std::fs::DirBuilder::new()
        .recursive(true)
        .create("output")
        .unwrap();
    for (i, faces) in planes_parts.into_iter().enumerate() {
        let mesh = planes.create_mesh_by_face_indices(&faces);
        let file = std::fs::File::create(&format!("output/planes_parts_{}.obj", i)).unwrap();
        obj::write(&mesh, file).unwrap();
    }
    for (i, faces) in others_parts.into_iter().enumerate() {
        let mesh = others.create_mesh_by_face_indices(&faces);
        let file = std::fs::File::create(&format!("output/others_parts_{}.obj", i)).unwrap();
        obj::write(&mesh, file).unwrap();
    }
}
