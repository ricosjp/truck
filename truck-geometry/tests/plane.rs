#[test]
fn into_bspline() {
    use truck_geometry::prelude::*;
    let pt0 = Point3::new(0.0, 1.0, 2.0);
    let pt1 = Point3::new(1.0, 1.0, 3.0);
    let pt2 = Point3::new(0.0, 2.0, 3.0);
    let plane: Plane = Plane::new(pt0, pt1, pt2);
    let surface: BSplineSurface<Point3> = plane.into();
    assert_eq!(surface.range_tuple(), ((0.0, 1.0), (0.0, 1.0)));

    const N: usize = 100;
    for i in 0..=N {
        for j in 0..=N {
            let u = i as f64 / N as f64;
            let v = j as f64 / N as f64;
            let res = surface.subs(u, v);
            let ans = plane.subs(u, v);
            assert_near!(ans, res);
        }
    }
}
