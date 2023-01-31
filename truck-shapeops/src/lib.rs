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

mod divide_face;
mod faces_classification;
mod integrate;
mod intersection_curve;
mod loops_store;
mod polyline_construction;
pub use integrate::{and, or, ShapeOpsCurve, ShapeOpsSurface};

mod alternative;
