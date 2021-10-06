mod utils;

use derive_more::*;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod shape_wrappers;
pub use shape_wrappers::{Vertex, Edge, Wire, Face, Shell, Solid, AbstractShape, IntoWasm};
pub mod builder;
