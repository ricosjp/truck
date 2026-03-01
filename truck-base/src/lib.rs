//! Basic structs and traits: importing cgmath, curve and surface traits, tolerance

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

/// Defines bounding box
pub mod bounding_box;
/// Redefines vectors, matrices or points with scalar = f64.
pub mod cgmath64;
/// Additional traits for cgmath
pub mod cgmath_extend_traits;
/// Utilities for performing calculations related to differentiation
pub mod derivatives;
/// Utility
pub mod entry_map;
/// Deterministic hash functions
pub mod hash;
/// Id structure with `Copy`, `Hash` and `Eq` using raw pointers
pub mod id;
pub mod newton;
/// Setting Tolerance
pub mod tolerance;

pub use crate::cgmath_extend_traits::*;
pub use crate::derivatives::*;
pub use cgmath::prelude::*;
pub use cgmath::{frustum, ortho, perspective, Deg, Rad};
pub use matext4cgmath::*;

macro_rules! f64_type {
    ($typename: ident) => {
        /// redefinition, scalar = `f64`.
        pub type $typename = cgmath::$typename<f64>;
    };
    ($a: ident, $($b: ident), *) => {
        f64_type!($a);
        f64_type!($($b),*);
    };
}

f64_type!(Vector1, Vector2, Vector3, Vector4, Matrix2, Matrix3, Matrix4, Point1, Point2, Point3);
