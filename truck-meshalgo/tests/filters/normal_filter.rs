use truck_meshalgo::filters::*;
use truck_polymesh::*;
#[path = "../common/mod.rs"]
mod common;

#[test]
fn normalize_normals_test() {
    let mut mesh = PolygonMesh::new(
        StandardAttributes {
            positions: vec![Point3::new(0.0, 0.0, 0.0)],
            normals: vec![
                Vector3::new(100.0, 20.0, 56.0),
                Vector3::new(1.0e-12, 3.536e10, std::f64::NAN),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            ],
            ..Default::default()
        },
        Faces::from_iter(&[
            [(0, None, Some(0)), (0, None, Some(1)), (0, None, Some(2))].as_ref(),
            &[
                (0, None, Some(0)),
                (0, None, Some(1)),
                (0, None, Some(2)),
                (0, None, Some(3)),
            ],
            &[
                (0, None, Some(0)),
                (0, None, Some(1)),
                (0, None, Some(2)),
                (0, None, Some(3)),
                (0, None, Some(3)),
            ],
        ]),
    );
    mesh.normalize_normals();

    assert!(mesh.normals()[0].magnitude().near(&1.0));
    assert!(!mesh.normals()[1].magnitude().near(&1.0));
    assert!(!mesh.normals()[2].magnitude().near(&1.0));
    assert!(mesh.normals()[3].magnitude().near(&1.0));

    let mut iter = mesh.face_iter();
    let face = iter.next().unwrap();
    assert_eq!(face[0].nor, Some(0));
    assert_eq!(face[1].nor, None);
    assert_eq!(face[2].nor, None);
    let face = iter.next().unwrap();
    assert_eq!(face[0].nor, Some(0));
    assert_eq!(face[1].nor, None);
    assert_eq!(face[2].nor, None);
    assert_eq!(face[3].nor, Some(3));
    let face = iter.next().unwrap();
    assert_eq!(face[0].nor, Some(0));
    assert_eq!(face[1].nor, None);
    assert_eq!(face[2].nor, None);
    assert_eq!(face[3].nor, Some(3));
    assert_eq!(face[4].nor, Some(3));
}

#[test]
fn add_naive_normals_test() {
    let mut mesh = PolygonMesh::new(
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
            ..Default::default()
        },
        Faces::from_iter(&[[0, 1, 3, 2].as_ref(), &[1, 4, 3], &[2, 3, 5], &[3, 4, 6, 5]]),
    );

    // test of overwrite flag
    mesh.add_smooth_normals(1.0, true);
    let normals = mesh.normals().clone();
    mesh.add_naive_normals(false);
    assert_eq!(&normals, mesh.normals());

    // overwrite!
    mesh.add_naive_normals(true);

    let tri_faces = mesh.faces().tri_faces();
    let quad_faces = mesh.faces().quad_faces();
    for (i, v) in tri_faces.iter().flatten().enumerate() {
        if i < 3 {
            assert!(mesh.normals()[v.nor.unwrap()].near(&Vector3::new(-2.0, 5.0, 0.0).normalize()));
        } else {
            assert!(mesh.normals()[v.nor.unwrap()].near(&Vector3::new(2.0, 5.0, 0.0).normalize()));
        }
    }
    for (i, v) in quad_faces.iter().flatten().enumerate() {
        if i < 4 {
            assert!(mesh.normals()[v.nor.unwrap()].near(&Vector3::new(-2.0, 5.0, 0.0).normalize()));
        } else {
            assert!(mesh.normals()[v.nor.unwrap()].near(&Vector3::new(2.0, 5.0, 0.0).normalize()));
        }
    }
}

#[test]
fn add_smooth_normals() {
    let mut sphere = common::shapes::sphere(Point3::origin(), 1.0, 20, 10);
    sphere
        .put_together_same_attrs(TOLERANCE)
        .remove_degenerate_faces()
        .remove_unused_attrs();

    let mut naive_sphere = sphere.clone();
    naive_sphere.add_naive_normals(true);
    let naive_normals = naive_sphere.normals().clone();

    // variable overwrite
    naive_sphere.add_smooth_normals(1.0, false);
    // strict equals
    assert_eq!(&naive_normals, naive_sphere.normals());

    sphere.add_smooth_normals(1.0, true);

    let iter0 = naive_sphere.faces().face_iter().flatten();
    let iter1 = sphere.faces().face_iter().flatten();
    for (v0, v1) in iter0.zip(iter1) {
        let p0 = naive_sphere.positions()[v0.pos].to_vec();
        let n0 = naive_sphere.normals()[v0.nor.unwrap()];
        let p1 = sphere.positions()[v1.pos].to_vec();
        let n1 = sphere.normals()[v1.nor.unwrap()];
        assert_eq!(v0.pos, v1.pos);
        assert_eq!(p0, p1);
        // Smooth normal must be nearer truth normal than naive normal.
        assert!(p0.distance(n0) > p1.distance(n1));
    }
}
