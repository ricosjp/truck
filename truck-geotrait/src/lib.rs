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

/// Abstract traits: `Curve` and `Surface`.
pub mod traits;
pub use traits::*;
/// Algorithms for curves and surfaces.
pub mod algo;
