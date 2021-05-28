//! Mesh algorighms, include tessellations of the shape.

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

use truck_polymesh::*;
use common::*;

/// polygon mesh analizers, including
/// 
/// - determines topological properties: connectivity, boundary extraction, or shell conditions (colsed or oriented)
/// - detects collisions between two meshes and extracts interference lines
/// - investigates positional relations between mesh and point clouds.
pub mod analyzers;
mod common;
pub mod filters;
pub mod tessellation;

pub mod prelude {
    pub use truck_polymesh::*;
    pub use crate::analyzers::*;
    pub use crate::filters::*;
    pub use crate::tessellation::*;
}
