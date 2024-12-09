use proptest::prelude::*;
use truck_geometry::prelude::*;

proptest! {
    #[test]
    fn sp_test(t in -50f64..=50f64) {
        let curve = UnitHyperbola::<Point2>::new();
        let p = curve.subs(t);
        assert_near!(curve.search_parameter(p, None, 0).unwrap(), t);
    }
}

#[test]
fn snp_test() {
    let (t, r) = (2.0, 1.0);
    let curve = UnitHyperbola::<Point2>::new();
    let p = curve.subs(t);
    let q = p + r * Vector2::new(-p.x, p.y);
    let t = curve.search_nearest_parameter(q, None, 0).unwrap();
    let p = curve.subs(t);
    let dot = curve.der(t).dot(q - p);
    assert!(dot.so_small(), "{t} {dot}");
}

#[test]
fn sp_negative_test() {
    let curve = UnitHyperbola::<Point2>::new();
    let q = Point2::new(-1.0, 0.0);
    assert!(curve.search_parameter(q, None, 0).is_none());
}
