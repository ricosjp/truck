use std::f64::consts::PI;
use truck_geometry::prelude::*;

#[test]
fn contact_circle_as_curve() {
    let line = Line(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let (plane0, plane1) = (Plane::xy(), Plane::zx());
    let (r, t) = (0.5, 0.75);
    let rfs = RbfSurface::new(line, plane0, plane1, r);
    let cc = rfs.contact_circle(t).unwrap();

    assert_near!(cc.subs(0.0), cc.contact_point0().point);
    assert_near!(cc.subs(1.0), cc.contact_point1().point);

    let nurbs: NurbsCurve<Vector4> = cc.to_same_geometry();
    assert_near!(cc.subs(0.0), nurbs.subs(0.0));
    assert_near!(cc.subs(0.5), nurbs.subs(0.5));
    assert_near!(cc.subs(1.0), nurbs.subs(1.0));
    assert_near!((nurbs.subs(0.2) - cc.center()).magnitude(), r);

    const EPS: f64 = 1.0e-4;
    for i in 1..=9 {
        let s = i as f64 / 10.0;
        assert_near!(cc.subs(s).distance2(cc.center()), r * r);
        let cal_der = (cc.subs(s + EPS) - cc.subs(s - EPS)) / (2.0 * EPS);
        assert!((cal_der - cc.der(s)).magnitude() < EPS);
        let cal_der2 = (cc.der(s + EPS) - cc.der(s - EPS)) / (2.0 * EPS);
        assert!((cal_der2 - cc.der2(s)).magnitude() < EPS);
    }
}

#[test]
fn fillet_between_two_spheres() {
    let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 1.0), 2.0);
    let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -1.0), 2.0);
    let edge_circle = Processor::with_transform(
        UnitCircle::<Point3>::new(),
        Matrix4::from_scale(f64::sqrt(3.0)),
    );

    #[derive(Clone, Copy, Debug)]
    struct Radius(f64);
    impl RadiusFunction for Radius {
        fn der_n(&self, n: usize, t: f64) -> f64 {
            let o = if n == 0 { 1.0 } else { 0.0 };
            let x = match n % 4 {
                0 => f64::cos(t),
                1 => -f64::sin(t),
                2 => -f64::cos(t),
                _ => f64::sin(t),
            };
            o + 0.2 * x
        }
    }
    impl InvertibleRadiusFunction for Radius {
        fn inverse(&self) -> Self { Self(-self.0) }
        fn invert(&mut self) { self.0 *= -1.0; }
    }

    let fillet = RbfSurface::new(edge_circle, sphere0, sphere1, Radius(1.0));
    let cp_curve0 = fillet.contact_curve0();
    let cp_curve0_inv = cp_curve0.inverse();
    let cp_curve1 = fillet.contact_curve1();
    let cp_curve1_inv = cp_curve1.inverse();

    let uc = UnitCircle::<Point3>::new();
    const N: usize = 20;
    for i in 0..=N {
        let t = 2.0 * PI * i as f64 / N as f64;
        let cc = fillet.contact_circle(t).unwrap();

        let r = Radius(1.0).subs(t);
        let center_radius = ((r + 2.0).powi(2) - 1.0).sqrt();
        assert_near!(cc.center(), center_radius * uc.subs(t));

        let contact_radius = 2.0 / (2.0 + r) * center_radius;
        let contact_z = r / (2.0 + r);
        let contact_point0 = contact_radius * uc.subs(t) + contact_z * Vector3::unit_z();
        assert_near!(cp_curve0.subs(t), contact_point0);
        let contact_point1 = contact_radius * uc.subs(t) - contact_z * Vector3::unit_z();
        assert_near!(cp_curve1.subs(t), contact_point1);

        let t0 = cp_curve0
            .search_parameter(cc.contact_point0().point, None, 100)
            .unwrap();
        assert_near!(cp_curve0.subs(t0), cc.contact_point0().point);
        assert_near!(cp_curve0_inv.subs(2.0 * PI - t0), cc.contact_point0().point);
        let t1 = cp_curve1
            .search_parameter(cc.contact_point1().point, None, 100)
            .unwrap();
        assert_near!(cp_curve1.subs(t1), cc.contact_point1().point);
        assert_near!(cp_curve1_inv.subs(2.0 * PI - t1), cc.contact_point1().point);

        for j in 0..=N {
            let (u, v) = (j as f64 / N as f64, t);

            let eps = 1.0e-4;

            let ders = fillet.ders(2, u, v);

            let ders_plus = fillet.ders(1, u + eps, v);
            let ders_minus = fillet.ders(1, u - eps, v);
            let uder_approx = (ders_plus[0][0] - ders_minus[0][0]) / (2.0 * eps);
            assert!((ders[1][0] - uder_approx).magnitude() < eps);
            let uvder_approx = (ders_plus[0][1] - ders_minus[0][1]) / (2.0 * eps);
            assert!((ders[1][1] - uvder_approx).magnitude() < eps);
            let uuder_approx = (ders_plus[1][0] - ders_minus[1][0]) / (2.0 * eps);
            assert!((ders[2][0] - uuder_approx).magnitude() < eps);

            let ders_plus = fillet.ders(1, u, v + eps);
            let ders_minus = fillet.ders(1, u, v - eps);
            let vder_approx = (ders_plus[0][0] - ders_minus[0][0]) / (2.0 * eps);
            assert!((ders[0][1] - vder_approx).magnitude() < eps);
            let uvder_approx = (ders_plus[1][0] - ders_minus[1][0]) / (2.0 * eps);
            assert!((ders[1][1] - uvder_approx).magnitude() < eps);
            let vvder_approx = (ders_plus[0][1] - ders_minus[0][1]) / (2.0 * eps);
            assert!((ders[0][2] - vvder_approx).magnitude() < eps);

            let p = cc.subs(u);
            let (u0, v0) = fillet
                .search_parameter(p, None, 10)
                .unwrap_or_else(|| panic!("{:?}", (u, v)));
            assert_near!(fillet.subs(u0, v0), p);
        }
    }
}

// cross point with adjacent edge
#[test]
fn test_cpwae_plane() {
    let surface0 = Plane::xy();
    let surface1 = Plane::yz();
    let edge_curve = Line(Point3::origin(), Point3::new(0.0, 1.0, 0.0));
    let rbf_surface = RbfSurface::new(edge_curve, surface0, surface1, 0.5);

    let adjacent_curve = Line(Point3::new(0.0, 1.0, 0.0), Point3::new(-1.0, 2.0, 0.0));
    let (cp0, cp1, t, s) = rbf_surface
        .search_contact_curve0_cross_point_with_adjacent_edge(1.0, adjacent_curve, 0.0, 10)
        .unwrap();
    assert_near!(cp0.point, Point3::new(-0.5, 1.5, 0.0));
    assert_near!(cp1.point, Point3::new(0.0, 1.5, -0.5));
    assert_near!(t, 1.5);
    assert_near!(s, 0.5);

    let adjacent_curve = Line(Point3::new(0.0, 1.0, 0.0), Point3::new(0.0, 2.0, -1.0));
    let (cp0, cp1, t, s) = rbf_surface
        .search_contact_curve1_cross_point_with_adjacent_edge(1.0, adjacent_curve, 0.0, 10)
        .unwrap();
    assert_near!(cp0.point, Point3::new(-0.5, 1.5, 0.0));
    assert_near!(cp1.point, Point3::new(0.0, 1.5, -0.5));
    assert_near!(t, 1.5);
    assert_near!(s, 0.5);
}

#[test]
fn test_cpwae() {
    #[rustfmt::skip]
    let surface0: NurbsSurface<Vector4> = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(-1.0, 0.0, 0.0), Point3::new(-1.0, 0.5, 0.0), Point3::new(-1.0, 1.0, 1.0)],
            vec![Point3::new(0.0, 0.0, 0.0),  Point3::new(0.0, 0.5, 0.0),  Point3::new(0.0, 1.0, 1.0)],
            vec![Point3::new(1.0, 0.0, 0.0),  Point3::new(1.0, 0.5, 0.0),  Point3::new(1.0, 1.0, 1.0)],
        ],
    )
    .into();
    #[rustfmt::skip]
    let surface1: NurbsSurface<Vector4> = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(1.0, 0.0, 0.0),  Point3::new(1.0, 0.0, -0.5),  Point3::new(1.0, 1.0, -1.0)],
            vec![Point3::new(0.0, 0.0, 0.0),  Point3::new(0.0, 0.5, -0.5),  Point3::new(0.0, 1.0, -1.0)],
            vec![Point3::new(-1.0, 0.0, 0.0), Point3::new(-1.0, 0.0, -0.5), Point3::new(-1.0, 1.0, -1.0)],
        ],
    )
    .into();

    let line = Line(Point3::new(-1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));

    #[derive(Clone, Copy, Debug)]
    struct Radius;
    impl RadiusFunction for Radius {
        fn der_n(&self, n: usize, t: f64) -> f64 {
            match n {
                0 => 0.3 + 0.3 * t,
                1 => 0.3,
                _ => 0.0,
            }
        }
    }

    let boundary0 = surface0.splitted_boundary();
    let adjacent_curve = boundary0[1].clone();

    let rbf_surface = RbfSurface::new(line, &surface0, &surface1, Radius);

    let (contact0, _, t, s) = rbf_surface
        .search_contact_curve0_cross_point_with_adjacent_edge(1.0, &adjacent_curve, 0.0, 100)
        .unwrap();
    println!("{contact0:?}, {t}, {s}");

    let (u0, v0) = contact0.uv.into();
    assert_near!(contact0.point, surface0.subs(u0, v0));
    assert_near!(contact0.point, rbf_surface.subs(0.0, t));
    assert_near!(contact0.point, adjacent_curve.subs(s));
}
