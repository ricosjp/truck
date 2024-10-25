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
    let t = algo::curve::presearch(&poly, Point2::new(0.0, -1.0), poly.range_tuple(), 100);
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

fn exec_polycurve_closest_point() -> bool {
    let a = [
        1.0 * rand::random::<f64>() - 0.5,
        1.0 * rand::random::<f64>() - 0.5,
        1.0 * rand::random::<f64>() - 0.5,
        1.0 * rand::random::<f64>() - 0.5,
    ];
    let coef0 = vec![
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(4.0 * a[0] - 1.0, 4.0 * a[1] - 1.0, 0.0),
        Vector3::new(2.0 - 4.0 * a[0], 2.0 - 4.0 * a[1], 0.0),
    ];
    let coef1 = vec![
        Vector3::new(1.0, 0.0, 1.0),
        Vector3::new(4.0 * a[2] - 3.0, 4.0 * a[3] - 1.0, 0.0),
        Vector3::new(2.0 - 4.0 * a[2], 2.0 - 4.0 * a[3], 0.0),
    ];
    let poly0 = PolyCurve::<Point3>(coef0);
    let poly1 = PolyCurve::<Point3>(coef1);
    let res = algo::curve::search_closest_parameter(&poly0, &poly1, (0.5, 0.5), 100);
    let (t0, t1) = match res {
        Some(res) => res,
        None => return false,
    };

    let (p0, der0) = (poly0.subs(t0), poly0.der(t0));
    let (p1, der1) = (poly1.subs(t1), poly1.der(t1));
    let n = p1 - p0;

    n.dot(der0).so_small() && n.dot(der1).so_small()
}

#[test]
fn polycurve_closest_point() {
    let count = (0..10).filter(|_| exec_polycurve_closest_point()).count();
    println!("searching closest point error: {}", 10 - count);
    assert!(count >= 8);
}

fn exec_polycurve_intersection_point() -> bool {
    let a = [
        0.5 * rand::random::<f64>() + 0.1,
        0.5 * rand::random::<f64>() + 0.1,
        1.0 * rand::random::<f64>() - 0.5,
    ];
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
    let poly0 = PolyCurve::<Point2>(coef0);
    let poly1 = PolyCurve::<Point2>(coef1);
    let res = algo::curve::search_intersection_parameter2d(&poly0, &poly1, (0.5, 0.5), 100);
    let (t0, t1) = match res {
        Some(res) => res,
        None => return false,
    };

    let (p0, p1) = (poly0.subs(t0), poly1.subs(t1));
    poly0.subs(t0).near(&p0)
    && poly1.subs(t1).near(&p1)
    && p0.near(&p1)
}

#[test]
fn polycurve_intersection_point() {
    let count = (0..10).filter(|_| exec_polycurve_intersection_point()).count();
    println!("searching intersection point error: {}", 10 - count);
    assert!(count >= 8);
}
