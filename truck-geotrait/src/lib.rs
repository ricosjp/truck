//! Defines geometric traits: `ParametricCurve`, `ParametricSurface`, and so on.
//! Implements some algorithms for traits.

#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]
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
    ($tol: expr, $minimum: expr) => {
        assert!(
            $tol >= $minimum,
            "tolerance must be no less than {:e}",
            $minimum
        );
    };
    ($tol: expr) => {
        nonpositive_tolerance!($tol, TOLERANCE)
    };
}

/// Abstract traits: `Curve` and `Surface`.
pub mod traits;
pub use traits::*;
/// Algorithms for curves and surfaces.
pub mod algo;
#[cfg(feature = "derive")]
pub use truck_geoderive::*;
