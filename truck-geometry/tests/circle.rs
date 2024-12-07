use proptest::prelude::*;
use std::f64::consts::PI;
use truck_geometry::prelude::*;

proptest! {
    #[test]
    fn search_parameter(t in 0f64..=(2.0 * PI)) {
        let circle = UnitCircle::<Point2>::new();
        let p = circle.subs(t);
        let s = circle.search_nearest_parameter(p, None, 1).unwrap();
        assert_near!(s, t);
    }
    #[test]
    fn search_nearest_parameter(t in 0f64..=(2.0 * PI), a in 0.1f64..=5f64) {
        let circle = UnitCircle::<Point2>::new();
        let p = a * circle.subs(t);
        let s = circle.search_nearest_parameter(p, None, 1).unwrap();
        let q = a * circle.subs(s);
        assert_near!(p, q);
    }
}

#[test]
fn parameter_division() {
    let c = UnitCircle::<Point2>::new();
    let (_div, pts) = c.parameter_division(c.range_tuple(), 0.05);
    for a in pts.windows(2) {
        let p = a[0].midpoint(a[1]);
        assert!(p.to_vec().magnitude() > 0.95);
    }
}
