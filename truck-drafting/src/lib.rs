//! 2-dimensional drafting utilities based on truck geometry and topology.

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

/// re-export `truck_base`.
pub mod base {
    pub use truck_base::{
        assert_near, assert_near2, bounding_box::BoundingBox, cgmath64::*, prop_assert_near,
        prop_assert_near2, tolerance::*,
    };
    pub use truck_geotrait::*;
}
pub use base::*;

/// geometrical elements
pub mod geometry;
pub use geometry::*;

/// topological elements for 2-dimensional drafting.
pub mod topology {
    use crate::{Curve, Point2};
    truck_topology::prelude!(Point2, Curve, (), pub);
}
pub use topology::*;

/// basic drafting utilities such as vertices and primitive curves.
pub mod draw;
mod geom_impls;

/// corner treatment utilities such as fillets and chamfers.
pub mod corner;

/// classifies the errors that can occur in this crate.
pub mod errors;
