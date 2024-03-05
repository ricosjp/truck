#![allow(clippy::too_many_arguments)]

use proptest::*;
use ruststep::{ast::DataSection, tables::*};
use std::{f64::consts::PI, str::FromStr};
use truck_geometry::prelude as truck;
use truck_stepio::{
    out::*,
    r#in::{alias::*, *},
};

fn float_to_str(x: f64) -> String {
    if f64::abs(x) < 1.0e-6 {
        "0.0".to_string()
    } else if f64::abs(x) < 1.0e-2 && x != 0.0 {
        format!("{x:.7E}")
    } else {
        format!("{x:?}")
    }
}

/// create uniform unit vector from [0.0f64..1.0f64; 2]
fn dir_from_array(arr: [f64; 2]) -> Vector3 {
    let z = 2.0 * arr[1] - 1.0;
    let theta = 2.0 * PI * arr[0];
    let r = f64::sqrt(f64::max(1.0 - z * z, 0.0));
    Vector3::new(r * f64::cos(theta), r * f64::sin(theta), z)
}

fn step_to_entity<THolder>(step_str: &str) -> THolder::Owned
where
    THolder: Holder<Table = Table>,
    Table: EntityTable<THolder>, {
    let data_section = DataSection::from_str(step_str).unwrap();
    let table = Table::from_data_section(&data_section);
    EntityTable::<THolder>::get_owned(&table, 1).unwrap()
}

fn exec_test_near<THolder, T>(ans: T, step_str: &str)
where
    THolder: Holder<Table = Table>,
    Table: EntityTable<THolder>,
    T: for<'a> From<&'a THolder::Owned> + std::fmt::Debug + Tolerance, {
    let entity = step_to_entity(step_str);
    let res = T::from(&entity);
    assert_near!(res, ans);
}

fn exec_cartesian_point(coord: [f64; 3]) {
    let pt = Point2::new(coord[0], coord[1]);
    exec_test_near::<CartesianPointHolder, Point2>(
        pt,
        &format!("DATA;{}ENDSEC;", truck_stepio::out::StepDisplay::new(pt, 1)),
    );
    let pt = Point3::from(coord);
    exec_test_near::<CartesianPointHolder, Point3>(
        pt,
        &format!("DATA;{}ENDSEC;", truck_stepio::out::StepDisplay::new(pt, 1)),
    );
}

proptest! {
    #[test]
    fn cartesian_point(coord in array::uniform3(-100.0f64..100.0f64)) {
        exec_cartesian_point(coord)
    }
}

fn exec_direction(dir_array: [f64; 2]) {
    let theta = 2.0 * PI * dir_array[0];
    let vec = Vector2::new(f64::cos(theta), f64::sin(theta));
    exec_test_near::<DirectionHolder, Vector2>(
        vec,
        &format!(
            "DATA;#1 = DIRECTION('', ({}, {}));ENDSEC;",
            float_to_str(vec[0]),
            float_to_str(vec[1])
        ),
    );
    let vec = dir_from_array(dir_array);
    exec_test_near::<DirectionHolder, Vector3>(
        vec,
        &format!(
            "DATA;#1 = DIRECTION('', ({}, {}, {}));ENDSEC;",
            float_to_str(vec[0]),
            float_to_str(vec[1]),
            float_to_str(vec[2])
        ),
    );
}

proptest! {
    #[test]
    fn direction(dir_array in array::uniform2(0.0f64..1.0)) {
        exec_direction(dir_array)
    }
}

fn exec_vector(elem: [f64; 3]) {
    let vec = Vector2::new(elem[0], elem[1]);
    exec_test_near::<VectorHolder, Vector2>(
        vec,
        &format!("DATA;{}ENDSEC;", StepDisplay::new(vec, 1)),
    );
    let vec = Vector3::from(elem);
    exec_test_near::<VectorHolder, Vector3>(
        vec,
        &format!("DATA;{}ENDSEC;", StepDisplay::new(vec, 1)),
    );
}

proptest! {
    #[test]
    fn vector(elem in array::uniform3(-100.0f64..100.0f64)) {
        exec_vector(elem)
    }
}

fn exec_placement(org_coord: [f64; 3]) {
    let org = Point2::new(org_coord[0], org_coord[1]);
    exec_test_near::<PlacementHolder, Point2>(
        org,
        &format!(
            "DATA;#1 = PLACEMENT('', #2);{}ENDSEC;",
            StepDisplay::new(org, 2)
        ),
    );
    let org = Point3::from(org_coord);
    exec_test_near::<PlacementHolder, Point3>(
        org,
        &format!(
            "DATA;#1 = PLACEMENT('', #2);{}ENDSEC;",
            StepDisplay::new(org, 2)
        ),
    );
}

proptest! {
    #[test]
    fn placement(org_coord in array::uniform3(-100.0f64..100.0f64)) {
        exec_placement(org_coord)
    }
}

fn exec_axis1_placement(org_coord: [f64; 3], dir_array: [f64; 2]) {
    let p = Point2::new(org_coord[0], org_coord[1]);
    let theta = 2.0 * PI * dir_array[0];
    let dir = Vector2::new(f64::cos(theta), f64::sin(theta));
    let step_str = format!(
        "DATA;#1 = AXIS1_PLACEMENT('', #2, #3);{}{}ENDSEC;",
        StepDisplay::new(p, 2),
        StepDisplay::new(VectorAsDirection(dir), 3)
    );
    let placement = step_to_entity::<Axis1PlacementHolder>(&step_str);
    assert_near!(p, Point2::from(&placement.location));
    assert_near!(dir, placement.direction().truncate());

    let p = Point3::from(org_coord);
    let dir = dir_from_array(dir_array);
    let step_str = format!(
        "DATA;#1 = AXIS1_PLACEMENT('', #2, #3);{}{}ENDSEC;",
        StepDisplay::new(p, 2),
        StepDisplay::new(VectorAsDirection(dir), 3)
    );
    let placement = step_to_entity::<Axis1PlacementHolder>(&step_str);
    assert_near!(p, Point3::from(&placement.location));
    assert_near!(dir, placement.direction());
}

proptest! {
    #[test]
    fn axis1_placement(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0),
    ) {
        exec_axis1_placement(org_coord, dir_array)
    }
}

fn exec_axis2_placement2d(org_coord: [f64; 2], theta: f64) {
    let origin = Point2::from(org_coord);
    let dir = Vector2::new(f64::cos(theta), f64::sin(theta));
    let step_str = format!(
        "DATA;#1 = AXIS2_PLACEMENT_2D('', #2, #3);{}{}ENDSEC;",
        StepDisplay::new(origin, 2),
        StepDisplay::new(VectorAsDirection(dir), 3),
    );
    let placement = step_to_entity::<Axis2Placement2dHolder>(&step_str);
    let res: Matrix3 = (&placement).into();
    let n = Vector2::new(-dir.y, dir.x);
    let ans = Matrix3::from_cols(dir.extend(0.0), n.extend(0.0), origin.to_vec().extend(1.0));
    assert_near!(res, ans);
}

proptest! {
    #[test]
    fn axis2_placement_2d(
        org_coord in array::uniform2(-100.0f64..100.0f64),
        theta in 0.0f64..2.0 * PI,
    ) {
        exec_axis2_placement2d(org_coord, theta)
    }
}

fn exec_axis2_placement3d(org_coord: [f64; 3], dir_array: [f64; 2], ref_dir_array: [f64; 2]) {
    let p = Point3::from(org_coord);
    let z = dir_from_array(dir_array);
    let ref_dir = dir_from_array(ref_dir_array);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let step_str = format!(
        "DATA;#1 = AXIS2_PLACEMENT_3D('', #2, #3, #4);{}{}{}ENDSEC;",
        StepDisplay::new(p, 2),
        StepDisplay::new(VectorAsDirection(z), 3),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 4),
    );
    let placement = step_to_entity::<Axis2Placement3dHolder>(&step_str);
    let res: Matrix4 = (&placement).into();
    let ans = Matrix4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        p.to_vec().extend(1.0),
    );
    assert_near!(res, ans);
}

proptest! {
    #[test]
    fn axis2_placement_3d(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0f64),
        ref_dir_array in array::uniform2(0.0f64..1.0f64),
    ) {
        exec_axis2_placement3d(org_coord, dir_array, ref_dir_array)
    }
}

fn exec_line(org_coord: [f64; 3], vec_elem: [f64; 3]) {
    let p = Point3::from(org_coord);
    let v = Vector3::from(vec_elem);
    let q = p + v;
    let step_str = format!(
        "DATA;#1 = LINE('', #2, #3);{}{}ENDSEC;",
        StepDisplay::new(p, 2),
        StepDisplay::new(v, 3),
    );
    let line = step_to_entity::<LineHolder>(&step_str);
    let res: truck::Line<Point3> = (&line).into();
    let ans = truck::Line(p, q);
    assert_near!(res.0, ans.0);
    assert_near!(res.1, ans.1);
}

proptest! {
    #[test]
    fn line(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        vec_elem in array::uniform3(-100.0f64..100.0f64),
    ) {
        exec_line(org_coord, vec_elem)
    }
}

fn exec_polyline(length: usize, coords: Vec<[f64; 3]>) {
    let p = coords
        .into_iter()
        .take(length)
        .map(Point3::from)
        .collect::<Vec<_>>();
    let point_displays = p
        .iter()
        .enumerate()
        .map(|(idx, p)| StepDisplay::new(p, 2 + idx).to_string())
        .collect::<Vec<_>>()
        .concat();
    let index_slice = (0..length).map(|idx| 2 + idx);
    let step_str = format!(
        "DATA;#1 = POLYLINE('', {});{}ENDSEC;",
        IndexSliceDisplay(index_slice),
        point_displays
    );
    let polyline = step_to_entity::<PolylineHolder>(&step_str);
    let tpoly: PolylineCurve<Point3> = (&polyline).into();
    let res = tpoly.0;
    let ans = p;
    assert_eq!(res.len(), ans.len());
    res.into_iter()
        .zip(ans)
        .for_each(|(p, q)| assert_near!(p, q));
}

proptest! {
    #[test]
    fn polyline(
        length in 2usize..100,
        coords in collection::vec(array::uniform3(-100.0f64..100.0f64), 100)
    ) {
        exec_polyline(length, coords)
    }
}

fn exec_b_spline_curve_with_knots(
    knot_len: usize,
    knot_incrs: Vec<f64>,
    knot_mults: Vec<usize>,
    degree: usize,
    ctrlpt_coords: Vec<[f64; 3]>,
) {
    let mut s = 0.0;
    let vec = knot_mults
        .iter()
        .take(knot_len)
        .zip(knot_incrs)
        .flat_map(|(m, x)| {
            s += x;
            std::iter::repeat(s).take(*m)
        })
        .collect::<Vec<f64>>();
    let knots = KnotVec::from(vec);
    let cps = ctrlpt_coords
        .into_iter()
        .take(knots.len() - degree - 1)
        .map(Point3::from)
        .collect::<Vec<_>>();
    let bsp = BSplineCurve::new(knots, cps);
    let step_str = format!("DATA;{}ENDSEC;", StepDisplay::new(&bsp, 1));
    let bsp_step = step_to_entity::<BSplineCurveWithKnotsHolder>(&step_str);
    let res: BSplineCurve<Point3> = (&bsp_step).try_into().unwrap();
    assert_eq!(res.knot_vec().len(), bsp.knot_vec().len());
    assert_eq!(res.control_points().len(), bsp.control_points().len());
    res.knot_vec()
        .iter()
        .zip(bsp.knot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.control_points()
        .iter()
        .zip(bsp.control_points())
        .for_each(|(x, y)| assert_near!(x, y));
}

proptest! {
    #[test]
    fn b_spline_curve_with_knots(
        knot_len in 7usize..20,
        knot_incrs in collection::vec(1.0e-3f64..100.0f64, 20),
        knot_mults in collection::vec(1usize..4usize, 20),
        degree in 2usize..6,
        ctrlpt_coords in collection::vec(array::uniform3(-100.0f64..100.0f64), 80),
    ) {
        exec_b_spline_curve_with_knots(knot_len, knot_incrs, knot_mults, degree, ctrlpt_coords)
    }
}

fn step_bsp_curve_ctrls(points: &[Point3]) -> (String, String) {
    (
        IndexSliceDisplay((0..points.len()).map(|i| 2 + i)).to_string(),
        points
            .iter()
            .enumerate()
            .map(|(i, p)| StepDisplay::new(*p, i + 2).to_string())
            .collect::<Vec<_>>()
            .concat(),
    )
}

fn exec_bezier_curve(degree: usize, ctrlpt_coords: Vec<[f64; 3]>) {
    let points = ctrlpt_coords
        .into_iter()
        .take(degree + 1)
        .map(Point3::from)
        .collect::<Vec<_>>();
    let (step_cps_indices, step_cps) = step_bsp_curve_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = BEZIER_CURVE('', {degree}, {step_cps_indices}, .UNSPECIFIED., .U., .U.);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<BezierCurveHolder>(&step_str);
    let res: BSplineCurve<Point3> = (&bsp_step).try_into().unwrap();
    let ans = BSplineCurve::new(KnotVec::bezier_knot(degree), points);
    assert_eq!(res.knot_vec().len(), ans.knot_vec().len());
    assert_eq!(res.control_points().len(), ans.control_points().len());
    res.knot_vec()
        .iter()
        .zip(ans.knot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.control_points()
        .iter()
        .zip(ans.control_points())
        .for_each(|(x, y)| assert_near!(x, y));
}

proptest! {
    #[test]
    fn bezier_curve(
        degree in 1usize..6,
        ctrlpt_coords in collection::vec(array::uniform3(-100.0f64..100.0f64), 6),
    ) {
        exec_bezier_curve(degree, ctrlpt_coords)
    }
}

fn exec_quasi_uniform_curve(degree: usize, division: usize, ctrlpt_coords: Vec<[f64; 3]>) {
    let mut knots = KnotVec::uniform_knot(degree, division);
    knots.transform(division as f64, 0.0);
    let points = ctrlpt_coords
        .into_iter()
        .take(knots.len() - degree - 1)
        .map(Point3::from)
        .collect::<Vec<_>>();
    let (step_cps_indices, step_cps) = step_bsp_curve_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = QUASI_UNIFORM_CURVE('', {degree}, {step_cps_indices}, .UNSPECIFIED., .U., .U.);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<QuasiUniformCurveHolder>(&step_str);
    let res: BSplineCurve<Point3> = (&bsp_step).try_into().unwrap();
    let ans = BSplineCurve::new(knots, points);
    assert_eq!(res.knot_vec().len(), ans.knot_vec().len());
    assert_eq!(res.control_points().len(), ans.control_points().len());
    res.knot_vec()
        .iter()
        .zip(ans.knot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.control_points()
        .iter()
        .zip(ans.control_points())
        .for_each(|(x, y)| assert_near!(x, y));
}

proptest! {
    #[test]
    fn quasi_uniform_curve(
        degree in 1usize..4,
        division in 3usize..5,
        ctrlpt_coords in collection::vec(array::uniform3(-100.0f64..100.0f64), 20),
    ) {
        exec_quasi_uniform_curve(degree, division, ctrlpt_coords)
    }
}

fn exec_uniform_curve(degree: usize, knot_len: usize, ctrlpt_coords: Vec<[f64; 3]>) {
    let knots = KnotVec::from_iter((0..knot_len).map(|i| i as f64 - degree as f64));
    let points = ctrlpt_coords
        .into_iter()
        .take(knot_len - degree - 1)
        .map(Point3::from)
        .collect::<Vec<_>>();
    let (step_cps_indices, step_cps) = step_bsp_curve_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = UNIFORM_CURVE('', {degree}, {step_cps_indices}, .UNSPECIFIED., .U., .U.);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<UniformCurveHolder>(&step_str);
    let res: BSplineCurve<Point3> = (&bsp_step).try_into().unwrap();
    let ans = BSplineCurve::new(knots, points);
    assert_eq!(res.knot_vec().len(), ans.knot_vec().len());
    assert_eq!(res.control_points().len(), ans.control_points().len());
    res.knot_vec()
        .iter()
        .zip(ans.knot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.control_points()
        .iter()
        .zip(ans.control_points())
        .for_each(|(x, y)| assert_near!(x, y));
}

proptest! {
   #[test]
    fn uniform_curve(
        degree in 1usize..4,
        knot_len in 6usize..10,
        ctrlpt_coords in collection::vec(array::uniform3(-100.0f64..100.0f64), 40),
    ) {
        exec_uniform_curve(degree, knot_len, ctrlpt_coords)
    }
}

fn exec_nurbs_curve_b_spline_with_knots(
    knot_len: usize,
    knot_incrs: Vec<f64>,
    knot_mults: Vec<usize>,
    mut weights: Vec<f64>,
    degree: usize,
    ctrlpt_coords: Vec<[f64; 3]>,
) {
    let mut s = 0.0;
    let vec = knot_mults
        .iter()
        .take(knot_len)
        .zip(knot_incrs)
        .flat_map(|(m, x)| {
            s += x;
            std::iter::repeat(s).take(*m)
        })
        .collect::<Vec<f64>>();
    let knots = KnotVec::from(vec);
    let cps = ctrlpt_coords
        .into_iter()
        .take(knots.len() - degree - 1)
        .map(Point3::from)
        .collect::<Vec<_>>();
    weights.truncate(cps.len());
    let bsp = BSplineCurve::new(knots, cps);
    let nurbs = NurbsCurve::<Vector4>::try_from_bspline_and_weights(bsp, weights).unwrap();
    let step_str = format!("DATA;{}ENDSEC;", StepDisplay::new(&nurbs, 1));
    let nurbs_step = step_to_entity::<RationalBSplineCurveHolder>(&step_str);
    let res: NurbsCurve<Vector4> = (&nurbs_step).try_into().unwrap();
    assert_eq!(res.knot_vec().len(), nurbs.knot_vec().len());
    assert_eq!(res.control_points().len(), nurbs.control_points().len());
    res.knot_vec()
        .iter()
        .zip(nurbs.knot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.control_points()
        .iter()
        .zip(nurbs.control_points())
        .for_each(|(x, y)| assert_near!(x, y));
}

proptest! {
    #[test]
    fn nurbs_curve_b_spline_curve_with_knots(
        knot_len in 7usize..20,
        knot_incrs in collection::vec(1.0e-3f64..100.0f64, 20),
        knot_mults in collection::vec(1usize..4usize, 20),
        weights in collection::vec(0.01f64..100.0, 80),
        degree in 2usize..6,
        ctrlpt_coords in collection::vec(array::uniform3(-100.0f64..100.0f64), 80),
    ) {
        exec_nurbs_curve_b_spline_with_knots(knot_len, knot_incrs, knot_mults, weights, degree, ctrlpt_coords)
    }
}

fn exec_nurbs_curve_bezier_curve(
    degree: usize,
    ctrlpt_coords: Vec<[f64; 3]>,
    mut weights: Vec<f64>,
) {
    let points = ctrlpt_coords
        .into_iter()
        .take(degree + 1)
        .map(Point3::from)
        .collect::<Vec<_>>();
    weights.truncate(points.len());
    let weights_display = SliceDisplay(&weights);
    let (step_cps_indices, step_cps) = step_bsp_curve_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = (
    BEZIER_CURVE()
    BOUNDED_CURVE()
    B_SPLINE_CURVE({degree}, {step_cps_indices}, .UNSPECIFIED., .U., .U.)
    CURVE()
    GEOMETRIC_REPRESENTATION_ITEM()
    RATIONAL_B_SPLINE_CURVE({weights_display})
    REPRESENTATION_ITEM('')
);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<RationalBSplineCurveHolder>(&step_str);
    let res: NurbsCurve<Vector4> = (&bsp_step).try_into().unwrap();
    let bsp = BSplineCurve::new(KnotVec::bezier_knot(degree), points);
    let ans = NurbsCurve::<Vector4>::try_from_bspline_and_weights(bsp, weights).unwrap();
    assert_eq!(res.knot_vec().len(), ans.knot_vec().len());
    assert_eq!(res.control_points().len(), ans.control_points().len());
    res.knot_vec()
        .iter()
        .zip(ans.knot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.control_points()
        .iter()
        .zip(ans.control_points())
        .for_each(|(x, y)| assert_near!(x, y));
}

proptest! {
    #[test]
    fn nurbs_curve_bezier_curve(
        degree in 1usize..6,
        ctrlpt_coords in collection::vec(array::uniform3(-100.0f64..100.0f64), 6),
        weights in collection::vec(0.01f64..100.0, 6),
    ) {
        exec_nurbs_curve_bezier_curve(degree, ctrlpt_coords, weights)
    }
}

fn exec_nurbs_curve_quasi_uniform_curve(
    degree: usize,
    division: usize,
    ctrlpt_coords: Vec<[f64; 3]>,
    mut weights: Vec<f64>,
) {
    let mut knots = KnotVec::uniform_knot(degree, division);
    knots.transform(division as f64, 0.0);
    let points = ctrlpt_coords
        .into_iter()
        .take(knots.len() - degree - 1)
        .map(Point3::from)
        .collect::<Vec<_>>();
    weights.truncate(points.len());
    let weights_display = SliceDisplay(&weights);
    let (step_cps_indices, step_cps) = step_bsp_curve_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = (
    BOUNDED_CURVE()
    B_SPLINE_CURVE({degree}, {step_cps_indices}, .UNSPECIFIED., .U., .U.)
    CURVE()
    GEOMETRIC_REPRESENTATION_ITEM()
    QUASI_UNIFORM_CURVE()
    RATIONAL_B_SPLINE_CURVE({weights_display})
    REPRESENTATION_ITEM('')
);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<RationalBSplineCurveHolder>(&step_str);
    let res: NurbsCurve<Vector4> = (&bsp_step).try_into().unwrap();
    let bsp = BSplineCurve::new(knots, points);
    let ans = NurbsCurve::<Vector4>::try_from_bspline_and_weights(bsp, weights).unwrap();
    assert_eq!(res.knot_vec().len(), ans.knot_vec().len());
    assert_eq!(res.control_points().len(), ans.control_points().len());
    res.knot_vec()
        .iter()
        .zip(ans.knot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.control_points()
        .iter()
        .zip(ans.control_points())
        .for_each(|(x, y)| assert_near!(x, y));
}

proptest! {
    #[test]
    fn nurbs_curve_quasi_uniform_curve(
        degree in 1usize..4,
        division in 3usize..5,
        ctrlpt_coords in collection::vec(array::uniform3(-100.0f64..100.0f64), 20),
        weights in collection::vec(0.01f64..100.0, 20),
    ) {
        exec_nurbs_curve_quasi_uniform_curve(degree, division, ctrlpt_coords, weights)
    }
}

fn exec_nurbs_curve_uniform_curve(
    degree: usize,
    knot_len: usize,
    ctrlpt_coords: Vec<[f64; 3]>,
    mut weights: Vec<f64>,
) {
    let knots = KnotVec::from_iter((0..knot_len).map(|i| i as f64 - degree as f64));
    let points = ctrlpt_coords
        .into_iter()
        .take(knot_len - degree - 1)
        .map(Point3::from)
        .collect::<Vec<_>>();
    weights.truncate(points.len());
    let weights_display = SliceDisplay(&weights);
    let (step_cps_indices, step_cps) = step_bsp_curve_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = (
    BOUNDED_CURVE()
    B_SPLINE_CURVE({degree}, {step_cps_indices}, .UNSPECIFIED., .U., .U.)
    CURVE()
    GEOMETRIC_REPRESENTATION_ITEM()
    RATIONAL_B_SPLINE_CURVE({weights_display})
    REPRESENTATION_ITEM('')
    UNIFORM_CURVE()
);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<RationalBSplineCurveHolder>(&step_str);
    let res: NurbsCurve<Vector4> = (&bsp_step).try_into().unwrap();
    let bsp = BSplineCurve::new(knots, points);
    let ans = NurbsCurve::<Vector4>::try_from_bspline_and_weights(bsp, weights).unwrap();
    assert_eq!(res.knot_vec().len(), ans.knot_vec().len());
    assert_eq!(res.control_points().len(), ans.control_points().len());
    res.knot_vec()
        .iter()
        .zip(ans.knot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.control_points()
        .iter()
        .zip(ans.control_points())
        .for_each(|(x, y)| assert_near!(x, y));
}

proptest! {
   #[test]
    fn nurbs_curve_uniform_curve(
        degree in 1usize..4,
        knot_len in 6usize..10,
        ctrlpt_coords in collection::vec(array::uniform3(-100.0f64..100.0f64), 40),
        weights in collection::vec(0.01f64..100.0, 40),
    ) {
        exec_nurbs_curve_uniform_curve(degree, knot_len, ctrlpt_coords, weights)
    }
}

fn exec_circle(org_coord: [f64; 3], dir_array: [f64; 2], ref_dir_array: [f64; 2], radius: f64) {
    let origin = Point3::from(org_coord);
    let z = dir_from_array(dir_array);
    let ref_dir = dir_from_array(ref_dir_array);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let step_str = format!(
        "DATA; #1 = CIRCLE('', #2, {radius}); #2 = AXIS2_PLACEMENT_3D('', #3, #4, #5); {}{}{}ENDSEC;",
        StepDisplay::new(origin, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_circle = step_to_entity::<CircleHolder>(&step_str);
    let ellipse: alias::Ellipse<Point3, Matrix4> = (&step_circle).try_into().unwrap();
    let mat = Matrix4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        origin.to_vec().extend(1.0),
    );
    (0..10).for_each(|i| {
        let t = 2.0 * PI * i as f64 / 10.0;
        let p = Point3::new(radius * f64::cos(t), radius * f64::sin(t), 0.0);
        assert_near!(ellipse.subs(t), mat.transform_point(p));
    });
}

proptest! {
    #[test]
    fn circle(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0),
        ref_dir_array in array::uniform2(0.0f64..1.0),
        radius in 1.0e-2f64..100.0,
    ) {
        exec_circle(org_coord, dir_array, ref_dir_array, radius)
    }
}

fn exec_ellipse(org_coord: [f64; 3], dir_array: [f64; 2], ref_dir_array: [f64; 2], radius: [f64; 2]) {
    let origin = Point3::from(org_coord);
    let z = dir_from_array(dir_array);
    let ref_dir = dir_from_array(ref_dir_array);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let step_str = format!(
        "DATA; #1 = ELLIPSE('', #2, {}, {}); #2 = AXIS2_PLACEMENT_3D('', #3, #4, #5); {}{}{}ENDSEC;",
        FloatDisplay(radius[0]),
        FloatDisplay(radius[1]),
        StepDisplay::new(origin, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_ellipse = step_to_entity::<EllipseHolder>(&step_str);
    let ellipse: alias::Ellipse<Point3, Matrix4> = (&step_ellipse).try_into().unwrap();
    let mat = Matrix4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        origin.to_vec().extend(1.0),
    );
    (0..10).for_each(|i| {
        let t = 2.0 * PI * i as f64 / 10.0;
        let p = Point3::new(radius[0] * f64::cos(t), radius[1] * f64::sin(t), 0.0);
        assert_near!(ellipse.subs(t), mat.transform_point(p));
    });
}

proptest! {
    #[test]
    fn ellipse(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0),
        ref_dir_array in array::uniform2(0.0f64..1.0),
        radius in array::uniform2(1.0e-2f64..100.0),
    ) {
        exec_ellipse(org_coord, dir_array, ref_dir_array, radius)
    }
}

fn exec_hyperbola(org_coord: [f64; 3], dir_array: [f64; 2], ref_dir_array: [f64; 2], radius: [f64; 2]) {
    let origin = Point3::from(org_coord);
    let z = dir_from_array(dir_array);
    let ref_dir = dir_from_array(ref_dir_array);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let step_str = format!(
        "DATA; #1 = HYPERBOLA('', #2, {}, {}); #2 = AXIS2_PLACEMENT_3D('', #3, #4, #5); {}{}{}ENDSEC;",
        FloatDisplay(radius[0]),
        FloatDisplay(radius[1]),
        StepDisplay::new(origin, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_hyperbola = step_to_entity::<HyperbolaHolder>(&step_str);
    let hyperbola: alias::Hyperbola<Point3, Matrix4> = (&step_hyperbola).try_into().unwrap();
    let mat = Matrix4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        origin.to_vec().extend(1.0),
    );
    (0..10).for_each(|i| {
        let t = 2.0 * i as f64 / 10.0 - 1.0;
        let p = Point3::new(radius[0] * f64::cosh(t), radius[1] * f64::sinh(t), 0.0);
        assert_near!(hyperbola.subs(t), mat.transform_point(p));
    });
}

proptest! {
    #[test]
    fn hyperbola(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0),
        ref_dir_array in array::uniform2(0.0f64..1.0),
        radius in array::uniform2(1.0e-2f64..100.0),
    ) {
        exec_hyperbola(org_coord, dir_array, ref_dir_array, radius)
    }
}

fn exec_parabola(org_coord: [f64; 3], dir_array: [f64; 2], ref_dir_array: [f64; 2], focal_dist: f64) {
    let origin = Point3::from(org_coord);
    let z = dir_from_array(dir_array);
    let ref_dir = dir_from_array(ref_dir_array);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let step_str = format!(
        "DATA; #1 = PARABOLA('', #2, {}); #2 = AXIS2_PLACEMENT_3D('', #3, #4, #5); {}{}{}ENDSEC;",
        FloatDisplay(focal_dist),
        StepDisplay::new(origin, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_parabola = step_to_entity::<ParabolaHolder>(&step_str);
    let parabola: alias::Parabola<Point3, Matrix4> = (&step_parabola).try_into().unwrap();
    let mat = Matrix4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        origin.to_vec().extend(1.0),
    );
    (0..10).for_each(|i| {
        let t = 2.0 * i as f64 / 10.0 - 1.0;
        let p = Point3::new(focal_dist * t * t, focal_dist * 2.0 * t, 0.0);
        assert_near!(parabola.subs(t), mat.transform_point(p));
    });
}

proptest! {
    #[test]
    fn parabola(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0),
        ref_dir_array in array::uniform2(0.0f64..1.0),
        focal_dist in 0.01f64..100.0,
    ) {
        exec_parabola(org_coord, dir_array, ref_dir_array, focal_dist)
    }
}

fn exec_plane(org_coord: [f64; 3], dir_array: [f64; 2], ref_dir_array: [f64; 2]) {
    let origin = Point3::from(org_coord);
    let z = dir_from_array(dir_array);
    let ref_dir = dir_from_array(ref_dir_array);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let step_str = format!(
        "DATA;
#1 = PLANE('', #2);
#2 = AXIS2_PLACEMENT_3D('', #3, #4, #5);
{}{}{}ENDSEC;",
        StepDisplay::new(origin, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_plane = step_to_entity::<PlaneHolder>(&step_str);
    let plane = truck::Plane::from(&step_plane);
    assert_near!(plane.origin(), origin);
    assert_near!(plane.u_axis(), x);
    assert_near!(plane.v_axis(), y);
}

proptest! {
    #[test]
    fn plane(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0f64),
        ref_dir_array in array::uniform2(0.0f64..1.0f64),
    ) {
        exec_plane(org_coord, dir_array, ref_dir_array)
    }
}

fn exec_spherical_surface(
    org_coord: [f64; 3],
    dir_array: [f64; 2],
    ref_dir_array: [f64; 2],
    radius: f64,
) {
    let p = Point3::from(org_coord);
    let z = dir_from_array(dir_array);
    let ref_dir = dir_from_array(ref_dir_array);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let step_str = format!(
        "DATA;
#1 = SPHERICAL_SURFACE('', #2, {radius});
#2 = AXIS2_PLACEMENT_3D('', #3, #4, #5);
{}{}{}ENDSEC;",
        StepDisplay::new(p, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_sphere = step_to_entity::<ElementarySurfaceAnyHolder>(&step_str);
    let sphere: alias::ElementarySurface = (&step_sphere).into();
    let mat = Matrix4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        p.to_vec().extend(1.0),
    );
    (0..=10)
        .flat_map(move |i| (0..=10).map(move |j| (i, j)))
        .for_each(|(i, j)| {
            let u = 2.0 * PI * i as f64 / 10.0;
            let v = PI * j as f64 / 10.0 - PI / 2.0;
            let res = sphere.subs(u, v);
            let ans = mat.transform_point(Point3::new(
                radius * f64::cos(u) * f64::cos(v),
                radius * f64::sin(u) * f64::cos(v),
                radius * f64::sin(v),
            ));
            assert_near!(res, ans);
        })
}

proptest! {
    #[test]
    fn spherical_surface(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0f64),
        ref_dir_array in array::uniform2(0.0f64..1.0f64),
        radius in 1.0e-2f64..100.0f64
    ) {
        exec_spherical_surface(org_coord, dir_array, ref_dir_array, radius)
    }
}

fn exec_cylindrical_surface(
    org_coord: [f64; 3],
    dir_array: [f64; 2],
    ref_dir_array: [f64; 2],
    radius: f64,
) {
    let p = Point3::from(org_coord);
    let z = dir_from_array(dir_array);
    let ref_dir = dir_from_array(ref_dir_array);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let step_str = format!(
        "DATA;
#1 = CYLINDRICAL_SURFACE('', #2, {radius});
#2 = AXIS2_PLACEMENT_3D('', #3, #4, #5);
{}{}{}ENDSEC;",
        StepDisplay::new(p, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_cylinder = step_to_entity::<ElementarySurfaceAnyHolder>(&step_str);
    let cylinder: alias::ElementarySurface = (&step_cylinder).into();
    let mat = Matrix4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        p.to_vec().extend(1.0),
    );
    (0..=10)
        .flat_map(move |i| (0..=10).map(move |j| (i, j)))
        .for_each(|(i, j)| {
            let u = 2.0 * PI * i as f64 / 10.0;
            let v = j as f64;
            let res = cylinder.subs(u, v);
            let ans =
                mat.transform_point(Point3::new(radius * f64::cos(u), radius * f64::sin(u), v));
            assert_near!(res, ans, "u:{u} v:{v} res:{res:?} ans:{ans:?}");
        })
}

proptest! {
    #[test]
    fn cylindrical_surface(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0f64),
        ref_dir_array in array::uniform2(0.0f64..1.0f64),
        radius in 1.0e-2f64..100.0f64
    ) {
        exec_cylindrical_surface(org_coord, dir_array, ref_dir_array, radius)
    }
}

fn exec_toroidal_surface(
    org_coord: [f64; 3],
    dir_array: [f64; 2],
    ref_dir_array: [f64; 2],
    radii: [f64; 2],
) {
    let p = Point3::from(org_coord);
    let z = dir_from_array(dir_array);
    let ref_dir = dir_from_array(ref_dir_array);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let major_radius = f64::max(radii[0], radii[1]);
    let minor_radius = (f64::min(radii[0], radii[1])) / 2.0;
    let step_str = format!(
        "DATA;
#1 = TOROIDAL_SURFACE('', #2, {major_radius}, {minor_radius});
#2 = AXIS2_PLACEMENT_3D('', #3, #4, #5);
{}{}{}ENDSEC;",
        StepDisplay::new(p, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_toroidal = step_to_entity::<ElementarySurfaceAnyHolder>(&step_str);
    let toroidal: alias::ElementarySurface = (&step_toroidal).into();
    let mat = Matrix4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        p.to_vec().extend(1.0),
    );
    (0..=10)
        .flat_map(move |i| (0..=10).map(move |j| (i, j)))
        .for_each(|(i, j)| {
            let u = 2.0 * PI * i as f64 / 10.0;
            let v = 2.0 * PI * j as f64 / 10.0;
            let res = toroidal.subs(u, v);
            let ans = mat.transform_point(Point3::new(
                (major_radius + minor_radius * f64::cos(v)) * f64::cos(u),
                (major_radius + minor_radius * f64::cos(v)) * f64::sin(u),
                minor_radius * f64::sin(v),
            ));
            assert_near!(res, ans, "u:{u} v:{v} res:{res:?} ans:{ans:?}");
        })
}

proptest! {
    #[test]
    fn toroidal_surface(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0f64),
        ref_dir_array in array::uniform2(0.0f64..1.0f64),
        radii in array::uniform2(1.0e-2f64..100.0f64),
    ) {
        exec_toroidal_surface(org_coord, dir_array, ref_dir_array, radii)
    }
}

fn exec_conical_surface(
    org_coord: [f64; 3],
    dir_array: [f64; 2],
    ref_dir_array: [f64; 2],
    radius: f64,
    semi_angle: f64,
) {
    let p = Point3::from(org_coord);
    let z = dir_from_array(dir_array);
    let ref_dir = dir_from_array(ref_dir_array);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let step_str = format!(
        "DATA;
#1 = CONICAL_SURFACE('', #2, {radius}, {semi_angle});
#2 = AXIS2_PLACEMENT_3D('', #3, #4, #5);
{}{}{}ENDSEC;",
        StepDisplay::new(p, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_conical = step_to_entity::<ConicalSurfaceHolder>(&step_str);
    let conical: alias::ConicalSurface = (&step_conical).into();
    let mat = Matrix4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        p.to_vec().extend(1.0),
    );
    (0..=10)
        .flat_map(move |i| (0..=10).map(move |j| (i, j)))
        .for_each(|(i, j)| {
            let u = 2.0 * PI * i as f64 / 10.0;
            let v = j as f64 / 10.0;
            let tan = f64::tan(semi_angle);
            let res = conical.subs(u, v);
            let ans = mat.transform_point(Point3::new(
                (radius + v * tan) * f64::cos(u),
                (radius + v * tan) * f64::sin(u),
                v,
            ));
            assert_near!(res, ans, "u:{u} v:{v} res:{res:?} ans:{ans:?}");
        })
}

proptest! {
    #[test]
    fn conical_surface(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_array in array::uniform2(0.0f64..1.0f64),
        ref_dir_array in array::uniform2(0.0f64..1.0f64),
        radius in 0.01f64..100.0f64,
        semi_angle in 0.0f64..PI / 2.0,
    ) {
        exec_conical_surface(org_coord, dir_array, ref_dir_array, radius, semi_angle)
    }
}

fn coords_to_points(
    upoints_len: usize,
    vpoints_len: usize,
    coords: Vec<Vec<[f64; 3]>>,
) -> Vec<Vec<Point3>> {
    coords
        .into_iter()
        .take(upoints_len)
        .map(move |vec: Vec<[f64; 3]>| {
            vec.into_iter()
                .take(vpoints_len)
                .map(Point3::from)
                .collect()
        })
        .collect()
}

fn compare_bsp_surfaces(res: &BSplineSurface<Point3>, ans: &BSplineSurface<Point3>) {
    assert_eq!(res.uknot_vec().len(), ans.uknot_vec().len());
    assert_eq!(res.vknot_vec().len(), ans.vknot_vec().len());
    assert_eq!(res.control_points().len(), ans.control_points().len());
    res.uknot_vec()
        .iter()
        .zip(ans.uknot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.vknot_vec()
        .iter()
        .zip(ans.vknot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.control_points()
        .iter()
        .flatten()
        .zip(ans.control_points().iter().flatten())
        .for_each(|(x, y)| assert_near!(x, y));
}

fn compare_nurbs_surfaces(res: &NurbsSurface<Vector4>, ans: &NurbsSurface<Vector4>) {
    assert_eq!(res.uknot_vec().len(), ans.uknot_vec().len());
    assert_eq!(res.vknot_vec().len(), ans.vknot_vec().len());
    assert_eq!(res.control_points().len(), ans.control_points().len());
    res.uknot_vec()
        .iter()
        .zip(ans.uknot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.vknot_vec()
        .iter()
        .zip(ans.vknot_vec())
        .for_each(|(x, y)| assert_near!(x, y));
    res.control_points()
        .iter()
        .flatten()
        .zip(ans.control_points().iter().flatten())
        .for_each(|(x, y)| assert_near!(x, y));
}

fn exec_b_spline_surface_with_knots(
    uknot_len: usize,
    uknot_mults: Vec<usize>,
    uknot_incrs: Vec<f64>,
    udegree: usize,
    vknot_len: usize,
    vknot_mults: Vec<usize>,
    vknot_incrs: Vec<f64>,
    vdegree: usize,
    ctrlpt_coords: Vec<Vec<[f64; 3]>>,
) {
    let mut s = 0.0;
    let uvec = uknot_mults
        .iter()
        .take(uknot_len)
        .zip(uknot_incrs)
        .flat_map(|(m, x)| {
            s += x;
            std::iter::repeat(s).take(*m)
        })
        .collect::<Vec<f64>>();
    let uknots = KnotVec::from(uvec);
    let mut s = 0.0;
    let vvec = vknot_mults
        .iter()
        .take(vknot_len)
        .zip(vknot_incrs)
        .flat_map(|(m, x)| {
            s += x;
            std::iter::repeat(s).take(*m)
        })
        .collect::<Vec<f64>>();
    let vknots = KnotVec::from(vvec);
    let cps = coords_to_points(
        uknots.len() - udegree - 1,
        vknots.len() - vdegree - 1,
        ctrlpt_coords,
    );
    let bsp = BSplineSurface::new((uknots, vknots), cps);
    let step_str = format!("DATA;{}ENDSEC;", StepDisplay::new(&bsp, 1));
    let bsp_step = step_to_entity::<BSplineSurfaceWithKnotsHolder>(&step_str);
    let res: BSplineSurface<Point3> = (&bsp_step).try_into().unwrap();
    compare_bsp_surfaces(&res, &bsp);
}

proptest! {
    #[test]
    fn b_spline_surface_with_knots(
        uknot_len in 7usize..10,
        uknot_mults in collection::vec(1usize..4usize, 10),
        uknot_incrs in collection::vec(1.0e-3f64..100.0f64, 10),
        udegree in 2usize..6,
        vknot_len in 7usize..10,
        vknot_mults in collection::vec(1usize..4usize, 10),
        vknot_incrs in collection::vec(1.0e-3f64..100.0f64, 10),
        vdegree in 2usize..6,
        ctrlpt_coords in collection::vec(collection::vec(array::uniform3(-100.0f64..100.0f64), 40), 40),
    ) {
        exec_b_spline_surface_with_knots(
            uknot_len,
            uknot_mults,
            uknot_incrs,
            udegree,
            vknot_len,
            vknot_mults,
            vknot_incrs,
            vdegree,
            ctrlpt_coords,
        )
    }
}

fn step_bsp_surface_ctrls(points: &[Vec<Point3>]) -> (String, String) {
    let indices = (0..points.len())
        .map(|i| {
            IndexSliceDisplay(
                (0..points[0].len())
                    .map(|j| 2 + i * points[0].len() + j)
                    .collect::<Vec<usize>>(),
            )
        })
        .collect::<Vec<_>>();
    let step_cps_indices = SliceDisplay(&indices).to_string();
    let step_cps = points
        .iter()
        .flatten()
        .enumerate()
        .fold(String::new(), |string, (i, p)| {
            let display: StepDisplay<Point3> = StepDisplay::new(*p, 2 + i);
            string + &display.to_string()
        });
    (step_cps_indices, step_cps)
}

fn exec_bezier_surface([udegree, vdegree]: [usize; 2], ctrlpt_coords: Vec<Vec<[f64; 3]>>) {
    let points = coords_to_points(udegree + 1, vdegree + 1, ctrlpt_coords);
    let (step_cps_indices, step_cps) = step_bsp_surface_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = BEZIER_SURFACE('', {udegree}, {vdegree}, {step_cps_indices}, .UNSPECIFIED., .U., .U., .U.);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<BezierSurfaceHolder>(&step_str);
    let res: BSplineSurface<Point3> = (&bsp_step).try_into().unwrap();
    let ans = BSplineSurface::new(
        (KnotVec::bezier_knot(udegree), KnotVec::bezier_knot(vdegree)),
        points,
    );
    compare_bsp_surfaces(&res, &ans);
}

proptest! {
    #[test]
    fn bezier_surface(
        degrees in array::uniform2(1usize..6),
        ctrlpt_coords in collection::vec(collection::vec(array::uniform3(-100.0f64..100.0f64), 6), 6),
    ) {
        exec_bezier_surface(degrees, ctrlpt_coords)
    }
}

fn exec_quasi_uniform_surface(
    [udegree, vdegree]: [usize; 2],
    [udivision, vdivision]: [usize; 2],
    ctrlpt_coords: Vec<Vec<[f64; 3]>>,
) {
    let mut uknots = KnotVec::uniform_knot(udegree, udivision);
    uknots.transform(udivision as f64, 0.0);
    let mut vknots = KnotVec::uniform_knot(vdegree, vdivision);
    vknots.transform(vdivision as f64, 0.0);
    let points = coords_to_points(
        uknots.len() - udegree - 1,
        vknots.len() - vdegree - 1,
        ctrlpt_coords,
    );
    let (step_cps_indices, step_cps) = step_bsp_surface_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = QUASI_UNIFORM_SURFACE('', {udegree}, {vdegree}, {step_cps_indices}, .UNSPECIFIED., .U., .U., .U.);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<QuasiUniformSurfaceHolder>(&step_str);
    let res: BSplineSurface<Point3> = (&bsp_step).try_into().unwrap();
    let ans = BSplineSurface::new((uknots, vknots), points);
    compare_bsp_surfaces(&res, &ans);
}

proptest! {
    #[test]
    fn quasi_uniform_surface(
        degrees in array::uniform2(1usize..6),
        divisions in array::uniform2(2usize..5),
        ctrlpt_coords in collection::vec(collection::vec(array::uniform3(-100.0f64..100.0f64), 30), 30),
    ) {
        exec_quasi_uniform_surface(degrees, divisions, ctrlpt_coords)
    }
}

fn exec_uniform_surface(
    [udegree, vdegree]: [usize; 2],
    [uknot_len, vknot_len]: [usize; 2],
    ctrlpt_coords: Vec<Vec<[f64; 3]>>,
) {
    let uknots = KnotVec::from_iter((0..uknot_len).map(|i| i as f64 - udegree as f64));
    let vknots = KnotVec::from_iter((0..vknot_len).map(|i| i as f64 - vdegree as f64));
    let points = coords_to_points(
        uknots.len() - udegree - 1,
        vknots.len() - vdegree - 1,
        ctrlpt_coords,
    );
    let (step_cps_indices, step_cps) = step_bsp_surface_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = UNIFORM_SURFACE('', {udegree}, {vdegree}, {step_cps_indices}, .UNSPECIFIED., .U., .U., .U.);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<UniformSurfaceHolder>(&step_str);
    let res: BSplineSurface<Point3> = (&bsp_step).try_into().unwrap();
    let ans = BSplineSurface::new((uknots, vknots), points);
    compare_bsp_surfaces(&res, &ans);
}

proptest! {
    #[test]
    fn uniform_surface(
        degrees in array::uniform2(1usize..6),
        knot_lens in array::uniform2(7usize..30),
        ctrlpt_coords in collection::vec(collection::vec(array::uniform3(-100.0f64..100.0f64), 30), 30),
    ) {
        exec_uniform_surface(degrees, knot_lens, ctrlpt_coords)
    }
}

fn exec_nurbs_surface_b_spline_surface_with_knots(
    uknot_len: usize,
    uknot_mults: Vec<usize>,
    uknot_incrs: Vec<f64>,
    udegree: usize,
    vknot_len: usize,
    vknot_mults: Vec<usize>,
    vknot_incrs: Vec<f64>,
    vdegree: usize,
    ctrlpt_coords: Vec<Vec<[f64; 3]>>,
    mut weights: Vec<Vec<f64>>,
) {
    let mut s = 0.0;
    let uvec = uknot_mults
        .iter()
        .take(uknot_len)
        .zip(uknot_incrs)
        .flat_map(|(m, x)| {
            s += x;
            std::iter::repeat(s).take(*m)
        })
        .collect::<Vec<f64>>();
    let uknots = KnotVec::from(uvec);
    let mut s = 0.0;
    let vvec = vknot_mults
        .iter()
        .take(vknot_len)
        .zip(vknot_incrs)
        .flat_map(|(m, x)| {
            s += x;
            std::iter::repeat(s).take(*m)
        })
        .collect::<Vec<f64>>();
    let vknots = KnotVec::from(vvec);
    let cps = coords_to_points(
        uknots.len() - udegree - 1,
        vknots.len() - vdegree - 1,
        ctrlpt_coords,
    );
    weights.truncate(cps.len());
    weights
        .iter_mut()
        .zip(&cps)
        .for_each(|(vec, vec0)| vec.truncate(vec0.len()));
    let bsp = BSplineSurface::new((uknots, vknots), cps);
    let ans = NurbsSurface::<Vector4>::try_from_bspline_and_weights(bsp, weights).unwrap();
    let step_str = format!("DATA;{}ENDSEC;", StepDisplay::new(&ans, 1));
    let bsp_step = step_to_entity::<RationalBSplineSurfaceHolder>(&step_str);
    let res: NurbsSurface<Vector4> = (&bsp_step).try_into().unwrap();
    compare_nurbs_surfaces(&res, &ans);
}

proptest! {
    #[test]
    fn nurbs_surface_b_spline_surface_with_knots(
        uknot_len in 7usize..10,
        uknot_mults in collection::vec(1usize..4usize, 10),
        uknot_incrs in collection::vec(1.0e-3f64..100.0f64, 10),
        udegree in 2usize..6,
        vknot_len in 7usize..10,
        vknot_mults in collection::vec(1usize..4usize, 10),
        vknot_incrs in collection::vec(1.0e-3f64..100.0f64, 10),
        vdegree in 2usize..6,
        ctrlpt_coords in collection::vec(collection::vec(array::uniform3(-100.0f64..100.0f64), 40), 40),
        weights in collection::vec(collection::vec(0.01f64..100.0f64, 40), 40),
    ) {
        exec_nurbs_surface_b_spline_surface_with_knots(
            uknot_len,
            uknot_mults,
            uknot_incrs,
            udegree,
            vknot_len,
            vknot_mults,
            vknot_incrs,
            vdegree,
            ctrlpt_coords,
            weights,
        )
    }
}

fn exec_nurbs_surface_bezier_surface(
    [udegree, vdegree]: [usize; 2],
    ctrlpt_coords: Vec<Vec<[f64; 3]>>,
    mut weights: Vec<Vec<f64>>,
) {
    let points = coords_to_points(udegree + 1, vdegree + 1, ctrlpt_coords);
    weights.truncate(points.len());
    weights
        .iter_mut()
        .zip(&points)
        .for_each(|(vec, vec0)| vec.truncate(vec0.len()));
    let weights_diaplays = weights
        .iter()
        .map(|vec| SliceDisplay(vec))
        .collect::<Vec<_>>();
    let weights_display = SliceDisplay(&weights_diaplays);
    let (step_cps_indices, step_cps) = step_bsp_surface_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = (
    BEZIER_SURFACE()
    BOUNDED_SURFACE()
    B_SPLINE_SURFACE({udegree}, {vdegree}, {step_cps_indices}, .UNSPECIFIED., .U., .U., .U.)
    GEOMETRIC_REPRESENTATION_ITEM()
    RATIONAL_B_SPLINE_SURFACE({weights_display})
    REPRESENTATION_ITEM('')
    SURFACE()
);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<RationalBSplineSurfaceHolder>(&step_str);
    let res: NurbsSurface<Vector4> = (&bsp_step).try_into().unwrap();
    let bsp = BSplineSurface::new(
        (KnotVec::bezier_knot(udegree), KnotVec::bezier_knot(vdegree)),
        points,
    );
    let ans = NurbsSurface::<Vector4>::try_from_bspline_and_weights(bsp, weights).unwrap();
    compare_nurbs_surfaces(&res, &ans);
}

proptest! {
    #[test]
    fn nurbs_surface_bezier_surface(
        degrees in array::uniform2(1usize..6),
        ctrlpt_coords in collection::vec(collection::vec(array::uniform3(-100.0f64..100.0f64), 6), 6),
        weights in collection::vec(collection::vec(0.01f64..100.0f64, 6), 6),
    ) {
        exec_nurbs_surface_bezier_surface(degrees, ctrlpt_coords, weights)
    }
}

fn exec_nurbs_surface_quasi_uniform_surface(
    [udegree, vdegree]: [usize; 2],
    [udivision, vdivision]: [usize; 2],
    ctrlpt_coords: Vec<Vec<[f64; 3]>>,
    mut weights: Vec<Vec<f64>>,
) {
    let mut uknots = KnotVec::uniform_knot(udegree, udivision);
    uknots.transform(udivision as f64, 0.0);
    let mut vknots = KnotVec::uniform_knot(vdegree, vdivision);
    vknots.transform(vdivision as f64, 0.0);
    let points = coords_to_points(
        uknots.len() - udegree - 1,
        vknots.len() - vdegree - 1,
        ctrlpt_coords,
    );
    weights.truncate(points.len());
    weights
        .iter_mut()
        .zip(&points)
        .for_each(|(vec, vec0)| vec.truncate(vec0.len()));
    let weights_diaplays = weights
        .iter()
        .map(|vec| SliceDisplay(vec))
        .collect::<Vec<_>>();
    let weights_display = SliceDisplay(&weights_diaplays);
    let (step_cps_indices, step_cps) = step_bsp_surface_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = (
    BOUNDED_SURFACE()
    B_SPLINE_SURFACE({udegree}, {vdegree}, {step_cps_indices}, .UNSPECIFIED., .U., .U., .U.)
    GEOMETRIC_REPRESENTATION_ITEM()
    QUASI_UNIFORM_SURFACE()
    RATIONAL_B_SPLINE_SURFACE({weights_display})
    REPRESENTATION_ITEM('')
    SURFACE()
);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<RationalBSplineSurfaceHolder>(&step_str);
    let res: NurbsSurface<Vector4> = (&bsp_step).try_into().unwrap();
    let bsp = BSplineSurface::new((uknots, vknots), points);
    let ans = NurbsSurface::<Vector4>::try_from_bspline_and_weights(bsp, weights).unwrap();
    compare_nurbs_surfaces(&res, &ans);
}

proptest! {
    #[test]
    fn nurbs_surface_quasi_uniform_surface(
        degrees in array::uniform2(1usize..6),
        divisions in array::uniform2(2usize..5),
        ctrlpt_coords in collection::vec(collection::vec(array::uniform3(-100.0f64..100.0f64), 30), 30),
        weights in collection::vec(collection::vec(0.01f64..100.0f64, 30), 30),
    ) {
        exec_nurbs_surface_quasi_uniform_surface(degrees, divisions, ctrlpt_coords, weights)
    }
}

fn exec_nurbs_surface_uniform_surface(
    [udegree, vdegree]: [usize; 2],
    [uknot_len, vknot_len]: [usize; 2],
    ctrlpt_coords: Vec<Vec<[f64; 3]>>,
    mut weights: Vec<Vec<f64>>,
) {
    let uknots = KnotVec::from_iter((0..uknot_len).map(|i| i as f64 - udegree as f64));
    let vknots = KnotVec::from_iter((0..vknot_len).map(|i| i as f64 - vdegree as f64));
    let points = coords_to_points(
        uknots.len() - udegree - 1,
        vknots.len() - vdegree - 1,
        ctrlpt_coords,
    );
    println!("{} {}", uknots.len(), points.len());
    weights.truncate(points.len());
    weights
        .iter_mut()
        .zip(&points)
        .for_each(|(vec, vec0)| vec.truncate(vec0.len()));
    let weights_diaplays = weights
        .iter()
        .map(|vec| SliceDisplay(vec))
        .collect::<Vec<_>>();
    let weights_display = SliceDisplay(&weights_diaplays);
    let (step_cps_indices, step_cps) = step_bsp_surface_ctrls(&points);
    let step_str = format!(
        "DATA;
#1 = (
    BOUNDED_SURFACE()
    B_SPLINE_SURFACE({udegree}, {vdegree}, {step_cps_indices}, .UNSPECIFIED., .U., .U., .U.)
    GEOMETRIC_REPRESENTATION_ITEM()
    RATIONAL_B_SPLINE_SURFACE({weights_display})
    REPRESENTATION_ITEM('')
    SURFACE()
    UNIFORM_SURFACE()
);
{step_cps}ENDSEC;"
    );
    let bsp_step = step_to_entity::<RationalBSplineSurfaceHolder>(&step_str);
    let res: NurbsSurface<Vector4> = (&bsp_step).try_into().unwrap();
    let bsp = BSplineSurface::new((uknots, vknots), points);
    let ans = NurbsSurface::<Vector4>::try_from_bspline_and_weights(bsp, weights).unwrap();
    compare_nurbs_surfaces(&res, &ans);
}

proptest! {
    #[test]
    fn nurbs_surface_uniform_surface(
        degrees in array::uniform2(1usize..6),
        knot_lens in array::uniform2(7usize..30),
        ctrlpt_coords in collection::vec(collection::vec(array::uniform3(-100.0f64..100.0f64), 30), 30),
        weights in collection::vec(collection::vec(0.01f64..100.0f64, 30), 30),
    ) {
        exec_nurbs_surface_uniform_surface(degrees, knot_lens, ctrlpt_coords, weights)
    }
}

fn exec_surface_of_linear_extrusion(
    point0_coord: [f64; 3],
    point1_coord: [f64; 3],
    axis_elem: [f64; 3],
) {
    let line = Line(Point3::from(point0_coord), Point3::from(point1_coord));
    if line.0.near(&line.1) {
        return;
    }
    let axis = Vector3::from(axis_elem);
    let step_str = format!(
        "DATA;#1 = SURFACE_OF_LINEAR_EXTRUSION('', #4, #2);{}{}ENDSEC;",
        StepDisplay::new(axis, 2),
        StepDisplay::new(&line, 4),
    );
    let step_surface = step_to_entity::<SurfaceOfLinearExtrusionHolder>(&step_str);
    let surface: StepExtrudedCurve = (&step_surface).try_into().unwrap();
    (0..=100)
        .flat_map(move |i| (0..=100).map(move |j| (i, j)))
        .for_each(|(i, j)| {
            let (u, v) = (i as f64 / 10.0, j as f64 / 10.0);
            assert_near!(surface.subs(u, v), line.subs(u) + axis * v);
        });
}

proptest! {
    #[test]
    fn surface_of_linear_extrusion(
        point0_coord in array::uniform3(-100.0f64..100.0f64),
        point1_coord in array::uniform3(-100.0f64..100.0f64),
        axis_elem in array::uniform3(-100.0f64..100.0f64),
    ) {
        exec_surface_of_linear_extrusion(point0_coord, point1_coord, axis_elem)
    }
}

fn exec_surface_of_revolution(
    point0_coord: [f64; 3],
    point1_coord: [f64; 3],
    org_coord: [f64; 3],
    axis_array: [f64; 2],
) {
    let line = Line(Point3::from(point0_coord), Point3::from(point1_coord));
    if line.0.near(&line.1) {
        return;
    }
    let origin = Point3::from(org_coord);
    let dir = dir_from_array(axis_array);
    let step_str = format!(
        "DATA;
#1 = SURFACE_OF_REVOLUTION('', #5, #2);
#2 = AXIS1_PLACEMENT('', #3, #4);
{}{}{}ENDSEC;",
        StepDisplay::new(origin, 3),
        StepDisplay::new(VectorAsDirection(dir), 4),
        StepDisplay::new(&line, 5),
    );
    let step_surface = step_to_entity::<SurfaceOfRevolutionHolder>(&step_str);
    let surface: StepRevolutedCurve = (&step_surface).try_into().unwrap();
    (0..=100)
        .flat_map(move |i| (0..=100).map(move |j| (i, j)))
        .for_each(|(i, j)| {
            let (u, v) = (i as f64 / 10.0, j as f64 / 10.0);
            let lc = line.subs(v) - origin;
            let ans = origin
                + lc * f64::cos(u)
                + dir * lc.dot(dir) * (1.0 - f64::cos(u))
                + dir.cross(lc) * f64::sin(u);
            assert_near!(surface.subs(u, v), ans);
        });
}

proptest! {
    #[test]
    fn surface_of_revolution(
        point0_coord in array::uniform3(-100f64..100.0),
        point1_coord in array::uniform3(-100f64..100.0),
        org_coord in array::uniform3(-100f64..100.0),
        axis_array in array::uniform2(0.0f64..1.0),
    ) {
        exec_surface_of_revolution(point0_coord, point1_coord, org_coord, axis_array)
    }
}
