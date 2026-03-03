//! Geometric primitives for CAD modeling: B-spline and NURBS curves/surfaces,
//! knot vectors, and decorator types (revolved, extruded, intersection curves).

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
use std::{fmt::Debug, ops::Bound};
use monstertruck_core::bounding_box::Bounded;

const INCLUDE_CURVE_TRIALS: usize = 100;
const PRESEARCH_DIVISION: usize = 50;

/// re-export `monstertruck_core`
pub mod base {
    pub use monstertruck_core::{
        assert_near, assert_near2, bounding_box::BoundingBox, cgmath64::*, hash, hash::HashGen,
        prop_assert_near, prop_assert_near2, tolerance::*,
    };
    pub use monstertruck_traits::*;
}
/// NURBS and B-spline curves, surfaces, and knot vectors.
pub mod nurbs;

/// Error types for geometry operations.
pub mod errors;

/// Concrete geometric primitives: [`Plane`], [`Sphere`], [`Line`], etc.
pub mod specifieds;

/// Composite geometry: revolved curves, intersection curves, processor wrappers.
pub mod decorators;

/// T-Spline and T-NURCC surface types.
pub mod t_spline;

/// re-export all modules.
pub mod prelude {
    use crate::*;
    pub use base::*;
    pub use decorators::*;
    pub use errors::*;
    pub use nurbs::*;
    pub use specifieds::*;
    pub use t_spline::*;
}
