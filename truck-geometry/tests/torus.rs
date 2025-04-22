use proptest::prelude::*;
use std::f64::consts::PI;
use std::ops::RangeBounds;
use truck_geometry::prelude::*;

proptest! {
    #[test]
    fn surface(
        small_radius in 1f64..=5f64,
        radius_diff in 0.1f64..=5f64,
        (u, v) in (0f64..=2.0 * PI, 0f64..=2.0 * PI),
        deform in -0.4f64..=0.4
    ) {
        const EPS: f64 = 1.0e-2;
        let large_radius = 2.0 * small_radius + radius_diff;
        let torus = Torus::new(Point3::origin(), large_radius, small_radius);

        let p = torus.subs(u, v);
        let r = large_radius * Point3::new(f64::cos(u), f64::sin(u), 0.0);
        prop_assert_near!(p.distance(r), small_radius);
        prop_assert_near!(p.z, small_radius * f64::sin(v));

        let uder0 = torus.uder(u, v);
        let uder1 = (torus.subs(u + EPS, v) - torus.subs(u - EPS, v)) / (2.0 * EPS);
        prop_assert!((uder0 - uder1).magnitude() < EPS, "{:?} {:?}", uder0, uder1);

        let vder0 = torus.vder(u, v);
        let vder1 = (torus.subs(u, v + EPS) - torus.subs(u, v - EPS)) / (2.0 * EPS);
        prop_assert!((vder0 - vder1).magnitude() < EPS, "{:?} {:?}", vder0, vder1);

        let uuder0 = torus.uuder(u, v);
        let uuder1 = (torus.uder(u + EPS, v) - torus.uder(u - EPS, v)) / (2.0 * EPS);
        prop_assert!(
            (uuder0 - uuder1).magnitude() < EPS,
            "{:?} {:?}",
            uuder0,
            uuder1
        );

        let uvder0 = torus.uvder(u, v);
        let uvder1 = (torus.vder(u + EPS, v) - torus.vder(u - EPS, v)) / (2.0 * EPS);
        prop_assert!(
            (uvder0 - uvder1).magnitude() < EPS,
            "{:?} {:?}",
            uvder0,
            uvder1
        );

        let vvder0 = torus.vvder(u, v);
        let vvder1 = (torus.vder(u, v + EPS) - torus.vder(u, v - EPS)) / (2.0 * EPS);
        prop_assert!(
            (vvder0 - vvder1).magnitude() < EPS,
            "{:?} {:?}",
            vvder0,
            vvder1
        );

        let n0 = torus.normal(u, v);
        let n1 = torus.uder(u, v).cross(torus.vder(u, v)).normalize();
        prop_assert_near!(n0, n1);

        let (u0, v0) = torus.search_parameter(p, None, 1).unwrap();
        let (urange, vrange) = torus.parameter_range();
        prop_assert!(urange.contains(&u0) && vrange.contains(&v0), "{u0}, {v0}");
        prop_assert_near!(torus.subs(u0, v0), p);

        let deform = deform * small_radius;
        let q = p + deform * n0;
        let (u0, v0) = torus.search_nearest_parameter(q, None, 1).unwrap();
        let (urange, vrange) = torus.parameter_range();
        prop_assert!(urange.contains(&u0) && vrange.contains(&v0), "{u0}, {v0}");
        prop_assert_near!(torus.subs(u0, v0), p);
    }

    #[test]
    fn test_der_mn(
        (u, v) in (0f64..=PI, 0f64..=2.0 * PI),
        (m, n) in (0usize..=4, 0usize..=4),
        center in prop::array::uniform3(-100f64..=100.0),
        large_radius in 6.0f64..=10.0,
        small_radius in 0.1f64..=5.0,
        u_derivate in prop::bool::ANY,
    ) {
        let torus = Torus::new(Point3::from(center), large_radius, small_radius);

        const EPS: f64 = 1.0e-4;
        let (der0, der1) = if u_derivate {
            let der0 = torus.der_mn(u, v, m + 1, n);
            let der1 = (torus.der_mn(u + EPS, v, m, n) - torus.der_mn(u - EPS, v, m, n)) / (2.0 * EPS);
            (der0, der1)
        } else {
            let der0 = torus.der_mn(u, v, m, n + 1);
            let der1 = (torus.der_mn(u, v + EPS, m, n) - torus.der_mn(u, v - EPS, m, n)) / (2.0 * EPS);
            (der0, der1)
        };
        prop_assert!((der0 - der1).magnitude() < 0.01 * der0.magnitude());
    }
}
