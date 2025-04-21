use proptest::prelude::*;
use truck_geometry::prelude::*;

#[test]
fn nurbs_circle() {
    let knot_vec = KnotVec::bezier_knot(2);
    let control_points = vec![
        Vector3::new(1.0, 0.0, 1.0),
        Vector3::new(1.0, 1.0, 1.0),
        Vector3::new(0.0, 2.0, 2.0),
    ];
    let curve = NurbsCurve::new(BSplineCurve::new(knot_vec, control_points));

    const N: usize = 10;
    for i in 0..=N {
        let t = i as f64 / N as f64;
        let p = curve.subs(t).to_vec();
        assert_near!(p.magnitude(), 1.0);
        let der = curve.der(t);
        assert!(p.dot(der).so_small());
    }
}

proptest! {
    #[test]
    fn test_der_n(
        t in 0f64..=1.0,
        n in 0usize..=4,
        degree in 2usize..=6,
        div in 1usize..=10,
        pts in prop::array::uniform16(prop::array::uniform3(-10f64..=10.0)),
        weights in prop::array::uniform16(0.5f64..=10.0),
    ) {
        prop_assume!(degree > n + 1);
        let knot_vec = KnotVec::uniform_knot(degree, div);
        let control_points = pts[0..degree + div]
            .iter()
            .zip(weights)
            .map(|(&p, w)| Vector4::new(p[0], p[1], p[2], w))
            .collect::<Vec<_>>();
        let bsp = NurbsCurve::new(BSplineCurve::new(knot_vec, control_points));

        const EPS: f64 = 1.0e-4;
        let der0 = bsp.der_n(t, n + 1);
        let der1 = (bsp.der_n(t + EPS, n) - bsp.der_n(t - EPS, n)) / (2.0 * EPS);
        prop_assert!((der0 - der1).magnitude() <= 0.01 * der0.magnitude());
    }
}

fn exec_concat_positive_test(
    v0: [[f64; 3]; 8],
    v1: [f64; 8],
    t: f64,
    w: f64,
) -> std::result::Result<(), TestCaseError> {
    let mut part0 = NurbsCurve::new(BSplineCurve::new(
        KnotVec::uniform_knot(4, 4),
        v0.into_iter()
            .zip(v1)
            .map(|(v0, v1)| Vector3::from(v0).extend(v1))
            .collect(),
    ));
    let mut part1 = part0.cut(t);
    part1.transform_control_points(|vec| *vec *= w);
    prop_assert_near!(part0.back(), part1.front());
    concat_random_test(&part0, &part1, 10);
    Ok(())
}

proptest! {
    #[test]
    fn concat_positive_test(
        v0 in prop::array::uniform8(prop::array::uniform3(-10f64..10f64)),
        v1 in prop::array::uniform8(0.5f64..=10f64),
        t in 0f64..=1f64,
        w in -10f64..=10f64,
    ) {
        exec_concat_positive_test(v0, v1, t, w)?;
    }
}

#[test]
fn test_parameter_division() {
    let knot_vec = KnotVec::uniform_knot(2, 3);
    let ctrl_pts = vec![
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(2.0, 0.0, 0.0, 2.0),
        Vector4::new(0.0, 3.0, 0.0, 3.0),
        Vector4::new(0.0, 0.0, 2.0, 2.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
    ];
    let curve = NurbsCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    let tol = 0.01;
    let (div, pts) = curve.parameter_division(curve.range_tuple(), tol * 0.5);
    let knot_vec = curve.knot_vec();
    assert_eq!(knot_vec[0], div[0]);
    assert_eq!(knot_vec.range_length(), div.last().unwrap() - div[0]);
    for i in 1..div.len() {
        let pt0 = curve.subs(div[i - 1]);
        assert_eq!(pt0, pts[i - 1]);
        let pt1 = curve.subs(div[i]);
        assert_eq!(pt1, pts[i]);
        let value_middle = pt0.midpoint(pt1);
        let param_middle = curve.subs((div[i - 1] + div[i]) / 2.0);
        let dist = value_middle.distance(param_middle);
        assert!(dist < tol, "large distance: {dist}");
    }
}
