use proptest::*;
use ruststep::{ast::DataSection, tables::*};
use std::{f64::consts::PI, str::FromStr};
use truck_geometry::prelude as truck;
use truck_stepio::{
    out::{IndexSliceDisplay, StepDisplay, VectorAsDirection},
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

fn exec_direction(elem: [f64; 3]) {
    let vec = Vector2::new(elem[0], elem[1]).normalize();
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
    let vec = Vector3::from(elem).normalize();
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

proptest! {
    #[test]
    fn direction(elem in array::uniform3(-100.0f64..100.0f64)) {
        exec_direction(elem)
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

fn exec_axis1_placement(org_coord: [f64; 3], dir_elem: [f64; 3]) {
    let p = Point2::new(org_coord[0], org_coord[1]);
    let v = Vector2::new(dir_elem[0], dir_elem[1]);
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

    let p = Point3::from(org_coord);
    let v = Vector3::from(dir_elem);
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

proptest! {
    #[test]
    fn axis1_placement(
        org_coord in array::uniform3(-100.0f64..100.0f64),
        dir_elem in array::uniform3(-100.0f64..100.0f64),
    ) {
        exec_axis1_placement(org_coord, dir_elem)
    }
}

fn exec_axis2_placement2d(org_coord: [f64; 2], dir_elem: [f64; 2]) {
    let origin = Point2::from(org_coord);
    let v = Vector2::from(dir_elem);
    let dir = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
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
        dir_elem in array::uniform2(-100.0f64..100.0f64),
    ) {
        exec_axis2_placement2d(org_coord, dir_elem)
    }
}

fn exec_axis2_placement3d(org_coord: [f64; 3], dir_elem: [f64; 3], ref_dir_elem: [f64; 3]) {
    let p = Point3::from(org_coord);
    let v = Vector3::from(dir_elem);
    let z = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let ref_dir = Vector3::from(ref_dir_elem);
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
        dir_elem in array::uniform3(-100.0f64..100.0f64),
        ref_dir_elem in array::uniform3(-100.0f64..100.0f64),
    ) {
        exec_axis2_placement3d(org_coord, dir_elem, ref_dir_elem)
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
    knot_mults: Vec<usize>,
    knot_incrs: Vec<f64>,
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
    let degree = knot_mults[0] + 1;
    if vec.len() <= degree + 1 {
        return;
    }
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
        knot_len in 3usize..20,
        knot_mults in collection::vec(1usize..4usize, 20),
        knot_incrs in collection::vec(1.0e-3f64..100.0f64, 20),
        ctrlpt_coords in collection::vec(array::uniform3(-100.0f64..100.0f64), 80),
    ) {
        exec_b_spline_curve_with_knots(knot_len, knot_mults, knot_incrs, ctrlpt_coords)
    }
}

fn exec_bezier_curve(degree: usize, ctrlpt_coords: Vec<[f64; 3]>) {
    let points = ctrlpt_coords
        .into_iter()
        .take(degree + 1)
        .map(Point3::from)
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

fn exec_circle(org_coord: [f64; 3], dir_elem: [f64; 3], ref_dir_elem: [f64; 3], radius: f64) {
    let origin = Point3::from(org_coord);
    let v = Vector3::from(dir_elem);
    let z = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let ref_dir = Vector3::from(ref_dir_elem);
    let v = z.cross(ref_dir);
    let y = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let x = y.cross(z).normalize();
    let step_str = format!(
        "DATA;
#1 = CIRCLE('', #2, {radius});
#2 = AXIS2_PLACEMENT_3D('', #3, #4, #5);
{}{}{}ENDSEC;",
        StepDisplay::new(origin, 3),
        StepDisplay::new(VectorAsDirection(z), 4),
        StepDisplay::new(VectorAsDirection(ref_dir.normalize()), 5),
    );
    let step_circle = step_to_entity::<CircleHolder>(&step_str);
    let ellipse: Ellipse<Point3, Matrix4> = (&step_circle).try_into().unwrap();
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
        dir_elem in array::uniform3(-100.0f64..100.0f64),
        ref_dir_elem in array::uniform3(-100.0f64..100.0f64),
        radius in 1.0e-2f64..100.0,
    ) {
        exec_circle(org_coord, dir_elem, ref_dir_elem, radius)
    }
}

fn exec_plane(org_coord: [f64; 3], dir_elem: [f64; 3], ref_dir_elem: [f64; 3],) {
    let origin = Point3::from(org_coord);
    let v = Vector3::from(dir_elem);
    let z = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let ref_dir = Vector3::from(ref_dir_elem);
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
        dir_elem in array::uniform3(-100.0f64..100.0f64),
        ref_dir_elem in array::uniform3(-100.0f64..100.0f64),
    ) {
        exec_plane(org_coord, dir_elem, ref_dir_elem)
    }
}

fn exec_spherical_surface(org_coord: [f64; 3], dir_elem: [f64; 3], ref_dir_elem: [f64; 3], radius: f64) {
    let p = Point3::from(org_coord);
    let v = Vector3::from(dir_elem);
    let z = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let ref_dir = Vector3::from(ref_dir_elem);
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
        dir_elem in array::uniform3(-100.0f64..100.0f64),
        ref_dir_elem in array::uniform3(-100.0f64..100.0f64),
        radius in 1.0e-2f64..100.0f64
    ) {
        exec_spherical_surface(org_coord, dir_elem, ref_dir_elem, radius)
    }
}

fn exec_cylindrical_surface(org_coord: [f64; 3], dir_elem: [f64; 3], ref_dir_elem: [f64; 3], radius: f64) {
    let p = Point3::from(org_coord);
    let v = Vector3::from(dir_elem);
    let z = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let ref_dir = Vector3::from(ref_dir_elem);
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
        dir_elem in array::uniform3(-100.0f64..100.0f64),
        ref_dir_elem in array::uniform3(-100.0f64..100.0f64),
        radius in 1.0e-2f64..100.0f64
    ) {
        exec_cylindrical_surface(org_coord, dir_elem, ref_dir_elem, radius)
    }
}

fn exec_toroidal_surface(org_coord: [f64; 3], dir_elem: [f64; 3], ref_dir_elem: [f64; 3], radii: [f64; 2]) {
    let p = Point3::from(org_coord);
    let v = Vector3::from(dir_elem);
    let z = match v.so_small() {
        true => return,
        false => v.normalize(),
    };
    let ref_dir = Vector3::from(ref_dir_elem);
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
        dir_elem in array::uniform3(-100.0f64..100.0f64),
        ref_dir_elem in array::uniform3(-100.0f64..100.0f64),
        radii in array::uniform2(1.0e-2f64..100.0f64),
    ) {
        exec_toroidal_surface(org_coord, dir_elem, ref_dir_elem, radii)
    }
}
