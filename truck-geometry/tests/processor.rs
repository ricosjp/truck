use proptest::prelude::*;
use truck_geometry::prelude::*;
type PResult = std::result::Result<(), TestCaseError>;

fn exec_compatible_with_bspcurve(ycoords: [f64; 7], mat: [f64; 9]) -> PResult {
    let knot_vec = KnotVec::uniform_knot(3, 4);
    let control_points: Vec<Point3> = ycoords
        .into_iter()
        .enumerate()
        .map(|(i, y)| Point3::new(i as f64, y, 0.0))
        .collect();
    let mut curve = BSplineCurve::new(knot_vec, control_points);
    let mut processor = Processor::new(curve.clone());
    let mat = *<&Matrix3>::from(&mat);
    prop_assume!(!mat.determinant().so_small(), "omitted: {:?}", mat);

    curve.transform_by(mat);
    processor.transform_by(mat);
    prop_assert_eq!(curve.parameter_range(), processor.parameter_range());

    const N: usize = 100;
    for i in 0..=N {
        let t = i as f64 / N as f64;
        prop_assert_near!(ParametricCurve::subs(&curve, t), processor.subs(t));
        prop_assert_near!(ParametricCurve::der(&curve, t), processor.der(t));
        prop_assert_near!(ParametricCurve::der2(&curve, t), processor.der2(t));
    }

    curve.invert();
    processor.invert();
    prop_assert_eq!(curve.parameter_range(), processor.parameter_range());
    for i in 0..=N {
        let t = i as f64 / N as f64;
        prop_assert_near!(ParametricCurve::subs(&curve, t), processor.subs(t));
        prop_assert_near!(ParametricCurve::der(&curve, t), processor.der(t));
        prop_assert_near!(ParametricCurve::der2(&curve, t), processor.der2(t));
    }
    Ok(())
}

proptest! {
    #[test]
    fn compatible_with_bspcurve(
        ycoords in prop::array::uniform7(-10f64..10f64),
        mat in prop::array::uniform9(-2f64..2f64),
    ) {
        exec_compatible_with_bspcurve(ycoords, mat)?;
    }
}

fn exec_compatible_with_bspsurface(
    ycoords: [[f64; 7]; 7],
    mat: [f64; 9],
    (u, v): (f64, f64),
) -> PResult {
    let knot_vec = KnotVec::uniform_knot(3, 4);
    let knot_vecs = (knot_vec.clone(), knot_vec);
    let control_points: Vec<Vec<Point3>> = ycoords
        .into_iter()
        .enumerate()
        .map(|(i, arr)| {
            arr.into_iter()
                .enumerate()
                .map(|(j, y)| Point3::new(i as f64, j as f64, y))
                .collect()
        })
        .collect();

    let mut surface = BSplineSurface::new(knot_vecs, control_points);
    let mut processor = Processor::new(surface.clone());
    let mat = *<&Matrix3>::from(&mat);
    prop_assume!(!mat.determinant().so_small(), "omitted: {:?}", mat);

    surface.transform_by(mat);
    processor.transform_by(mat);
    assert_eq!(surface.range_tuple(), processor.range_tuple());

    let pt0 = ParametricSurface::subs(&surface, u, v);
    let pt1 = processor.subs(u, v);
    prop_assert_near!(pt0, pt1);
    let uder0 = surface.uder(u, v);
    let uder1 = processor.uder(u, v);
    prop_assert_near!(uder0, uder1);
    let vder0 = surface.vder(u, v);
    let vder1 = processor.vder(u, v);
    prop_assert_near!(vder0, vder1);
    let uuder0 = surface.uuder(u, v);
    let uuder1 = processor.uuder(u, v);
    prop_assert_near!(uuder0, uuder1);
    let uvder0 = surface.uvder(u, v);
    let uvder1 = processor.uvder(u, v);
    prop_assert_near!(uvder0, uvder1);
    let vvder0 = surface.vvder(u, v);
    let vvder1 = processor.vvder(u, v);
    prop_assert_near!(vvder0, vvder1);
    let n0 = surface.normal(u, v);
    let n1 = processor.normal(u, v);
    prop_assert_near!(n0, n1);

    surface.swap_axes();
    processor.invert();
    prop_assert_eq!(surface.range_tuple(), processor.range_tuple());
    let pt0 = ParametricSurface::subs(&surface, u, v);
    let pt1 = processor.subs(u, v);
    prop_assert_near!(pt0, pt1);
    let uder0 = surface.uder(u, v);
    let uder1 = processor.uder(u, v);
    prop_assert_near!(uder0, uder1);
    let vder0 = surface.vder(u, v);
    let vder1 = processor.vder(u, v);
    prop_assert_near!(vder0, vder1);
    let uuder0 = surface.uuder(u, v);
    let uuder1 = processor.uuder(u, v);
    prop_assert_near!(uuder0, uuder1);
    let uvder0 = surface.uvder(u, v);
    let uvder1 = processor.uvder(u, v);
    prop_assert_near!(uvder0, uvder1);
    let vvder0 = surface.vvder(u, v);
    let vvder1 = processor.vvder(u, v);
    prop_assert_near!(vvder0, vvder1);
    let n0 = surface.normal(u, v);
    let n1 = processor.normal(u, v);
    prop_assert_near!(n0, n1);
    Ok(())
}

proptest! {
    #[test]
    fn compatible_with_bspsurface(
        ycoords in prop::array::uniform7(prop::array::uniform7(-10f64..10f64)),
        mat in prop::array::uniform9(-2f64..2f64),
        (u, v) in (0f64..=1f64, 0f64..=1f64),
    ) {
        exec_compatible_with_bspsurface(ycoords, mat, (u, v))?;
    }
}
