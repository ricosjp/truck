//! Geometrical structs: knot vector, B-spline and NURBS

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

extern crate serde;
extern crate truck_base;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use truck_base::bounding_box::Bounded;

/// re-export `truck_base`
pub mod base {
    pub use truck_base::bounding_box::*;
    pub use truck_base::cgmath64::*;
    pub use truck_base::geom_traits::*;
    pub use truck_base::tolerance::*;
}
pub use base::*;
/// Declares the nurbs
pub mod nurbs;
pub use nurbs::*;

/// Enumerats `Error`.
pub mod errors;
pub use errors::*;

/// Declares the specified gememetric items: Plane, Sphere, and so on.
pub mod specifieds;
pub use specifieds::*;

pub mod decorators;
pub use decorators::*;

#[doc(hidden)]
#[inline(always)]
pub fn inv_or_zero(delta: f64) -> f64 {
    if delta.so_small() {
        0.0
    } else {
        1.0 / delta
    }
}
