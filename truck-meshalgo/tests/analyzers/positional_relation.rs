use super::*;
#[path = "../common/mod.rs"]
mod common;

#[test]
fn sphere_collision() {
    let sphere0 = common::shapes::sphere(Point3::new(0.0, 0.0, -2.0), 1.0, 50, 50);
    let sphere1 = common::shapes::sphere(Point3::new(0.0, 0.0, 2.0), 1.0, 50, 50);
    assert!(sphere0.collision(&sphere1).is_empty());
    let sphere0 = common::shapes::sphere(Point3::new(0.0, 0.0, -0.7), 1.0, 50, 50);
    let sphere1 = common::shapes::sphere(Point3::new(0.0, 0.0, 0.7), 1.0, 50, 50);
    let instant = std::time::Instant::now();
    let segs = sphere0.collision(&sphere1);
    println!("collision: {}s", instant.elapsed().as_secs_f64());
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
