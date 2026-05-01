#![cfg(feature = "polynomial")]

use proptest::{prelude::*, property_test};
use truck_base::{cgmath64::*, tolerance::*};
use truck_geotrait::{polynomial::PolynomialCurve, *};

#[test]
fn polycurve_test() {
    let coef = vec![
        Vector1::new(4.0),
        Vector1::new(3.0),
        Vector1::new(2.0),
        Vector1::new(1.0),
    ];
    let poly = PolynomialCurve::<Point1>(coef);
    for i in 0..10 {
        let t = i as f64;
        let res = poly.subs(t);
        let ans = Point1::new(t * t * t + 2.0 * t * t + 3.0 * t + 4.0);
        assert_eq!(res, ans);
        let res = poly.der(t);
        let ans = Vector1::new(3.0 * t * t + 4.0 * t + 3.0);
        assert_eq!(res, ans);
        let res = poly.der2(t);
        let ans = Vector1::new(6.0 * t + 4.0);
        assert_eq!(res, ans);
    }
}

#[test]
fn polycurve_presearch() {
    let coef = vec![
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, -2.0),
        Vector2::new(0.0, 1.0),
    ];
    let poly = PolynomialCurve::<Point2>(coef);
    let t = algo::curve::presearch(&poly, Point2::new(0.0, -1.0), poly.range_tuple(), 100);
    assert_eq!(t, 0.0);
}

#[property_test]
fn polycurve_snp_on_curve(
    #[strategy = prop::array::uniform4(-0.5f64..=0.5f64)] a: [f64; 4],
    #[strategy = -1.0f64..=1.0f64] t: f64,
    #[strategy = -0.2f64..=0.2f64] hint_offset: f64,
) {
    let coef = vec![
        Vector3::new(0.0, a[0], a[1]),
        Vector3::new(1.0, a[1], a[2]),
        Vector3::new(0.0, a[2], a[3]),
        Vector3::new(0.0, a[3], a[0]),
    ];
    let poly = PolynomialCurve::<Point3>(coef);
    let pt = poly.subs(t);
    let hint = t + hint_offset;
    let res = algo::curve::search_nearest_parameter(&poly, pt, hint, 100);
    prop_assert!(res.is_some());
    prop_assert!(poly.subs(res.unwrap()).near(&pt));
}

#[property_test]
fn polycurve_division(#[strategy = prop::array::uniform15(-10.0f64..=10.0f64)] a: [f64; 15]) {
    let coef: Vec<Vector3> = a
        .chunks_exact(3)
        .map(|a| Vector3::new(a[0], a[1], a[2]))
        .collect();
    let poly = PolynomialCurve::<Point3>(coef);
    let (division, pts) = algo::curve::parameter_division(&poly, (-10.0, 10.0), 0.05);
    for (a, pt) in division.windows(2).zip(pts) {
        let pt0 = poly.subs(a[0]);
        prop_assert_eq!(pt0, pt);
        let pt1 = poly.subs(a[1]);
        for i in 1..3 {
            let t = i as f64 / 3.0;
            let res = pt0 + (pt1 - pt0) * t;
            let t = a[0] * (1.0 - t) + a[1] * t;
            let ans = poly.subs(t);
            prop_assert!(res.distance(ans) < 0.1);
        }
    }
}

#[property_test]
fn polycurve_closest_point(#[strategy = prop::array::uniform4(-0.5f64..=0.5f64)] a: [f64; 4]) {
    let coef0 = vec![
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(2.0 * a[0] + 1.0, 2.0 * a[1] + 1.0, 0.0),
        Vector3::new(-2.0 * a[0], -2.0 * a[1], 0.0),
    ];
    let coef1 = vec![
        Vector3::new(1.0, 0.0, 1.0),
        Vector3::new(2.0 * a[2] - 2.0, 2.0 * a[3], 0.0),
        Vector3::new(1.0 - 2.0 * a[2], 1.0 - 2.0 * a[3], 0.0),
    ];
    let poly0 = PolynomialCurve::<Point3>(coef0);
    let poly1 = PolynomialCurve::<Point3>(coef1);
    let res = algo::curve::search_closest_parameter(&poly0, &poly1, (0.5, 0.5), 100);
    prop_assert!(res.is_some());
    let (t0, t1) = res.unwrap();

    let (p0, der0) = (poly0.subs(t0), poly0.der(t0));
    let (p1, der1) = (poly1.subs(t1), poly1.der(t1));
    let n = p1 - p0;

    prop_assert!(n.dot(der0).so_small() && n.dot(der1).so_small());
}

#[property_test]
fn polycurve_intersection_point(
    #[strategy = 0.1f64..=0.6f64] a0: f64,
    #[strategy = 0.1f64..=0.6f64] a1: f64,
    #[strategy = -0.5f64..=0.5f64] a2: f64,
) {
    let a = [a0, a1, a2];
    let (x, y) = (-1.0 + a[0], 1.0 - a[1]);
    let coef0 = vec![
        Vector2::new(-1.0, 1.0),
        Vector2::new(x + 3.0, -2.0),
        Vector2::new(-2.0 * x + y - 3.0, 0.0),
        Vector2::new(x - y + 2.0, 0.0),
    ];
    let coef1 = vec![
        Vector2::new(-1.0, 0.0),
        Vector2::new(2.0, 4.0 * a[2]),
        Vector2::new(0.0, -4.0 * a[2]),
    ];
    let poly0 = PolynomialCurve::<Point2>(coef0);
    let poly1 = PolynomialCurve::<Point2>(coef1);
    let res = algo::curve::search_intersection_parameter(&poly0, &poly1, (0.5, 0.5), 100);
    prop_assert!(res.is_some());
    let (t0, t1) = res.unwrap();

    let (p0, p1) = (poly0.subs(t0), poly1.subs(t1));
    prop_assert!(poly0.subs(t0).near(&p0) && poly1.subs(t1).near(&p1) && p0.near(&p1));
}
