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

use common::*;

/// re-export polymesh
pub mod rexport_polymesh {
    pub use truck_polymesh::*;
}
use truck_polymesh::{polygon_mesh::PolygonMeshEditor, StandardVertex as Vertex, *};

/// polygon mesh analizers, including
///
/// - determines topological properties: connectivity, boundary extraction, or shell conditions (colsed or oriented)
/// - detects collisions between two meshes and extracts interference lines
/// - investigates positional relations between mesh and point clouds.
pub mod analyzers;
mod common;
/// Edits meshes. Add normals, optimizing data, and so on.
pub mod filters;
/// Tessellates shapes.
pub mod tessellation;

/// This module contains all traits and re-exports `truck_polymesh`.
pub mod prelude {
    pub use crate::analyzers::*;
    pub use crate::filters::*;
    pub use crate::rexport_polymesh::*;
    pub use crate::tessellation::*;
}
