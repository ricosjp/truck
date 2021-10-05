use super::*;
use truck_polymesh::*;
use truck_meshalgo::filters::*;

#[test]
fn triangulate_test() {
    // cube consisting quad faces
    let positions = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
    ];
    let tri_faces = Faces::from_iter(&[
        &[3, 2, 1],
        &[3, 1, 0],
        &[0, 1, 5],
        &[0, 5, 4],
        &[1, 2, 6],
        &[1, 6, 5],
        &[2, 3, 7],
        &[2, 7, 6],
        &[3, 0, 4],
        &[3, 4, 7],
        &[4, 5, 6],
        &[4, 6, 7],
    ]);
    let quad_faces = Faces::from_iter(&[
        &[3, 2, 1, 0],
        &[0, 1, 5, 4],
        &[1, 2, 6, 5],
        &[2, 3, 7, 6],
        &[3, 0, 4, 7],
        &[4, 5, 6, 7],
    ]);
    let tri_mesh = PolygonMesh::new(positions.clone(), Vec::new(), Vec::new(), tri_faces);
    let mut quad_mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), quad_faces);
    quad_mesh.triangulate();
    assert_eq!(tri_mesh.faces(), quad_mesh.faces());
}

#[test]
fn quadrangulate_test() {
    // cube consisting quad faces
    let positions = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
    ];
    let tri_faces = Faces::from_iter(&[
        &[3, 2, 0],
        &[1, 0, 2],
        &[0, 1, 4],
        &[5, 4, 1],
        &[1, 2, 5],
        &[6, 5, 2],
        &[2, 3, 6],
        &[7, 6, 3],
        &[3, 0, 7],
        &[4, 7, 0],
        &[4, 5, 7],
        &[6, 7, 5],
    ]);
    let quad_faces = Faces::from_iter(&[
        &[3, 2, 1, 0],
        &[0, 1, 5, 4],
        &[1, 2, 6, 5],
        &[2, 3, 7, 6],
        &[3, 0, 4, 7],
        &[4, 5, 6, 7],
    ]);
    let mut tri_mesh = PolygonMesh::new(positions.clone(), Vec::new(), Vec::new(), tri_faces);
    let quad_mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), quad_faces);
    tri_mesh.quadrangulate(TOLERANCE, TOLERANCE);
    assert_eq!(tri_mesh.faces(), quad_mesh.faces());
}
