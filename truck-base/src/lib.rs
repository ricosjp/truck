//! Basic structs and traits: importing cgmath, curve and surface traits, tolerance

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

/// Defines bounding box
pub mod bounding_box;
/// Redefines vectors, matrices or points with scalar = f64.
pub mod cgmath64;
/// Additional traits for cgmath
pub mod cgmath_extend_traits;
/// Abstract traits: `Curve` and `Surface`.
pub mod geom_traits;
/// ID structure with `Copy`, `Hash` and `Eq` using raw pointers
pub mod id;
/// Setting Tolerance
pub mod tolerance;
