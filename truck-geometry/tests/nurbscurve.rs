use proptest::prelude::*;
use truck_geometry::prelude::*;

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
