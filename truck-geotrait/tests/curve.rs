use truck_base::{cgmath64::*, tolerance::*};
use truck_geotrait::*;
mod polynomial;
use polynomial::PolyCurve;

#[test]
fn polycurve_test() {
    let coef = vec![
        Vector1::new(4.0),
        Vector1::new(3.0),
        Vector1::new(2.0),
        Vector1::new(1.0),
    ];
    let poly = PolyCurve::<Point1>(coef);
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
    let poly = PolyCurve::<Point2>(coef);
    let t = algo::curve::presearch(&poly, Point2::new(0.0, -1.0), poly.parameter_range(), 100);
    assert_eq!(t, 0.0);
}

fn exec_polycurve_snp_on_curve() -> bool {
    let coef: Vec<Vector3> = (0..5)
        .map(|_| {
            Vector3::new(
                20.0 * rand::random::<f64>() - 10.0,
                20.0 * rand::random::<f64>() - 10.0,
                20.0 * rand::random::<f64>() - 10.0,
            )
        })
        .collect();
    let poly = PolyCurve::<Point3>(coef);
    let t = 20.0 * rand::random::<f64>() - 10.0;
    let pt = poly.subs(t);
    let hint = t + 1.0 * rand::random::<f64>() - 0.5;
    match algo::curve::search_nearest_parameter(&poly, pt, hint, 100) {
        Some(res) => match poly.subs(res).near(&pt) {
            true => true,
            false => {
                eprintln!(
                    "wrong answer\npolynomial: {poly:?}\nt: {t:?}\nhint: {hint:?}\nresult: {res:?}",
                );
                false
            }
        },
        None => {
            eprintln!("not converge\npolynomial: {poly:?}\nt: {t:?}\nhint: {hint:?}");
            false
        }
    }
}

#[test]
fn polycurve_snp_on_curve() {
    let count = (0..100).filter(|_| exec_polycurve_snp_on_curve()).count();
    assert!(count > 90, "wrong answer: {:?}", 100 - count);
}

fn exec_polycurve_division() -> bool {
    let coef: Vec<Vector3> = (0..5)
        .map(|_| {
            Vector3::new(
                20.0 * rand::random::<f64>() - 10.0,
                20.0 * rand::random::<f64>() - 10.0,
                20.0 * rand::random::<f64>() - 10.0,
            )
        })
        .collect();
    let poly = PolyCurve::<Point3>(coef);
    let (division, pts) = algo::curve::parameter_division(&poly, (-10.0, 10.0), 0.05);
    division.windows(2).zip(pts).all(|(a, pt)| {
        let pt0 = poly.subs(a[0]);
        assert_eq!(pt0, pt);
        let pt1 = poly.subs(a[1]);
        (1..3).all(|i| {
            let t = i as f64 / 3.0;
            let res = pt0 + (pt1 - pt0) * t;
            let t = a[0] * (1.0 - t) + a[1] * t;
            let ans = poly.subs(t);
            res.distance(ans) < 0.1
        })
    })
}

#[test]
fn polycurve_division() {
    let count = (0..100).filter(|_| exec_polycurve_division()).count();
    println!("division error: {}", 100 - count);
    assert!(count > 98);
}
