use proptest::{prelude::*, property_test};
use std::f64::consts::{PI, TAU};
use truck_geometry::prelude::*;

#[property_test]
fn search_parameter(#[strategy = 0.0..=TAU] t: f64) {
    let circle = UnitCircle::<Point2>::new();
    let p = circle.subs(t);
    let s = circle.search_nearest_parameter(p, None, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn search_nearest_parameter(#[strategy = 0.0..=TAU] t: f64, #[strategy = 0.1..=5f64] a: f64) {
    let circle = UnitCircle::<Point2>::new();
    let p = a * circle.subs(t);
    let s = circle.search_nearest_parameter(p, None, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn search_parameter_with_parameter_hint(
    #[strategy = -100.0..=100.0] t: f64,
    #[strategy = -0.2..=0.2] d: f64,
) {
    let circle = UnitCircle::<Point2>::new();
    let p = circle.subs(t);
    let s = circle.search_nearest_parameter(p, t + d, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn search_nearest_parameter_with_parameter_hint(
    #[strategy = -100.0..=100.0] t: f64,
    #[strategy = -0.2..=0.2] d: f64,
    #[strategy = 0.1..=5.0] a: f64,
) {
    let circle = UnitCircle::<Point2>::new();
    let p = a * circle.subs(t);
    let s = circle.search_nearest_parameter(p, t + d, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn search_parameter_with_range(
    #[strategy = -100.0..=100.0] start: f64,
    #[strategy = 0.01..=PI] length: f64,
    #[strategy = -0.1..=1.1] lerp_ratio: f64,
) {
    let circle = UnitCircle::<Point2>::new();
    let range = (start, start + length);
    let t = start + lerp_ratio * length;
    let p = circle.subs(t);
    let s = circle.search_nearest_parameter(p, range, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn search_nearest_parameter_with_range(
    #[strategy = -100.0..=100.0] start: f64,
    #[strategy = 0.01..=PI] length: f64,
    #[strategy = -0.1..=1.1] lerp_ratio: f64,
    #[strategy = 0.1..=5.0] a: f64,
) {
    let circle = UnitCircle::<Point2>::new();
    let range = (start, start + length);
    let t = start + lerp_ratio * length;
    let p = a * circle.subs(t);
    let s = circle.search_nearest_parameter(p, range, 1).unwrap();
    prop_assert_near2!(s, t);
}

#[property_test]
fn to_nurbs(#[strategy = 0.0..=PI] t0: f64, #[strategy = PI..=TAU] t1: f64) {
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

#[test]
fn parameter_division() {
    let c = UnitCircle::<Point2>::new();
    let (_div, pts) = c.parameter_division(c.range_tuple(), 0.05);
    for a in pts.windows(2) {
        let p = a[0].midpoint(a[1]);
        assert!(p.to_vec().magnitude() > 0.95);
    }
}
