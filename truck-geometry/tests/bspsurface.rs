use truck_geometry::prelude::*;
use proptest::prelude::*;

#[test]
fn test_substitution() {
    let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    let ctrl_pts = vec![
        vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(0.5, -1.0),
            Vector2::new(1.0, 0.0),
        ],
        vec![
            Vector2::new(0.0, 1.0),
            Vector2::new(0.5, 2.0),
            Vector2::new(1.0, 1.0),
        ],
    ];
    let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);

    // bspsurface: (v, 2v(1 - v)(2u - 1) + u)
    const N: usize = 100; // sample size
    for i in 0..=N {
        let u = (i as f64) / (N as f64);
        for j in 0..=N {
            let v = (j as f64) / (N as f64);
            assert_near2!(
                bspsurface.subs(u, v),
                Vector2::new(v, 2.0 * v * (1.0 - v) * (2.0 * u - 1.0) + u),
            );
        }
    }
}

#[test]
fn test_uderivation() {
    let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    let ctrl_pts = vec![
        vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(0.5, -1.0),
            Vector2::new(1.0, 0.0),
        ],
        vec![
            Vector2::new(0.0, 1.0),
            Vector2::new(0.5, 2.0),
            Vector2::new(1.0, 1.0),
        ],
    ];
    let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);

    // bspsurface: (v, 2v(1 - v)(2u - 1) + u), uderivation: (0.0, 4v(1 - v) + 1)
    const N: usize = 100; // sample size
    for i in 0..=N {
        let u = (i as f64) / (N as f64);
        for j in 0..=N {
            let v = (j as f64) / (N as f64);
            assert_near2!(
                bspsurface.uder(u, v),
                Vector2::new(0.0, 4.0 * v * (1.0 - v) + 1.0),
            );
        }
    }
}

#[test]
fn test_vderivation() {
    let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    let ctrl_pts = vec![
        vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(0.5, -1.0),
            Vector2::new(1.0, 0.0),
        ],
        vec![
            Vector2::new(0.0, 1.0),
            Vector2::new(0.5, 2.0),
            Vector2::new(1.0, 1.0),
        ],
    ];
    let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);

    // bspsurface: (v, 2v(1 - v)(2u - 1) + u), vderivation: (1, -2(2u - 1)(2v - 1))
    const N: usize = 100; // sample size
    for i in 0..=N {
        let u = (i as f64) / (N as f64);
        for j in 0..=N {
            let v = (j as f64) / (N as f64);
            assert_near2!(
                bspsurface.vder(u, v),
                Vector2::new(1.0, -2.0 * (2.0 * u - 1.0) * (2.0 * v - 1.0)),
            );
        }
    }
}

#[test]
fn test_uuderivation() {
    let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    let ctrl_pts = vec![
        vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(0.5, -1.0),
            Vector2::new(1.0, 0.0),
        ],
        vec![
            Vector2::new(0.0, 0.5),
            Vector2::new(0.5, 1.0),
            Vector2::new(1.0, 0.5),
        ],
        vec![
            Vector2::new(0.0, 1.0),
            Vector2::new(0.5, 2.0),
            Vector2::new(1.0, 1.0),
        ],
    ];
    let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);

    // bspsurface: (v, 2 u^2 v^2 - 2 u^2 v - 6 u v^2 + 6uv + 2v^2 + u - 2v)
    // uuder: (0, 4v(v - 1))
    const N: usize = 100; // sample size
    for i in 0..=N {
        let u = (i as f64) / (N as f64);
        for j in 0..=N {
            let v = (j as f64) / (N as f64);
            assert_near2!(
                bspsurface.uuder(u, v),
                Vector2::new(0.0, 4.0 * v * (v - 1.0)),
            );
        }
    }
}

#[test]
fn test_uvderivation() {
    let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    let ctrl_pts = vec![
        vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(0.5, -1.0),
            Vector2::new(1.0, 0.0),
        ],
        vec![
            Vector2::new(0.0, 0.5),
            Vector2::new(0.5, 1.0),
            Vector2::new(1.0, 0.5),
        ],
        vec![
            Vector2::new(0.0, 1.0),
            Vector2::new(0.5, 2.0),
            Vector2::new(1.0, 1.0),
        ],
    ];
    let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);

    // bspsurface: (v, 2 u^2 v^2 - 2 u^2 v - 6 u v^2 + 6uv + 2v^2 + u - 2v)
    // uvder: (0, 8uv - 4u - 12v + 6)
    const N: usize = 100; // sample size
    for i in 0..=N {
        let u = (i as f64) / (N as f64);
        for j in 0..=N {
            let v = (j as f64) / (N as f64);
            assert_near2!(
                bspsurface.uvder(u, v),
                Vector2::new(0.0, 8.0 * u * v - 4.0 * u - 12.0 * v + 6.0),
            );
        }
    }
}

#[test]
fn test_vvderivation() {
    let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    let ctrl_pts = vec![
        vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(0.5, -1.0),
            Vector2::new(1.0, 0.0),
        ],
        vec![
            Vector2::new(0.0, 0.5),
            Vector2::new(0.5, 1.0),
            Vector2::new(1.0, 0.5),
        ],
        vec![
            Vector2::new(0.0, 1.0),
            Vector2::new(0.5, 2.0),
            Vector2::new(1.0, 1.0),
        ],
    ];
    let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);

    // bspsurface: (v, 2 u^2 v^2 - 2 u^2 v - 6 u v^2 + 6uv + 2v^2 + u - 2v)
    // vvder: (0, 4(u^2 - 3u + 1))
    const N: usize = 100; // sample size
    for i in 0..=N {
        let u = (i as f64) / (N as f64);
        for j in 0..=N {
            let v = (j as f64) / (N as f64);
            assert_near2!(
                bspsurface.vvder(u, v),
                Vector2::new(0.0, 4.0 * (u * u - 3.0 * u + 1.0)),
            );
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
        u_derivate in prop::bool::ANY,
    ) {
        prop_assume!(udegree > m + 1);
        prop_assume!(vdegree > n + 1);
        let uknot_vec = KnotVec::uniform_knot(udegree, udiv);
        let vknot_vec = KnotVec::uniform_knot(vdegree, vdiv);
        let control_points = pts[..udegree + udiv]
            .iter()
            .map(|vec| {
                vec[..vdegree + vdiv]
                    .iter()
                    .map(|&p| Point3::from(p))
                    .collect()
            })
            .collect::<Vec<Vec<_>>>();
        let bsp = BSplineSurface::new((uknot_vec, vknot_vec), control_points);

        const EPS: f64 = 1.0e-4;
        let (der0, der1) = if u_derivate {
            let der0 = bsp.der_mn(u, v, m + 1, n);
            let der1 = (bsp.der_mn(u + EPS, v, m, n) - bsp.der_mn(u - EPS, v, m, n)) / (2.0 * EPS);
            (der0, der1)
        } else {
            let der0 = bsp.der_mn(u, v, m, n + 1);
            let der1 = (bsp.der_mn(u, v + EPS, m, n) - bsp.der_mn(u, v - EPS, m, n)) / (2.0 * EPS);
            (der0, der1)
        };
        prop_assert!((der0 - der1).magnitude() < 0.01 * der0.magnitude());
    }
}
