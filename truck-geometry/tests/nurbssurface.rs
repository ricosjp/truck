use proptest::prelude::*;
use truck_geometry::prelude::*;

#[test]
fn nurbs_sphere() {
    let knot_vec = KnotVec::bezier_knot(2);
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
    let surface = NurbsSurface::new(BSplineSurface::new(
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
        let uknot_vec = KnotVec::uniform_knot(udegree, udiv);
        let vknot_vec = KnotVec::uniform_knot(vdegree, vdiv);
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
        let bsp = NurbsSurface::new(BSplineSurface::new((uknot_vec, vknot_vec), control_points));

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
