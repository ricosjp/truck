//! Crate for operation shapes. Provides boolean operations to Solid, and shape healing for importing shapes from other CAD systems.
//! 
//! # Current Status
//! 
//! ## Boolean Operation
//!
//! Boolean operations are currently supported only for shapes where faces intersect transversally.
//! Cases where faces are tangent to each other are not yet supported.
//! Furthermore, performance optimization using BSP (Binary Space Partitioning) or similar methods remains a future task.
//!
//! ## Fillet
//! 
//! Fillets can be applied to a single edge whose end vertices are each adjacent to exactly three faces.
//! Continuous edges are currently unsupported.

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

mod healing;
pub use healing::{RobustSplitClosedEdgesAndFaces, SplitClosedEdgesAndFaces};
mod transversal;
pub use transversal::{and, or, ShapeOpsCurve, ShapeOpsSurface};
mod alternative;

/// Attaching fillet
///
/// # Current Status
/// Fillets can be applied to a single edge whose end vertices are each adjacent to exactly three faces.
/// Continuous edges are currently unsupported.
pub mod fillet;
