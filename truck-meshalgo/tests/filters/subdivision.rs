use truck_meshalgo::filters::*;
use truck_polymesh::*;

#[test]
fn loop_subdivision() {
    let positions = vec![
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, -1.0),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let tri_faces = vec![
        [0.into(), 1.into(), 2.into()],
        [3.into(), 2.into(), 1.into()],
    ];
    let mut polygon = PolygonMesh::new(
        StandardAttributes {
            positions,
            uv_coords: Vec::new(),
            normals: Vec::new(),
        },
        Faces::from_tri_and_quad_faces(tri_faces, Vec::new()),
    );
    polygon.loop_subdivision();
    assert_eq!(polygon.positions().len(), 9);
    assert_eq!(
        polygon.positions(),
        &[
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(0.0, 6.0, 6.0) / 8.0,
            Point3::new(0.0, 6.0, -6.0) / 8.0,
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 0.75, 0.0),
            Point3::new(-0.5, 0.5, -0.5),
            Point3::new(-0.5, 0.5, 0.5),
            Point3::new(0.5, 0.5, 0.5),
            Point3::new(0.5, 0.5, -0.5),
        ],
    );
}
