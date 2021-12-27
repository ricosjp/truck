use super::build::*;
use nom::Finish;
use ruststep::{ast::*, parser::exchange, tables::*};
use serde::Deserialize;
use std::str::FromStr;
use std::fmt::Debug;
use truck_geometry::*;

const REPRESENTATION_ITEM: &str = r#"
DATA;
  #1 = REPRESENTATION_ITEM(1);
  #2 = GEOMETRIC_REPRESENTATION_ITEM(#1);
ENDSEC;
"#;

#[test]
fn primitives() {
	test::<ap04x::CartesianPointHolder, Point3>(
		"CARTESIAN_POINT(#2, (1.0, 2.0, 3.0))",
		Point3::new(1.0, 2.0, 3.0),
	);

	fn test<
		'a,
		THolder: Holder<Table = ap04x::Tables> + Deserialize<'a> + Debug + 'a,
		U: From<THolder::Owned> + Debug + PartialEq,
	>(
		input: &str,
		answer: U,
	) {
		let table = ap04x::Tables::from_str(REPRESENTATION_ITEM).unwrap();

		let (residual, p): (_, Record) = exchange::simple_record(input).finish().unwrap();
		dbg!(&p);
		assert_eq!(residual, "");

		let a: THolder = Deserialize::deserialize(&p).unwrap();
		dbg!(&a);
		let a = a.into_owned(&table).unwrap();
		assert_eq!(U::from(a), answer);
	}
}
