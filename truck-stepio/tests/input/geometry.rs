use proptest::*;
use ruststep::{ast::DataSection, tables::*};
use std::str::FromStr;
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
}
