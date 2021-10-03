//! Defines geometric traits: `ParametricCurve`, `ParametricSurface`, and so on.
//! Implements some algorithms for traits.

#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

#[macro_export]
#[doc(hidden)]
macro_rules! nonpositive_tolerance {
	($tol: expr) => {
		assert!($tol > TOLERANCE, "tolerance must be more than {:e}", TOLERANCE);
	};
}

/// Abstract traits: `Curve` and `Surface`.
pub mod traits;
pub use traits::*;
/// Algorithms for curves and surfaces.
pub mod algo;
