use std::f64::consts::PI;

use truck_geometry::prelude::{rbf_surface::RadiusFunction, *};

#[test]
fn contact_circle_as_curve() {
    let line = Line(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let (plane0, plane1) = (Plane::xy(), Plane::zx());
    let (r, t) = (0.5, 0.75);
    let rfs = RbfSurface::new(line, plane0, plane1, r);
    let cc = rfs.contact_circle(t).unwrap();

    assert_near!(cc.subs(0.0), cc.contact_point0().point);
    assert_near!(cc.subs(1.0), cc.contact_point1().point);

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
    struct Radius;
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

    let fillet = RbfSurface::new(edge_circle, sphere0, sphere1, Radius);
    let cp_curve0 = fillet.contact_curve0();
    let cp_curve1 = fillet.contact_curve1();

    let uc = UnitCircle::<Point3>::new();
    const N: usize = 20;
    for i in 0..=N {
        let t = 2.0 * PI * i as f64 / N as f64;
        let cc = fillet.contact_circle(t).unwrap();

        let r = Radius.subs(t);
        let center_radius = ((r + 2.0).powi(2) - 1.0).sqrt();
        assert_near!(cc.center(), center_radius * uc.subs(t));

        let contact_radius = 2.0 / (2.0 + r) * center_radius;
        let contact_z = r / (2.0 + r);
        let contact_point0 = contact_radius * uc.subs(t) + contact_z * Vector3::unit_z();
        assert_near!(cp_curve0.subs(t), contact_point0);
        let contact_point1 = contact_radius * uc.subs(t) - contact_z * Vector3::unit_z();
        assert_near!(cp_curve1.subs(t), contact_point1);

        let eps = 1.0e-4;
        let cc_plus = fillet.contact_circle(t + eps).unwrap();
        let cc_minus = fillet.contact_circle(t - eps).unwrap();

        let center_der_approx = (cc_plus.center() - cc_minus.center()) / (2.0 * eps);
        assert!((fillet.center_der(cc) - center_der_approx).magnitude() < eps);

        let cp0_der_approx =
            (cc_plus.contact_point0().point - cc_minus.contact_point0().point) / (2.0 * eps);
        assert!((cp_curve0.der(t) - cp0_der_approx).magnitude() < eps);

        let cp1_der_approx =
            (cc_plus.contact_point1().point - cc_minus.contact_point1().point) / (2.0 * eps);
        assert!((cp_curve1.der(t) - cp1_der_approx).magnitude() < eps);

        let axis_der_approx = (cc_plus.axis() - cc_minus.axis()) / (2.0 * eps);
        assert!((fillet.axis_der(cc) - axis_der_approx).magnitude() < eps);

        let angle_der_approx = (cc_plus.angle() - cc_minus.angle()).0 / (2.0 * eps);
        assert!((fillet.angle_der(cc) - angle_der_approx).abs() < eps);

        let t0 = cp_curve0
            .search_parameter(cc.contact_point0().point, None, 10)
            .unwrap();
        assert_near!(cp_curve0.subs(t0), cc.contact_point0().point);
        let t1 = cp_curve1
            .search_parameter(cc.contact_point1().point, None, 10)
            .unwrap();
        assert_near!(cp_curve1.subs(t1), cc.contact_point1().point);

        for j in 0..=N {
            let (u, v) = (j as f64 / N as f64, t);
            let vder_approx = (cc_plus.subs(u) - cc_minus.subs(u)) / (2.0 * eps);
            assert!((fillet.vder(u, v) - vder_approx).magnitude() < eps);

            let p = cc.subs(u);
            let (u0, v0) = fillet
                .search_parameter(p, None, 10)
                .unwrap_or_else(|| panic!("{:?}", (u, v)));
            assert_near!(fillet.subs(u0, v0), p);
        }
    }
}
