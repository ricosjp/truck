use super::*;

#[test]
fn extract_planes_test() {
    let mesh = PolygonMesh::new(
        StandardAttributes {
            positions: vec![
                Point3::new(-5.0, 0.0, -2.0),
                Point3::new(-5.0, 0.0, 0.0),
                Point3::new(0.0, 2.0, -2.0),
                Point3::new(0.0, 2.0, 0.0),
                Point3::new(0.0, 2.0, 2.0),
                Point3::new(5.0, 0.0, 0.0),
                Point3::new(5.0, 0.0, 2.0),
            ],
            normals: vec![
                Vector3::new(-2.0, 5.0, 0.0).normalize(),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(2.0, 5.0, 0.0).normalize(),
            ],
            ..Default::default()
        },
        Faces::from_iter(&[
            [
                (0, None, Some(0)),
                (1, None, Some(0)),
                (3, None, Some(1)),
                (2, None, Some(1)),
            ]
            .as_ref(),
            &[(1, None, Some(1)), (4, None, Some(1)), (3, None, Some(1))],
            &[(2, None, Some(1)), (3, None, Some(1)), (5, None, Some(2))],
            &[
                (3, None, Some(1)),
                (4, None, Some(1)),
                (6, None, Some(1)),
                (5, None, Some(1)),
            ],
        ]),
    );
    let (planes, others) = mesh.extract_planes(TOLERANCE);
    assert_eq!(planes, vec![1, 2]);
    assert_eq!(others, vec![0, 3]);
}

#[test]
fn into_component_test() {
    // cube consisting tri_faces
    let mut mesh = PolygonMesh::new(
        StandardAttributes {
            positions: vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(1.0, 1.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
                Point3::new(0.0, 0.0, 1.0),
                Point3::new(1.0, 0.0, 1.0),
                Point3::new(1.0, 1.0, 1.0),
                Point3::new(0.0, 1.0, 1.0),
            ],
            ..Default::default()
        },
        Faces::from_iter(&[
            [3, 2, 1, 0].as_ref(),
            &[0, 1, 4],
            &[5, 4, 1],
            &[1, 2, 5],
            &[6, 5, 2],
            &[2, 3, 7, 6],
            &[3, 0, 7],
            &[4, 7, 0],
            &[4, 5, 7],
            &[6, 7, 5],
        ]),
    );

    // sign up normals
    mesh.add_naive_normals(true).put_together_same_attrs(TOLERANCE);

    let components = mesh.components(true);
    // The number of components is six because the mesh is a cube.
    assert_eq!(components.len(), 6);
    assert_eq!(components[0], vec![0, 1]);
    assert_eq!(components[1], vec![2, 3]);
    assert_eq!(components[2], vec![4, 5]);
    assert_eq!(components[3], vec![6, 7]);
    assert_eq!(components[4], vec![8]);
    assert_eq!(components[5], vec![9]);

    let components = mesh.components(false);
    assert_eq!(components.len(), 1);
}
