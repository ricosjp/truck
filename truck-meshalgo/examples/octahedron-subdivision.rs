//! Apply loop subdivision to regular octahedron.
//! 
//! - Input: hardcoded octahedron
//! - Output: octahedron.obj, subdivision-octahedron.obj

use truck_meshalgo::prelude::*;

fn main() {
    let mut polymesh = PolygonMesh::new(
        StandardAttributes {
            positions: vec![
                Point3::new(-1.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(0.0, -1.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
                Point3::new(0.0, 0.0, -1.0),
                Point3::new(0.0, 0.0, 1.0),
            ],
            ..Default::default()
        },
        Faces::from_tri_and_quad_faces(
            vec![
                [0.into(), 4.into(), 2.into()],
                [0.into(), 2.into(), 5.into()],
                [1.into(), 5.into(), 2.into()],
                [1.into(), 2.into(), 4.into()],
                [0.into(), 3.into(), 4.into()],
                [0.into(), 5.into(), 3.into()],
                [1.into(), 3.into(), 5.into()],
                [1.into(), 4.into(), 3.into()],
            ],
            Vec::new(),
        ),
    );
    let mut buf = Vec::<u8>::new();
    obj::write(&polymesh, &mut buf).unwrap();
    std::fs::write("octahedron.obj", &buf).unwrap();
    polymesh
        .loop_subdivision()
        .loop_subdivision()
        .loop_subdivision()
        .loop_subdivision()
        .loop_subdivision()
        .loop_subdivision()
        .add_smooth_normals(std::f64::consts::PI / 3.0, true);
    buf.clear();
    obj::write(&polymesh, &mut buf).unwrap();
    std::fs::write("subdivision-octahedron.obj", &buf).unwrap();
}
