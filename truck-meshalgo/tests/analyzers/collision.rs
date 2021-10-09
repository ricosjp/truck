use super::*;
#[path = "../common/mod.rs"]
mod common;

#[test]
fn sphere_interference() {
    let sphere0 = common::shapes::sphere(Point3::new(0.0, 0.0, -2.0), 1.0, 50, 50);
    let sphere1 = common::shapes::sphere(Point3::new(0.0, 0.0, 2.0), 1.0, 50, 50);
    assert!(sphere0.extract_interference(&sphere1).is_empty());
    let sphere0 = common::shapes::sphere(Point3::new(0.0, 0.0, -0.7), 1.0, 50, 50);
    let sphere1 = common::shapes::sphere(Point3::new(0.0, 0.0, 0.7), 1.0, 50, 50);
    let instant = std::time::Instant::now();
    let segs = sphere0.extract_interference(&sphere1);
    println!("extract interference: {}s", instant.elapsed().as_secs_f64());
    assert!(!segs.is_empty());
    let mut counter = Point3::origin();
    for (pt0, pt1) in segs {
        assert!(pt0[2].so_small());
        assert!(pt1[2].so_small());
        assert!(f64::abs(pt0.to_vec().magnitude2() - 0.51) < 0.05);
        assert!(f64::abs(pt1.to_vec().magnitude2() - 0.51) < 0.05);
        counter += if pt0.to_vec().cross(pt1.to_vec())[2] > 0.0 {
            pt1 - pt0
        } else {
            pt0 - pt1
        };
    }
    assert!(counter.near(&Point3::origin()));
}

#[test]
fn in_plane() {
    let positions: Vec<_> = (0..300)
        .map(|_| Point3::new(rand::random::<f64>(), 0.0, rand::random::<f64>()))
        .collect();
    let faces = Faces::from_iter((0..100).map(|i| [i, i + 100, i + 200]));
    let polygon0 = PolygonMesh::new(
        StandardAttributes {
            positions,
            ..Default::default()
        },
        faces.clone(),
    );
    let positions: Vec<_> = (0..300)
        .map(|_| Point3::new(rand::random::<f64>(), 0.0, rand::random::<f64>()))
        .collect();
    let polygon1 = PolygonMesh::new(
        StandardAttributes {
            positions,
            ..Default::default()
        },
        faces,
    );
    assert!(polygon0.collide_with(&polygon1).is_none());
}

#[test]
fn collision_sphere() {
    let unit = Vector3::new(
        rand::random::<f64>(),
        rand::random::<f64>(),
        rand::random::<f64>(),
    )
    .normalize();
    let instant = std::time::Instant::now();
    const N: usize = 10;
    for i in 0..N {
        let r = 10.0 * i as f64 / N as f64;
        let c0 = Point3::origin() + r * unit;
        let c1 = Point3::origin() - r * unit;
        let sphere0 = common::shapes::sphere(c0, 5.0, 50, 50);
        let sphere1 = common::shapes::sphere(c1, 5.0, 50, 50);
        if r < 5.0 {
            assert!(sphere0.collide_with(&sphere1).is_some());
        } else {
            assert!(sphere0.collide_with(&sphere1).is_none());
        }
    }
    println!(
        "10 times collision check: {}s",
        instant.elapsed().as_secs_f64()
    );
}
