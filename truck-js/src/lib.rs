//! Wasm wrapper API for truck

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

use derive_more::*;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// truck struct wrapped by wasm
pub trait IntoWasm: Sized {
    /// wasm wrapper struct
    type WasmWrapper: From<Self>;
    /// Into wasm wrapper
    fn into_wasm(self) -> Self::WasmWrapper { self.into() }
}

mod shape;
pub use shape::{AbstractShape, Edge, Face, Shell, Solid, Vertex, Wire};
/// the building model utility API
pub mod builder;
mod polygon;
/// the boolean operators: `and`, `or`, `not`.
pub mod shapeops;
pub use polygon::{PolygonBuffer, PolygonMesh, STLType};
