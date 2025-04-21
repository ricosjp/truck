use proptest::prelude::*;
use truck_geometry::prelude::*;

#[test]
fn test_substitution() {
    let knot_vec = KnotVec::from(vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]);
    let ctrl_pts = vec![
        Vector2::new(-1.0, 1.0),
        Vector2::new(0.0, -1.0),
        Vector2::new(1.0, 1.0),
    ];
    let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);

    // bspcurve coincides with (t, t * t) in the range [-1.0..1.0].
    const N: usize = 100; // sample size
    for i in 0..=N {
        let t = -1.0 + 2.0 * (i as f64) / (N as f64);
        assert_near2!(bspcurve.subs(t), Vector2::new(t, t * t));
    }
}

#[test]
fn test_derivation() {
    let knot_vec = KnotVec::bezier_knot(2);
    let ctrl_pts = vec![
        Vector2::new(0.0, 0.0),
        Vector2::new(0.5, 0.0),
        Vector2::new(1.0, 1.0),
    ];
    let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);

    // `bpscurve = (t, t^2), derived = (1, 2t)`
    const N: usize = 100; // sample size
    for i in 0..=N {
        let t = 1.0 / (N as f64) * (i as f64);
        assert_near2!(bspcurve.der(t), Vector2::new(1.0, 2.0 * t));
    }
}

#[test]
fn test_2nd_derivation() {
    let knot_vec = KnotVec::bezier_knot(3);
    let ctrl_pts = vec![
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, 0.0),
    ];
    let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);

    // bpscurve = (4t^3 - 6t^2 + 3t, -3t^2 + 3t), derived2 = (24t - 12, -6)
    const N: usize = 100; // sample size
    for i in 0..=N {
        let t = 1.0 / (N as f64) * (i as f64);
        assert_near2!(bspcurve.der2(t), Vector2::new(24.0 * t - 12.0, -6.0));
    }
}

proptest! {
    #[test]
    fn test_der_n(
        t in 0f64..=1.0,
        n in 0usize..=4,
        degree in 2usize..=6,
        div in 1usize..=10,
        pts in prop::array::uniform16(prop::array::uniform3(-10f64..=10.0))
    ) {
        prop_assume!(degree > n + 1);
        let knot_vec = KnotVec::uniform_knot(degree, div);
        let control_points = pts[0..degree + div]
            .iter()
            .map(|&p| Point3::from(p))
            .collect::<Vec<_>>();
        let bsp = BSplineCurve::new(knot_vec, control_points);

        const EPS: f64 = 1.0e-4;
        let der0 = bsp.der_n(t, n + 1);
        let der1 = (bsp.der_n(t + EPS, n) - bsp.der_n(t - EPS, n)) / (2.0 * EPS);
        prop_assert!((der0 - der1).magnitude() < 0.01 * der0.magnitude());
    }
}

proptest! {
    #[test]
    fn parameter_random_tests(c in prop::array::uniform3(-10f64..10f64)) {
        let curve = BSplineCurve::new(
            KnotVec::uniform_knot(4, 4),
            (0..8).map(|_| Point3::from(c)).collect(),
        );
        truck_geotrait::parameter_transform_random_test(&curve, 10);
        truck_geotrait::cut_random_test(&curve, 10);

        let mut part0 = curve.clone();
        let part1 = part0.cut(0.56);
        truck_geotrait::concat_random_test(&part0, &part1, 10);
    }
}

#[test]
fn concat_negative_test() {
    let curve0 = BSplineCurve::new(
        KnotVec::bezier_knot(1),
        vec![Point2::new(0.0, 0.0), Point2::new(0.0, 1.0)],
    );
    let mut curve1 = BSplineCurve::new(
        KnotVec::bezier_knot(1),
        vec![Point2::new(1.0, 1.0), Point2::new(1.0, 1.0)],
    );
    assert_eq!(
        curve0.try_concat(&curve1),
        Err(ConcatError::DisconnectedParameters(1.0, 0.0))
    );
    curve1.knot_translate(1.0);
    assert_eq!(
        curve0.try_concat(&curve1),
        Err(ConcatError::DisconnectedPoints(
            Point2::new(0.0, 1.0),
            Point2::new(1.0, 1.0)
        ))
    );
}

#[test]
fn test_near_as_curve() {
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        Vector4::new(1.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 1.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 0.0, 0.0),
        Vector4::new(1.0, 0.0, 1.0, 0.0),
        Vector4::new(1.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 0.0),
    ];
    let bspline0 = BSplineCurve::new(knot_vec, control_points.clone());
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.5, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        control_points[0],
        control_points[1],
        control_points[2],
        control_points[3] * (5.0 / 6.0) + control_points[2] * (1.0 / 6.0),
        control_points[4] * 0.5 + control_points[3] * 0.5,
        control_points[5] * (1.0 / 6.0) + control_points[4] * (5.0 / 6.0),
        control_points[5],
        control_points[6],
        control_points[7],
    ];
    let bspline1 = BSplineCurve::new(knot_vec, control_points);
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        Vector4::new(1.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 1.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.01, 0.0, 0.0),
        Vector4::new(1.0, 0.0, 1.0, 0.0),
        Vector4::new(1.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 0.0),
    ];
    let bspline2 = BSplineCurve::new(knot_vec, control_points);
    assert!(bspline0.near_as_curve(&bspline1));
    assert!(!bspline0.near_as_curve(&bspline2));
}

#[test]
fn test_parameter_division() {
    let knot_vec = KnotVec::uniform_knot(2, 3);
    let ctrl_pts = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 1.0),
    ];
    let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    let tol = 0.01;
    let (div, pts) = bspcurve.parameter_division(bspcurve.range_tuple(), tol);
    let knot_vec = bspcurve.knot_vec();
    assert_eq!(knot_vec[0], div[0]);
    assert_eq!(knot_vec.range_length(), div.last().unwrap() - div[0]);
    for i in 1..div.len() {
        let pt0 = bspcurve.subs(div[i - 1]);
        assert_eq!(pt0, pts[i - 1]);
        let pt1 = bspcurve.subs(div[i]);
        assert_eq!(pt1, pts[i]);
        let value_middle = pt0 + (pt1 - pt0) / 2.0;
        let param_middle = bspcurve.subs((div[i - 1] + div[i]) / 2.0);
        assert!(value_middle.distance(param_middle) < tol);
    }
}

#[test]
fn test_invert() {
    let knot_vec = KnotVec::uniform_knot(2, 2);
    let ctrl_pts = vec![
        Vector2::new(1.0, 2.0),
        Vector2::new(2.0, 3.0),
        Vector2::new(3.0, 4.0),
        Vector2::new(4.0, 5.0),
    ];
    let bspcurve0 = BSplineCurve::new(knot_vec, ctrl_pts);
    let mut bspcurve1 = bspcurve0.clone();
    bspcurve1.invert();

    const N: usize = 100;
    for i in 0..=N {
        let t = (i as f64) / (N as f64);
        assert_near2!(bspcurve0.subs(t), bspcurve1.subs(1.0 - t));
    }
}

#[test]
#[ignore]
fn bsp_bench() {
    const N: usize = 1000000;
    let instant = std::time::Instant::now();

    let knot_vec = KnotVec::uniform_knot(3, 10);
    for i in 0..=N {
        let t = i as f64 / N as f64;
        let x = knot_vec.try_bspline_basis_functions(3, 0, t).unwrap();
        assert_near!(x.iter().sum::<f64>(), 1.0);
    }

    println!("bsp-bench: {}ms", instant.elapsed().as_millis());
}
