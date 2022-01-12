use super::config_control_design as ap203;
use ruststep::tables::*;
use serde::Deserialize;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::str::FromStr;
use truck_stepio::alias::*;

const STEP_CODE: &str = r#"
DATA;
  #1 = LABEL('');
  #2 = REPRESENTATION_ITEM(#1);
  #3 = GEOMETRIC_REPRESENTATION_ITEM(#2);
  #4 = POINT(#3);
  #5 = CARTESIAN_POINT(#4, (LENGTH_MEASURE((1.0)), LENGTH_MEASURE((2.0))));
  #6 = CARTESIAN_POINT(#4, (LENGTH_MEASURE((1.0)), LENGTH_MEASURE((2.0)), LENGTH_MEASURE((3.0))));
  #7 = DIRECTION(#3, (0.0, 1.0));
  #8 = DIRECTION(#3, (0.0, 0.0, 1.0));
  #9 = VECTOR(#3, #7, LENGTH_MEASURE((6.0)));
  #10 = VECTOR(#3, #8, LENGTH_MEASURE((6.0)));
  #11 = PLACEMENT(#3, #5);
  #12 = PLACEMENT(#3, #6);
  #13 = AXIS_2_PLACEMENT_2D(#11, $);
  #14 = AXIS_2_PLACEMENT_3D(#12, $, $);
  #15 = SURFACE(#3);
  #16 = ELEMENTARY_SURFACE(#15, #14);
  #17 = TOROIDAL_SURFACE(#16, POSITIVE_LENGTH_MEASURE((LENGTH_MEASURE((5.0)))), POSITIVE_LENGTH_MEASURE((LENGTH_MEASURE((2.0)))));
ENDSEC;
"#;

fn test<'a, THolder, U>(table: &ap203::Tables, idx: u64, answer: U)
where
    THolder: Holder<Table = ap203::Tables> + Deserialize<'a> + Debug + 'a,
    U: From<THolder::Owned> + Debug + PartialEq,
    ap203::Tables: EntityTable<THolder>, {
    let a = EntityTable::<THolder>::get_owned(&table, idx).unwrap();
    assert_eq!(U::from(a), answer);
}

fn try_test<'a, THolder, U>(table: &ap203::Tables, idx: u64, answer: U)
where
    THolder: Holder<Table = ap203::Tables> + Deserialize<'a> + Debug + 'a,
    U: TryFrom<THolder::Owned, Error = ExpressParseError> + Debug + PartialEq,
    ap203::Tables: EntityTable<THolder>, {
    let a = EntityTable::<THolder>::get_owned(&table, idx).unwrap();
    assert_eq!(U::try_from(a).unwrap(), answer);
}

#[test]
fn primitives() {
    let table = ap203::Tables::from_str(STEP_CODE).unwrap();
    test::<ap203::CartesianPointHolder, Point2>(&table, 5, Point2::new(1.0, 2.0));
    test::<ap203::CartesianPointHolder, Point3>(&table, 6, Point3::new(1.0, 2.0, 3.0));
    try_test::<ap203::PointAnyHolder, Point2>(&table, 5, Point2::new(1.0, 2.0));
    try_test::<ap203::PointAnyHolder, Point3>(&table, 6, Point3::new(1.0, 2.0, 3.0));
    test::<ap203::DirectionHolder, Vector2>(&table, 7, Vector2::new(0.0, 1.0));
    test::<ap203::DirectionHolder, Vector3>(&table, 8, Vector3::new(0.0, 0.0, 1.0));
    test::<ap203::VectorHolder, Vector2>(&table, 9, Vector2::new(0.0, 6.0));
    test::<ap203::VectorHolder, Vector3>(&table, 10, Vector3::new(0.0, 0.0, 6.0));
    test::<ap203::PlacementHolder, Point2>(&table, 11, Point2::new(1.0, 2.0));
    test::<ap203::PlacementHolder, Point3>(&table, 12, Point3::new(1.0, 2.0, 3.0));
    test::<ap203::Axis2Placement2DHolder, Matrix3>(
        &table, 
        13,
        Matrix3::from_translation(Vector2::new(1.0, 2.0)),
    );
}
