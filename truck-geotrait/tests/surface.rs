use algo::surface;
use truck_base::{cgmath64::*, tolerance::*};
use truck_geotrait::*;
mod polynomial;
use polynomial::{PolyCurve, PolySurface};

#[test]
fn polysurface() {
    let coef0 = vec![
        Vector3::new(1.0, 0.0, 1.0),
        Vector3::new(3.0, 1.0, 0.0),
        Vector3::new(2.0, 0.0, 0.0),
    ];
    let coef1 = vec![
        Vector3::new(2.0, 1.0, 0.0),
        Vector3::new(-6.0, 0.0, 1.0),
        Vector3::new(4.0, 0.0, 0.0),
    ];
    let poly = PolySurface(PolyCurve(coef0), PolyCurve(coef1));
    for i in 0..5 {
        let u = i as f64;
        for j in 0..5 {
            let v = j as f64;
            assert_eq!(
                poly.subs(u, v)[0],
                (2.0 * u * u + 3.0 * u + 1.0) * (4.0 * v * v - 6.0 * v + 2.0)
            );
            assert_eq!(
                poly.uder(u, v)[0],
                (4.0 * u + 3.0) * (4.0 * v * v - 6.0 * v + 2.0)
            );
            assert_eq!(
                poly.vder(u, v)[0],
                (2.0 * u * u + 3.0 * u + 1.0) * (8.0 * v - 6.0)
            );
            assert_eq!(poly.uuder(u, v)[0], 4.0 * (4.0 * v * v - 6.0 * v + 2.0));
            assert_eq!(poly.uvder(u, v)[0], (4.0 * u + 3.0) * (8.0 * v - 6.0));
            assert_eq!(poly.vvder(u, v)[0], (2.0 * u * u + 3.0 * u + 1.0) * 8.0);
            assert!(poly.normal(u, v).dot(poly.uder(u, v)).so_small());
            assert!(poly.normal(u, v).dot(poly.vder(u, v)).so_small());
        }
    }
}

#[test]
fn polysurface_presearch() {
    let coef0 = vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 0.0, 0.0)];
    let coef1 = vec![Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)];
    let poly = PolySurface(PolyCurve(coef0), PolyCurve(coef1));
    let pt = Point3::new(0.2, 0.3, 1.0);
    let (u, v) = algo::surface::presearch(&poly, pt, ((0.0, 1.0), (0.0, 1.0)), 100);
    assert_eq!(u, 0.2);
    assert_eq!(v, 0.3);
}

fn exec_polysurface_snp_on_surface() -> bool {
    let coef0 = vec![
        Vector3::new(0.0, 1.0, 3.0 * rand::random::<f64>() - 1.5),
        Vector3::new(1.0, 0.0, 3.0 * rand::random::<f64>() - 1.5),
        Vector3::new(0.0, 0.0, 3.0 * rand::random::<f64>() - 1.5),
    ];
    let coef1 = vec![
        Vector3::new(1.0, 0.0, 3.0 * rand::random::<f64>() - 1.5),
        Vector3::new(0.0, 1.0, 3.0 * rand::random::<f64>() - 1.5),
        Vector3::new(0.0, 0.0, 3.0 * rand::random::<f64>() - 1.5),
    ];
    let poly = PolySurface(PolyCurve(coef0), PolyCurve(coef1));
    let u = 10.0 * rand::random::<f64>() - 5.0;
    let v = 10.0 * rand::random::<f64>() - 5.0;
    let pt = poly.subs(u, v);
    let u0 = u + 0.2 * rand::random::<f64>() - 0.1;
    let v0 = v + 0.2 * rand::random::<f64>() - 0.1;
    match algo::surface::search_nearest_parameter(&poly, pt, (u0, v0), 100) {
        Some(res) => match poly.subs(res.0, res.1).near(&pt) {
            true => true,
            false => {
                eprintln!(
                    "wrong answer\npolynomial: {:?}\nanswer: {:?}\nhint: {:?}\nresult: {:?}",
                    poly,
                    (u, v),
                    (u0, v0),
                    res,
                );
                false
            }
        },
        None => {
            eprintln!(
                "not converge\npolynomial: {:?}\nanswer: {:?}\nhint: {:?}",
                poly,
                (u, v),
                (u0, v0),
            );
            false
        }
    }
}

#[test]
fn polysurface_snp_on_surface() {
    let flag = (0..10).any(|_| {
        let count = (0..100)
            .filter(|_| exec_polysurface_snp_on_surface())
            .count();
        if count <= 90 {
            eprintln!("wrong answer: {:?}", 100 - count);
        }
        count > 90
    });
    assert!(flag, "too many failure");
}

fn exec_polysurface_sp_on_surface() -> bool {
    let coef0 = vec![
        Vector3::new(0.0, 1.0, 3.0 * rand::random::<f64>() - 1.5),
        Vector3::new(1.0, 0.0, 3.0 * rand::random::<f64>() - 1.5),
        Vector3::new(0.0, 0.0, 3.0 * rand::random::<f64>() - 1.5),
        Vector3::new(0.0, 0.0, 3.0 * rand::random::<f64>() - 1.5),
    ];
    let coef1 = vec![
        Vector3::new(1.0, 0.0, 3.0 * rand::random::<f64>() - 1.5),
        Vector3::new(0.0, 1.0, 3.0 * rand::random::<f64>() - 1.5),
        Vector3::new(0.0, 0.0, 3.0 * rand::random::<f64>() - 1.5),
        Vector3::new(0.0, 0.0, 3.0 * rand::random::<f64>() - 1.5),
    ];
    let poly = PolySurface(PolyCurve(coef0), PolyCurve(coef1));
    let u = 10.0 * rand::random::<f64>() - 5.0;
    let v = 10.0 * rand::random::<f64>() - 5.0;
    let pt = poly.subs(u, v);
    let u0 = u + 2.0 * rand::random::<f64>() - 1.0;
    let v0 = v + 2.0 * rand::random::<f64>() - 1.0;
    match algo::surface::search_parameter(&poly, pt, (u0, v0), 100) {
        Some(res) => match poly.subs(res.0, res.1).near(&pt) {
            true => true,
            false => {
                eprintln!(
                    "wrong answer\npolynomial: {:?}\nanswer: {:?}\nhint: {:?}\nresult: {:?}",
                    poly,
                    (u, v),
                    (u0, v0),
                    res,
                );
                false
            }
        },
        None => {
            eprintln!(
                "not converge\npolynomial: {:?}\nanswer: {:?}\nhint: {:?}",
                poly,
                (u, v),
                (u0, v0),
            );
            false
        }
    }
}

#[test]
fn polysurface_sp_on_surface() {
    let flag = (0..10).any(|_| {
        let count = (0..100)
            .filter(|_| exec_polysurface_sp_on_surface())
            .count();
        if count <= 90 {
            eprintln!("wrong answer: {:?}", 100 - count);
        }
        count > 90
    });
    assert!(flag, "too many failure");
}

fn exec_polysurface_intersection_point() -> bool {
    let (a, b) = (rand::random::<f64>(), rand::random::<f64>());
    let coef0 = vec![
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
    ];
    let coef1 = vec![
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
    ];
    let surface = PolySurface(PolyCurve(coef0), PolyCurve(coef1));
    let coef = vec![Vector3::new(a, b, 0.0), Vector3::new(0.0, 0.0, 1.0)];
    let curve = PolyCurve::<Point3>(coef);

    match surface::search_intersection_parameter(&surface, (0.5, 0.5), &curve, 0.0, 100) {
        Some(((x, y), z)) => {
            let (p, q) = (surface.subs(x, y), curve.subs(z));
            p.near(&q) && x.near(&a) && y.near(&b) && p.z.near(&z)
        }
        None => false,
    }
}

#[test]
fn polysurface_intersection_point() {
    let count = (0..10)
        .filter(|_| exec_polysurface_intersection_point())
        .count();
    assert!(count > 7, "wrong answer: {:?}", 10 - count);
}

fn exec_polysurface_division() -> bool {
    let coef0 = vec![
        Vector3::new(0.0, 1.0, 10.0 * rand::random::<f64>() - 5.0),
        Vector3::new(1.0, 0.0, 10.0 * rand::random::<f64>() - 5.0),
        Vector3::new(0.0, 0.0, 10.0 * rand::random::<f64>() - 5.0),
    ];
    let coef1 = vec![
        Vector3::new(1.0, 0.0, 10.0 * rand::random::<f64>() - 5.0),
        Vector3::new(0.0, 1.0, 10.0 * rand::random::<f64>() - 5.0),
        Vector3::new(0.0, 0.0, 10.0 * rand::random::<f64>() - 5.0),
    ];
    let poly = PolySurface(PolyCurve(coef0), PolyCurve(coef1));
    let (udiv, vdiv) = algo::surface::parameter_division(&poly, ((-1.0, 1.0), (-1.0, 1.0)), 0.1);
    for (i, u) in udiv
        .windows(2)
        .flat_map(move |u| (1..3).map(move |i| (i, u)))
    {
        for (j, v) in vdiv
            .windows(2)
            .flat_map(move |v| (1..3).map(move |j| (j, v)))
        {
            let p = i as f64 / 3.0;
            let q = j as f64 / 3.0;
            let pt0 = Point3::from_vec(
                poly.subs(u[0], v[0]).to_vec() * (1.0 - p) * (1.0 - q)
                    + poly.subs(u[1], v[0]).to_vec() * p * (1.0 - q)
                    + poly.subs(u[0], v[1]).to_vec() * (1.0 - p) * q
                    + poly.subs(u[1], v[1]).to_vec() * p * q,
            );
            let u = u[0] * (1.0 - p) + u[1] * p;
            let v = v[0] * (1.0 - q) + v[1] * q;
            let pt1 = poly.subs(u, v);
            if pt0.distance(pt1) > 0.2 {
                return false;
            }
        }
    }
    true
}

#[test]
fn polysurface_division() {
    let count = (0..10).filter(|_| exec_polysurface_division()).count();
    assert!(count > 8, "wrong answer: {:?}", 10 - count);
}
