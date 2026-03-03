use proptest::prelude::*;
use monstertruck_geometry::prelude::*;

#[test]
fn nurbs_sphere() {
    let knot_vec = KnotVector::bezier_knot(2);
    let control_points = vec![
        vec![
            Vector4::new(1.0, 0.0, 0.0, 1.0),
            Vector4::new(1.0, 1.0, 0.0, 1.0),
            Vector4::new(0.0, 2.0, 0.0, 2.0),
        ],
        vec![
            Vector4::new(1.0, 0.0, 1.0, 1.0),
            Vector4::new(1.0, 1.0, 1.0, 1.0),
            Vector4::new(0.0, 2.0, 0.0, 2.0),
        ],
        vec![
            Vector4::new(0.0, 0.0, 2.0, 2.0),
            Vector4::new(0.0, 2.0, 2.0, 2.0),
            Vector4::new(0.0, 4.0, 0.0, 4.0),
        ],
    ];
    let surface = NurbsSurface::new(BsplineSurface::new(
        (knot_vec.clone(), knot_vec),
        control_points,
    ));

    const N: usize = 10;
    for i in 0..=10 {
        for j in 0..=10 {
            let u = i as f64 / N as f64;
            let v = j as f64 / N as f64;
            let p = surface.subs(u, v).to_vec();
            assert_near!(p.magnitude(), 1.0);
            let uder = surface.uder(u, v);
            assert!(p.dot(uder).so_small());
            let vder = surface.vder(u, v);
            assert!(p.dot(vder).so_small());
        }
    }
}

proptest! {
    #[test]
    fn test_der_mn(
        (u, v) in (0f64..=1.0, 0f64..=1.0),
        (m, n) in (0usize..=4, 0usize..=4),
        (udegree, vdegree) in (2usize..=6, 2usize..=6),
        (udiv, vdiv) in (1usize..=10, 1usize..=10),
        pts in prop::array::uniform16(prop::array::uniform16(prop::array::uniform3(-10f64..=10.0))),
        weights in prop::array::uniform16(prop::array::uniform16(0.5f64..=10.0)),
        u_derivate in prop::bool::ANY,
    ) {
        prop_assume!(udegree > m + 1);
        prop_assume!(vdegree > n + 1);
        let knot_vector_u = KnotVector::uniform_knot(udegree, udiv);
        let knot_vector_v = KnotVector::uniform_knot(vdegree, vdiv);
        let control_points = pts[..udegree + udiv]
            .iter()
            .zip(weights)
            .map(|(vec, weights)| {
                vec[..vdegree + vdiv]
                    .iter()
                    .zip(weights)
                    .map(|(&p, w)| Vector4::new(p[0], p[1], p[2], w))
                    .collect()
            })
            .collect::<Vec<Vec<_>>>();
        let bsp = NurbsSurface::new(BsplineSurface::new((knot_vector_u, knot_vector_v), control_points));

        const EPS: f64 = 1.0e-4;
        let (der0, der1) = if u_derivate {
            let der0 = bsp.der_mn(m + 1, n, u, v);
            let der1 = (bsp.der_mn(m, n, u + EPS, v) - bsp.der_mn(m, n, u - EPS, v)) / (2.0 * EPS);
            (der0, der1)
        } else {
            let der0 = bsp.der_mn(m, n + 1, u, v);
            let der1 = (bsp.der_mn(m, n, u, v + EPS) - bsp.der_mn(m, n, u, v - EPS)) / (2.0 * EPS);
            (der0, der1)
        };
        prop_assert!((der0 - der1).magnitude() < 0.01 * der0.magnitude());
    }
}

fn endpoint_test_surface() -> NurbsSurface<Vector4> {
    let knot_vector_u = KnotVector::uniform_knot(2, 2);
    let knot_vector_v = KnotVector::uniform_knot(2, 2);
    let control_points = (0..4)
        .map(|i| {
            (0..4)
                .map(|j| {
                    let i = i as f64;
                    let j = j as f64;
                    Vector4::new(i, j, 0.1 * (i * i + j * j) + 0.05 * i * j, 1.0)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    NurbsSurface::new(BsplineSurface::new((knot_vector_u, knot_vector_v), control_points))
}

fn seam_samples() -> impl Iterator<Item = f64> { (0..=20).map(|i| i as f64 / 20.0) }

fn assert_u_seam_matches(surface: &NurbsSurface<Vector4>, cut: f64, check_higher_order: bool) {
    let mut left = surface.clone();
    let right = left.cut_u(cut);
    let can_compare_derivatives =
        left.control_points().len() > 1 && right.control_points().len() > 1;
    let higher_order_tolerance = 1.0e-2;

    seam_samples().for_each(|v| {
        assert_near!(left.subs(cut, v), right.subs(cut, v));
        assert_near!(left.subs(cut, v), surface.subs(cut, v));
        if can_compare_derivatives {
            assert_near!(left.uder(cut, v), right.uder(cut, v));
            assert_near!(left.uder(cut, v), surface.uder(cut, v));
            assert_near!(left.vder(cut, v), right.vder(cut, v));
            assert_near!(left.vder(cut, v), surface.vder(cut, v));
            if check_higher_order {
                assert!(
                    (left.uuder(cut, v) - right.uuder(cut, v)).magnitude()
                        <= higher_order_tolerance
                );
                assert!(
                    (left.uuder(cut, v) - surface.uuder(cut, v)).magnitude()
                        <= higher_order_tolerance
                );
                assert!(
                    (left.uvder(cut, v) - right.uvder(cut, v)).magnitude()
                        <= higher_order_tolerance
                );
                assert!(
                    (left.uvder(cut, v) - surface.uvder(cut, v)).magnitude()
                        <= higher_order_tolerance
                );
                assert!(
                    (left.vvder(cut, v) - right.vvder(cut, v)).magnitude()
                        <= higher_order_tolerance
                );
                assert!(
                    (left.vvder(cut, v) - surface.vvder(cut, v)).magnitude()
                        <= higher_order_tolerance
                );
            }
        }
    });
}

fn assert_v_seam_matches(surface: &NurbsSurface<Vector4>, cut: f64, check_higher_order: bool) {
    let mut lower = surface.clone();
    let upper = lower.cut_v(cut);
    let can_compare_derivatives =
        lower.control_points()[0].len() > 1 && upper.control_points()[0].len() > 1;
    let higher_order_tolerance = 1.0e-2;

    seam_samples().for_each(|u| {
        assert_near!(lower.subs(u, cut), upper.subs(u, cut));
        assert_near!(lower.subs(u, cut), surface.subs(u, cut));
        if can_compare_derivatives {
            assert_near!(lower.uder(u, cut), upper.uder(u, cut));
            assert_near!(lower.uder(u, cut), surface.uder(u, cut));
            assert_near!(lower.vder(u, cut), upper.vder(u, cut));
            assert_near!(lower.vder(u, cut), surface.vder(u, cut));
            if check_higher_order {
                assert!(
                    (lower.uuder(u, cut) - upper.uuder(u, cut)).magnitude()
                        <= higher_order_tolerance
                );
                assert!(
                    (lower.uuder(u, cut) - surface.uuder(u, cut)).magnitude()
                        <= higher_order_tolerance
                );
                assert!(
                    (lower.uvder(u, cut) - upper.uvder(u, cut)).magnitude()
                        <= higher_order_tolerance
                );
                assert!(
                    (lower.uvder(u, cut) - surface.uvder(u, cut)).magnitude()
                        <= higher_order_tolerance
                );
                assert!(
                    (lower.vvder(u, cut) - upper.vvder(u, cut)).magnitude()
                        <= higher_order_tolerance
                );
                assert!(
                    (lower.vvder(u, cut) - surface.vvder(u, cut)).magnitude()
                        <= higher_order_tolerance
                );
            }
        }
    });
}

#[test]
fn ucut_at_domain_start_regression() {
    let surface = endpoint_test_surface();
    let mut left = surface.clone();
    let right = left.cut_u(0.0);

    assert_eq!(left.control_points().len(), 1);
    for i in 0..=20 {
        let v = i as f64 / 20.0;
        assert_near!(left.subs(0.0, v), surface.subs(0.0, v));
    }
    for i in 0..=20 {
        let u = i as f64 / 20.0;
        for j in 0..=20 {
            let v = j as f64 / 20.0;
            assert_near!(right.subs(u, v), surface.subs(u, v));
        }
    }
}

#[test]
fn vcut_at_domain_start_regression() {
    let surface = endpoint_test_surface();
    let mut lower = surface.clone();
    let upper = lower.cut_v(0.0);

    assert_eq!(lower.control_points()[0].len(), 1);
    for i in 0..=20 {
        let u = i as f64 / 20.0;
        assert_near!(lower.subs(u, 0.0), surface.subs(u, 0.0));
    }
    for i in 0..=20 {
        let u = i as f64 / 20.0;
        for j in 0..=20 {
            let v = j as f64 / 20.0;
            assert_near!(upper.subs(u, v), surface.subs(u, v));
        }
    }
}

#[test]
fn ucut_near_domain_end_continuity_regression() {
    let surface = endpoint_test_surface();
    let cut = 0.999_999_385_948_107_8;
    assert_u_seam_matches(&surface, cut, true);
}

#[test]
fn vcut_near_domain_end_continuity_regression() {
    let surface = endpoint_test_surface();
    let cut = 0.999_999_385_948_107_8;
    assert_v_seam_matches(&surface, cut, true);
}

#[test]
fn ucut_at_domain_end_regression() {
    let surface = endpoint_test_surface();
    let mut left = surface.clone();
    let right = left.cut_u(1.0);

    assert_eq!(right.control_points().len(), 1);
    seam_samples().for_each(|v| assert_near!(right.subs(1.0, v), surface.subs(1.0, v)));
    seam_samples().for_each(|u| {
        seam_samples().for_each(|v| assert_near!(left.subs(u, v), surface.subs(u, v)));
    });
}

#[test]
fn vcut_at_domain_end_regression() {
    let surface = endpoint_test_surface();
    let mut lower = surface.clone();
    let upper = lower.cut_v(1.0);

    assert_eq!(upper.control_points()[0].len(), 1);
    seam_samples().for_each(|u| assert_near!(upper.subs(u, 1.0), surface.subs(u, 1.0)));
    seam_samples().for_each(|u| {
        seam_samples().for_each(|v| assert_near!(lower.subs(u, v), surface.subs(u, v)));
    });
}

#[test]
fn ucut_knot_boundary_sweep_regression() {
    let surface = endpoint_test_surface();
    let (knots, _) = surface.knot_vector_u().to_single_multi();
    knots
        .into_iter()
        .filter(|knot| *knot > 0.0 && *knot < 1.0)
        .for_each(|knot| {
            [-1.0e-9, 0.0, 1.0e-9].into_iter().for_each(|delta| {
                let cut = knot + delta;
                if (0.0..=1.0).contains(&cut) {
                    assert_u_seam_matches(&surface, cut, false);
                }
            });
        });
}

#[test]
fn vcut_knot_boundary_sweep_regression() {
    let surface = endpoint_test_surface();
    let (knots, _) = surface.knot_vector_v().to_single_multi();
    knots
        .into_iter()
        .filter(|knot| *knot > 0.0 && *knot < 1.0)
        .for_each(|knot| {
            [-1.0e-9, 0.0, 1.0e-9].into_iter().for_each(|delta| {
                let cut = knot + delta;
                if (0.0..=1.0).contains(&cut) {
                    assert_v_seam_matches(&surface, cut, false);
                }
            });
        });
}
