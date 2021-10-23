use ruststep::*;
use truck_base::cgmath64::*;

include!(concat!(
	env!("OUT_DIR"),
	"/10303-201-aim-long.rs",
));
parse_primitives!(explicit_draughting);

mod parse_primitives;
mod impl_curve;
