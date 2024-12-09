use std::f64::consts::PI;
use truck_geometry::prelude::*;

#[test]
fn revolve_test() {
    let pt0 = Point3::new(0.0, 2.0, 1.0);
    let pt1 = Point3::new(1.0, 0.0, 0.0);
    let curve = BSplineCurve::new(KnotVec::bezier_knot(1), vec![pt0, pt1]);
    let surface = RevolutedCurve::by_revolution(curve, Point3::origin(), Vector3::unit_y());
    const N: usize = 100;
    for i in 0..=N {
        for j in 0..=N {
            let u = i as f64 / N as f64;
            let v = 2.0 * PI * j as f64 / N as f64;
            let res = surface.subs(u, v);
            let ans = Point3::new(
                u * f64::cos(v) + (1.0 - u) * f64::sin(v),
                2.0 * (1.0 - u),
                -u * f64::sin(v) + (1.0 - u) * f64::cos(v),
            );
            assert_near!(res, ans);
            let res_uder = surface.uder(u, v);
            let ans_uder =
                Vector3::new(f64::cos(v) - f64::sin(v), -2.0, -f64::sin(v) - f64::cos(v));
            assert_near!(res_uder, ans_uder);
            let res_vder = surface.vder(u, v);
            let ans_vder = Vector3::new(
                -u * f64::sin(v) + (1.0 - u) * f64::cos(v),
                0.0,
                -u * f64::cos(v) - (1.0 - u) * f64::sin(v),
            );
            assert_near!(res_vder, ans_vder);
            let res_uuder = surface.uuder(u, v);
            let ans_uuder = Vector3::zero();
            assert_near!(res_uuder, ans_uuder);
            let res_uvder = surface.uvder(u, v);
            let ans_uvder =
                Vector3::new(-f64::sin(v) - f64::cos(v), 0.0, -f64::cos(v) + f64::sin(v));
            assert_near!(res_uvder, ans_uvder);
            let res_vvder = surface.vvder(u, v);
            let ans_vvder = Vector3::new(
                -u * f64::cos(v) - (1.0 - u) * f64::sin(v),
                0.0,
                u * f64::sin(v) - (1.0 - u) * f64::cos(v),
            );
            assert_near!(res_vvder, ans_vvder);
            let normal = surface.normal(u, v);
            assert!(normal.dot(res_uder).so_small());
            assert!(normal.dot(res_vder).so_small());
        }
    }
}

#[test]
fn search_parameter() {
    let line = BSplineCurve::new(
        KnotVec::bezier_knot(1),
        vec![Point3::new(0.0, 2.0, 1.0), Point3::new(1.0, 0.0, 0.0)],
    );
    let surface = RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_y());
    let pt = Point3::new(-0.5, 1.0, 0.5);
    let (u, v) = surface.search_parameter(pt, Some((0.4, 1.2)), 100).unwrap();
    assert_near!(surface.subs(u, v), pt);
}

#[test]
fn search_parameter_with_fixed_points() {
    let line = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, -1.0, 0.0),
        ],
    );
    let surface = RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_y());

    let (u, v) = surface
        .search_parameter(Point3::new(0.0, 1.0, 0.0), Some((0.5, 0.3)), 10)
        .unwrap();
    assert_near!(Vector2::new(u, v), Vector2::new(0.0, 0.3));

    let (u, v) = surface
        .search_parameter(Point3::new(0.0, -1.0, 0.0), Some((0.5, 0.3)), 10)
        .unwrap();
    assert_near!(Vector2::new(u, v), Vector2::new(1.0, 0.3));
}

#[test]
fn search_nearest_parameter() {
    let line = BSplineCurve::new(
        KnotVec::bezier_knot(1),
        vec![Point3::new(0.0, 2.0, 1.0), Point3::new(1.0, 0.0, 0.0)],
    );
    let surface = RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_y());
    let pt = surface.subs(0.4, 1.2) + 0.1 * surface.normal(0.4, 1.2);
    let (u, v) = surface
        .search_nearest_parameter(pt, Some((0.4, 1.2)), 100)
        .unwrap();
    assert_near!(Vector2::new(u, v), Vector2::new(0.4, 1.2));
}

#[test]
fn search_nearest_parameter_with_fixed_points() {
    let line = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, -1.0, 0.0),
        ],
    );
    let surface = RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_y());

    let (u, v) = surface
        .search_nearest_parameter(Point3::new(0.0, 2.0, 0.0), Some((0.5, 0.3)), 10)
        .unwrap();
    assert_near!(Vector2::new(u, v), Vector2::new(0.0, 0.3));

    let (u, v) = surface
        .search_nearest_parameter(Point3::new(0.0, -2.0, 0.0), Some((0.5, 0.3)), 10)
        .unwrap();
    assert_near!(Vector2::new(u, v), Vector2::new(1.0, 0.3));
}

#[test]
fn include_curve_normal() {
    let line = BSplineCurve::new(
        KnotVec::bezier_knot(1),
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 2.0, 2.0)],
    );
    let surface = RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_y());
    let parabola = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(-1.0, 1.0, 0.0),
        ],
    );
    assert!(surface.include(&parabola));
}

#[test]
fn include_curve_abnormal0() {
    let line = BSplineCurve::new(
        KnotVec::bezier_knot(1),
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 2.0, 2.0)],
    );
    let surface = RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_y());
    let parabola = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 2.0),
            Point3::new(-1.0, 1.0, 0.0),
        ],
    );
    assert!(!surface.include(&parabola));
}

#[test]
fn include_curve_abnormal1() {
    let curve = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(3),
        vec![
            Vector4::new(0.0, 3.0, 0.0, 1.0),
            Vector4::new(0.0, 3.0, 3.0, 0.5),
            Vector4::new(0.0, 0.0, 0.0, 1.0),
            Vector4::new(0.0, 0.0, 3.0, 1.0),
        ],
    ));
    let pt0 = curve.subs(0.2);
    let pt1 = curve.subs(0.6);
    let surface = RevolutedCurve::by_revolution(curve, Point3::origin(), Vector3::unit_y());
    let line = BSplineCurve::new(KnotVec::bezier_knot(1), vec![pt0, pt1]);
    assert!(!surface.include(&line));
}
