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
#![allow(clippy::many_single_char_names)]

extern crate serde;
extern crate truck_base;
extern crate truck_geotrait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use truck_base::bounding_box::Bounded;

const INCLUDE_CURVE_TRIALS: usize = 100;
const PRESEARCH_DIVISION: usize = 50;

/// re-export `truck_base`
pub mod base {
    pub use truck_base::bounding_box::*;
    pub use truck_base::cgmath64::*;
    pub use truck_base::tolerance::*;
    pub use truck_base::{hash, hash::HashGen};
    pub use truck_base::{assert_near, assert_near2};
    pub use truck_geotrait::*;
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

/// Declares some decorators
pub mod decorators;
pub use decorators::*;
