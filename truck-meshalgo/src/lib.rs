//! Mesh algorighms, include tessellations of the shape.

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

use common::*;

/// re-export polymesh
pub mod rexport_polymesh {
    pub use truck_polymesh::*;
}
use truck_polymesh::{StandardVertex as Vertex, *};

/// polygon mesh analizers, including
///
/// - determines topological properties: connectivity, boundary extraction, or shell conditions (closed or oriented)
/// - detects collisions between two meshes and extracts interference lines
/// - investigates positional relations between mesh and point clouds.
#[cfg(feature = "analyzers")]
pub mod analyzers;
mod common;
/// Edits meshes. Add normals, optimizing data, and so on.
#[cfg(feature = "filters")]
pub mod filters;
/// Tessellates shapes.
#[cfg(feature = "tessellation")]
pub mod tessellation;

/// VTK Output
#[cfg(feature = "vtk")]
#[cfg(not(target_arch = "wasm32"))]
pub mod vtk;

/// This module contains all traits and re-exports `truck_polymesh`.
pub mod prelude {
    #[cfg(feature = "analyzers")]
    pub use crate::analyzers::*;
    #[cfg(feature = "filters")]
    pub use crate::filters::*;
    pub use crate::rexport_polymesh::*;
    #[cfg(feature = "tessellation")]
    pub use crate::tessellation::*;
    #[cfg(feature = "vtk")]
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::vtk::*;
}
