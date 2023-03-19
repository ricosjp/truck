use ruststep::ast::DataSection;
use std::str::FromStr;
use truck_stepio::r#in::*;

#[test]
fn line() {
	let step = DataSection::from_str("
DATA;
#1 = LINE(#2, #3);
#2 = 
ENDSEC;
	").unwrap();
}
