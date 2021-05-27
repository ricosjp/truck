use super::*;
#[path = "../common/mod.rs"]
mod common;

#[test]
fn sphere() {
    let sphere0 = common::shapes::sphere(Point3::origin(), 10.0, 50, 50);
    let sphere1 = common::shapes::sphere(Point3::origin(), 11.0, 50, 50);
    assert!(sphere0.is_clung_to_by(&sphere1.positions(), 1.01));
    assert!(sphere1.is_clung_to_by(&sphere0.positions(), 1.01));
    assert!(!sphere0.is_clung_to_by(&sphere1.positions(), 0.99));
    assert!(!sphere1.is_clung_to_by(&sphere0.positions(), 0.99));
}

#[test]
fn tetrahedron() {
    let positions = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, -1.0, 1.0),
        Point3::new(1.0, -1.0, 0.0),
        Point3::new(-1.0, -1.0, 0.0),
    ];
    let tri_faces = vec![[0, 2, 1], [0, 3, 2], [0, 1, 3], [1, 2, 3]];
    let mesh0 = PolygonMesh::new(
        positions,
        Vec::new(),
        Vec::new(),
        Faces::from_iter(tri_faces),
    );
    let positions = vec![
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
    ];
    let tri_faces = vec![[0, 2, 1], [0, 3, 2], [0, 1, 3], [1, 2, 3]];
    let mesh1 = PolygonMesh::new(
        positions,
        Vec::new(),
        Vec::new(),
        Faces::from_iter(tri_faces),
    );
    assert!(mesh0.is_clung_to_by(mesh1.positions(), 1.0 + TOLERANCE));
    assert!(!mesh0.is_clung_to_by(mesh1.positions(), 1.0 - TOLERANCE));
}
