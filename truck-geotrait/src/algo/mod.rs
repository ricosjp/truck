use crate::traits::*;
use truck_base::{cgmath64::*, tolerance::*};

#[macro_export]
#[doc(hidden)]
macro_rules! nonpositive_tolerance {
	($tol: expr) => {
		assert!($tol < TOLERANCE, "tolerance must be more than {:e}", TOLERANCE);
	};
}

/// curve algorithms
pub mod curve;
/// surface algorithms
pub mod surface;
