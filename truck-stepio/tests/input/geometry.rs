use proptest::*;
use ruststep::{ast::DataSection, tables::*};
use std::{f64::consts::PI, str::FromStr};
use truck_geometry::prelude as truck;
use truck_stepio::{
    out::{StepDisplay, VectorAsDirection},
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

fn exec_cartesian_point(arg: [f64; 3]) {
    let pt = Point2::new(arg[0], arg[1]);
    exec_test_near::<CartesianPointHolder, Point2>(
        pt,
        &format!("DATA;{}ENDSEC;", truck_stepio::out::StepDisplay::new(pt, 1)),
    );
    let pt = Point3::from(arg);
    exec_test_near::<CartesianPointHolder, Point3>(
        pt,
        &format!("DATA;{}ENDSEC;", truck_stepio::out::StepDisplay::new(pt, 1)),
    );
}

fn exec_direction(arg: [f64; 3]) {
    let vec = Vector2::new(arg[0], arg[1]).normalize();
    if vec.so_small() {
        return;
    }
    exec_test_near::<DirectionHolder, Vector2>(
        vec,
        &format!(
            "DATA;#1 = DIRECTION('', ({}, {}));ENDSEC;",
            float_to_str(vec[0]),
            float_to_str(vec[1])
        ),
    );
    let vec = Vector3::from(arg).normalize();
    if vec.so_small() {
        return;
    }
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

fn exec_vector(arg: [f64; 3]) {
    let vec = Vector2::new(arg[0], arg[1]);
    exec_test_near::<VectorHolder, Vector2>(
        vec,
        &format!("DATA;{}ENDSEC;", StepDisplay::new(vec, 1)),
    );
    let vec = Vector3::from(arg);
    exec_test_near::<VectorHolder, Vector3>(
        vec,
        &format!("DATA;{}ENDSEC;", StepDisplay::new(vec, 1)),
    );
}

fn exec_placement(arg: [f64; 3]) {
    let p = Point2::new(arg[0], arg[1]);
    exec_test_near::<PlacementHolder, Point2>(
        p,
        &format!(
            "DATA;#1 = PLACEMENT('', #2);{}ENDSEC;",
            StepDisplay::new(p, 2)
        ),
    );
    let p = Point3::from(arg);
    exec_test_near::<PlacementHolder, Point3>(
        p,
        &format!(
            "DATA;#1 = PLACEMENT('', #2);{}ENDSEC;",
            StepDisplay::new(p, 2)
        ),
    );
}

fn exec_axis1_placement(arg: [f64; 6]) {
    let p = Point2::new(arg[0], arg[1]);
    let v = Vector2::new(arg[3], arg[4]);
    if v.so_small() {
        return;
    }
    let dir = v.normalize();
    let step_str = format!(
        "DATA;#1 = AXIS1_PLACEMENT('', #2, #3);{}{}ENDSEC;",
        StepDisplay::new(p, 2),
        StepDisplay::new(VectorAsDirection(dir), 3)
    );
    let placement = step_to_entity::<Axis1PlacementHolder>(&step_str);
    assert_near!(p, Point2::from(&placement.location));
    assert_near!(dir, placement.direction().truncate());

    let p = Point3::new(arg[0], arg[1], arg[2]);
    let v = Vector3::new(arg[3], arg[4], arg[5]);
    if v.so_small() {
        return;
    }
    let dir = v.normalize();
    let step_str = format!(
        "DATA;#1 = AXIS1_PLACEMENT('', #2, #3);{}{}ENDSEC;",
        StepDisplay::new(p, 2),
        StepDisplay::new(VectorAsDirection(dir), 3)
    );
    let placement = step_to_entity::<Axis1PlacementHolder>(&step_str);
    assert_near!(p, Point3::from(&placement.location));
    assert_near!(dir, placement.direction());
}

fn exec_axis2_placement2d(arg: [f64; 4]) {
    let p = Point2::new(arg[0], arg[1]);
    let v = Vector2::new(arg[2], arg[3]);
    let dir = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let step_str = format!(
        "DATA;#1 = AXIS2_PLACEMENT_2D('', #2, #3);{}{}ENDSEC;",
        StepDisplay::new(p, 2),
        StepDisplay::new(VectorAsDirection(dir), 3),
    );
    let placement = step_to_entity::<Axis2Placement2dHolder>(&step_str);
    let res: Matrix3 = (&placement).into();
    let n = Vector2::new(-dir.y, dir.x);
    let ans = Matrix3::from_cols(dir.extend(0.0), n.extend(0.0), p.to_vec().extend(1.0));
    assert_near!(res, ans);
}

fn exec_axis2_placement3d(arg: [f64; 9]) {
    let p = Point3::new(arg[0], arg[1], arg[2]);
    let v = Vector3::new(arg[3], arg[4], arg[5]);
    let z = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let ref_dir = Vector3::new(arg[6], arg[7], arg[8]);
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

fn exec_line(arg: [f64; 6]) {
    let p = Point3::new(arg[0], arg[1], arg[2]);
    let v = Vector3::new(arg[3], arg[4], arg[5]);
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

fn exec_polyline(arg: [f64; 12]) {
    let p = arg
        .chunks(3)
        .map(|x| Point3::new(x[0], x[1], x[2]))
        .collect::<Vec<_>>();
    let step_str = format!(
        "DATA;#1 = POLYLINE('', (#2, #3, #4, #5));{}{}{}{}ENDSEC;",
        StepDisplay::new(p[0], 2),
        StepDisplay::new(p[1], 3),
        StepDisplay::new(p[2], 4),
        StepDisplay::new(p[3], 5),
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

fn exec_b_spline_curve_with_knots(arg0: [usize; 5], arg1: [f64; 5], arg2: [f64; 32]) {
    let mut s = 0.0;
    let vec = arg0
        .into_iter()
        .zip(arg1)
        .flat_map(|(m, x)| {
            s += x;
            std::iter::repeat(s).take(m)
        })
        .collect::<Vec<f64>>();
    let degree = arg0[0] + 1;
    if vec.len() <= degree + 1 {
        return;
    }
    let knots = KnotVec::from(vec);
    let cps = arg2
        .windows(3)
        .take(knots.len() - degree - 1)
        .map(|x| Point3::new(x[0], x[1], x[2]))
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

fn exec_bezier_curve(arg0: usize, arg1: [f64; 18]) {
    let degree = arg0;
    let points = arg1
        .chunks(3)
        .take(degree + 1)
        .map(|p| Point3::new(p[0], p[1], p[2]))
        .collect::<Vec<_>>();
    let step_cps_indices = (0..=degree)
        .map(|i| format!("#{}", i + 2))
        .collect::<Vec<_>>();
    let step_cps_indices = step_cps_indices.join(",");
    let step_cps = (0..=degree).fold(String::new(), |string, i| {
        string + &StepDisplay::new(points[i], i + 2).to_string()
    });
    let step_str = format!(
        "DATA;
#1 = BEZIER_CURVE('', {degree}, ({step_cps_indices}), .UNSPECIFIED., .U., .U.);
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

fn exec_quasi_uniform_curve(arg0: usize, arg1: usize, arg2: [f64; 30]) {
    let degree = arg0;
    let division = arg1;
    let mut knots = KnotVec::uniform_knot(degree, division);
    knots.transform(division as f64, 0.0);
    let points = arg2
        .windows(3)
        .take(knots.len() - degree - 1)
        .map(|x| Point3::new(x[0], x[1], x[2]))
        .collect::<Vec<_>>();
    let step_cps_indices = (0..points.len())
        .map(|i| format!("#{}", i + 2))
        .collect::<Vec<_>>();
    let step_cps_indices = step_cps_indices.join(",");
    let step_cps = (0..points.len()).fold(String::new(), |string, i| {
        string + &StepDisplay::new(points[i], i + 2).to_string()
    });
    let step_str = format!(
        "DATA;
#1 = QUASI_UNIFORM_CURVE('', {degree}, ({step_cps_indices}), .UNSPECIFIED., .U., .U.);
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

fn exec_uniform_curve(arg0: usize, arg1: usize, arg2: [f64; 30]) {
    let degree = arg0;
    let all = arg1;
    let knots = KnotVec::from_iter((0..all).map(|i| i as f64 - degree as f64));
    let points = arg2
        .chunks(3)
        .take(all - degree - 1)
        .map(|x| Point3::new(x[0], x[1], x[2]))
        .collect::<Vec<_>>();
    let step_cps_indices = (0..points.len())
        .map(|i| format!("#{}", i + 2))
        .collect::<Vec<_>>();
    let step_cps_indices = step_cps_indices.join(",");
    let step_cps = (0..points.len()).fold(String::new(), |string, i| {
        string + &StepDisplay::new(points[i], i + 2).to_string()
    });
    let step_str = format!(
        "DATA;
#1 = UNIFORM_CURVE('', {degree}, ({step_cps_indices}), .UNSPECIFIED., .U., .U.);
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

fn exec_circle(arg: [f64; 10]) {
    let p = Point3::new(arg[0], arg[1], arg[2]);
    let v = Vector3::new(arg[3], arg[4], arg[5]);
    let z = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let ref_dir = Vector3::new(arg[6], arg[7], arg[8]);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let radius = arg[9] + 100.0;
    let step_str = format!(
        "DATA;
#1 = CIRCLE('', #2, {radius});
#2 = AXIS2_PLACEMENT_3D('', #3, #4, #5);
{}{}{}ENDSEC;",
        StepDisplay::new(p, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_circle = step_to_entity::<CircleHolder>(&step_str);
    let ellipse: Ellipse<Point3, Matrix4> = (&step_circle).try_into().unwrap();
    let mat = Matrix4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        p.to_vec().extend(1.0),
    );
    (0..10).for_each(|i| {
        let t = 2.0 * PI * i as f64 / 10.0;
        let p = Point3::new(radius * f64::cos(t), radius * f64::sin(t), 0.0);
        assert_near!(ellipse.subs(t), mat.transform_point(p));
    });
}

proptest! {
    #[test]
    fn cartesian_point(arg in array::uniform3(-100.0f64..100.0f64)) {
        exec_cartesian_point(arg)
    }
    #[test]
    fn direction(arg in array::uniform3(-100.0f64..100.0f64)) {
        exec_direction(arg)
    }
    #[test]
    fn vector(arg in array::uniform3(-100.0f64..100.0f64)) {
        exec_vector(arg)
    }
    #[test]
    fn placement(arg in array::uniform3(-100.0f64..100.0f64)) {
        exec_placement(arg)
    }
    #[test]
    fn axis1_placement(arg in array::uniform6(-100.0f64..100.0f64)) {
        exec_axis1_placement(arg)
    }
    #[test]
    fn axis2_placement_2d(arg in array::uniform4(-100.0f64..100.0f64)) {
        exec_axis2_placement2d(arg)
    }
    #[test]
    fn axis2_placement_3d(arg in array::uniform9(-100.0f64..100.0f64)) {
        exec_axis2_placement3d(arg)
    }
    #[test]
    fn line(arg in array::uniform6(-100.0f64..100.0f64)) {
        exec_line(arg)
    }
    #[test]
    fn polyline(arg in array::uniform12(-100.0f64..100.0f64)) {
        exec_polyline(arg)
    }
    #[test]
    fn b_spline_curve_with_knots(
        arg0 in array::uniform5(0usize..3usize),
        arg1 in array::uniform5(1.0e-3f64..100.0f64),
        arg2 in array::uniform32(-100.0f64..100.0f64),
    ) {
        exec_b_spline_curve_with_knots(arg0, arg1, arg2)
    }
    #[test]
    fn bezier_curve(
        arg0 in 1usize..6,
        arg1 in array::uniform18(-100.0f64..100.0f64),
    ) {
        exec_bezier_curve(arg0, arg1)
    }
    #[test]
    fn quasi_uniform_curve(
        arg0 in 1usize..4,
        arg1 in 3usize..5,
        arg2 in array::uniform30(-100.0f64..100.0f64),
    ) {
        exec_quasi_uniform_curve(arg0, arg1, arg2)
    }
    #[test]
    fn uniform_curve(
        arg0 in 1usize..4,
        arg1 in 6usize..10,
        arg2 in array::uniform30(-100.0f64..100.0f64),
    ) {
        exec_uniform_curve(arg0, arg1, arg2)
    }
    #[test]
    fn circle(arg in array::uniform10(-100.0f64..100.0f64)) {
        exec_circle(arg)
    }
}
