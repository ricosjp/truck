use proptest::prelude::*;
use truck_geotrait::algo::DefaultSplitParams;
use std::f64::consts::PI;
use truck_geometry::prelude::*;

proptest! {
    #[test]
    fn search_parameter(t in 0f64..=(2.0 * PI)) {
        let circle = UnitCircle::<Point2>::new();
        let p = circle.subs(t);
        let s = circle.search_nearest_parameter(p, None, 1).unwrap();
        prop_assert_near!(s, t);
    }
    #[test]
    fn search_nearest_parameter(t in 0f64..=(2.0 * PI), a in 0.1f64..=5f64) {
        let circle = UnitCircle::<Point2>::new();
        let p = a * circle.subs(t);
        let s = circle.search_nearest_parameter(p, None, 1).unwrap();
        let q = a * circle.subs(s);
        prop_assert_near!(p, q);
    }

    #[test]
    fn to_nurbs(t0 in 0f64..=PI, t1 in PI..=(2.0 * PI)) {
        let circle = UnitCircle::<Point2>::new();
        let arc = TrimmedCurve::new(circle, (t0, t1));
        let bsp: NurbsCurve<_> = arc.to_same_geometry();
        prop_assert_near!(bsp.front(), arc.front());
        prop_assert_near!(bsp.back(), arc.back());
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let p = bsp.subs(t).to_vec();
            let der = bsp.der(t);
            prop_assert_near!(p.magnitude2(), 1.0);
            prop_assert!(der.dot(p).so_small());
        }
    }
}

#[test]
fn parameter_division() {
    let c = UnitCircle::<Point2>::new();
    let (_div, pts) = c.parameter_division(c.range_tuple(), DefaultSplitParams::new(0.05));
    for a in pts.windows(2) {
        let p = a[0].midpoint(a[1]);
        assert!(p.to_vec().magnitude() > 0.95);
    }
}
