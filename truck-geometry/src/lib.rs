//! Geometrical structs: knot vector, B-spline and NURBS

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

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use truck_base::bounding_box::Bounded;

const INCLUDE_CURVE_TRIALS: usize = 100;
const PRESEARCH_DIVISION: usize = 50;

/// re-export `truck_base`
pub mod base {
    pub use truck_base::bounding_box::BoundingBox;
    pub use truck_base::cgmath64::*;
    pub use truck_base::tolerance::*;
    pub use truck_base::{assert_near, assert_near2};
    pub use truck_base::{hash, hash::HashGen};
    pub use truck_geotrait::*;
}
/// Declares the nurbs
pub mod nurbs;

/// Enumerats `Error`.
pub mod errors;

/// Declares the specified gememetric items: Plane, Sphere, and so on.
pub mod specifieds;

/// Declares some decorators
pub mod decorators;

/// re-export all modules.
pub mod prelude {
    use crate::*;
    pub use base::*;
    pub use decorators::*;
    pub use errors::*;
    pub use nurbs::*;
    pub use specifieds::*;
}
