//! Provides boolean operations to Solid

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

mod transversal;
pub use transversal::{and, or, ShapeOpsCurve, ShapeOpsSurface};
mod alternative;
