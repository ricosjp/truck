//! Fillet operations for [`Shell`](truck_topology::Shell) edges.
//!
//! Provides rolling-ball fillet operations: single-edge fillets,
//! fillets with side face updates, and fillets along open or closed wire chains.
//! The [`fillet_edges`] function provides a high-level API that automatically
//! resolves face adjacency from edge IDs.

#[allow(private_interfaces)]
mod edge_select;
#[allow(private_interfaces)]
mod ops;

mod convert;
mod error;
mod geometry;
mod params;
mod topology;
mod types;

#[cfg(test)]
mod tests;

pub use convert::{FilletableCurve, FilletableSurface};
pub use edge_select::{fillet_edges, fillet_edges_generic};
pub use error::FilletError;
pub use ops::{fillet_along_wire, fillet_with_side, simple_fillet};
pub use params::{FilletOptions, FilletProfile, RadiusSpec};
pub use types::ParamCurveLinear;
