use super::build::config_control_design as ap04x;
use ruststep::tables::*;
use serde::Deserialize;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::str::FromStr;
use truck_stepio::alias::*;

const REPRESENTATION_ITEM: &str = r#"
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

fn test<'a, THolder, U>(idx: u64, answer: U)
where
    THolder: Holder<Table = ap04x::Tables> + Deserialize<'a> + Debug + 'a,
    U: From<THolder::Owned> + Debug + PartialEq,
    ap04x::Tables: EntityTable<THolder>, {
    let table = ap04x::Tables::from_str(REPRESENTATION_ITEM).unwrap();
    let a = EntityTable::<THolder>::get_owned(&table, idx).unwrap();
    assert_eq!(U::from(a), answer);
}

fn try_test<'a, THolder, U>(idx: u64, answer: U)
where
    THolder: Holder<Table = ap04x::Tables> + Deserialize<'a> + Debug + 'a,
    U: TryFrom<THolder::Owned, Error = ExpressParseError> + Debug + PartialEq,
    ap04x::Tables: EntityTable<THolder>, {
    let table = ap04x::Tables::from_str(REPRESENTATION_ITEM).unwrap();
    let a = EntityTable::<THolder>::get_owned(&table, idx).unwrap();
    assert_eq!(U::try_from(a).unwrap(), answer);
}

#[test]
fn primitives() {
    test::<ap04x::CartesianPointHolder, Point2>(5, Point2::new(1.0, 2.0));
    test::<ap04x::CartesianPointHolder, Point3>(6, Point3::new(1.0, 2.0, 3.0));
    try_test::<ap04x::PointAnyHolder, Point2>(5, Point2::new(1.0, 2.0));
    try_test::<ap04x::PointAnyHolder, Point3>(6, Point3::new(1.0, 2.0, 3.0));
    test::<ap04x::DirectionHolder, Vector2>(7, Vector2::new(0.0, 1.0));
    test::<ap04x::DirectionHolder, Vector3>(8, Vector3::new(0.0, 0.0, 1.0));
    test::<ap04x::VectorHolder, Vector2>(9, Vector2::new(0.0, 6.0));
    test::<ap04x::VectorHolder, Vector3>(10, Vector3::new(0.0, 0.0, 6.0));
    test::<ap04x::PlacementHolder, Point2>(11, Point2::new(1.0, 2.0));
    test::<ap04x::PlacementHolder, Point3>(12, Point3::new(1.0, 2.0, 3.0));
    test::<ap04x::Axis2Placement2DHolder, Matrix3>(
        13,
        Matrix3::from_translation(Vector2::new(1.0, 2.0)),
    );
}

#[test]
fn mesh_torus() {
    use std::f64::consts::PI;
    use truck_polymesh::{obj, StructuredMesh};
    let table = ap04x::Tables::from_str(REPRESENTATION_ITEM).unwrap();
    let step_torus = EntityTable::<ap04x::ToroidalSurfaceHolder>::get_owned(&table, 17).unwrap();
    let truck_torus = ToroidalSurface::from(&step_torus).inverse();
    let mesh = StructuredMesh::from_surface(&truck_torus, ((0.0, 2.0 * PI), (0.0, 2.0 * PI)), 0.05)
        .destruct();
    obj::write(&mesh, std::fs::File::create("torus-from-step.obj").unwrap()).unwrap();
}
