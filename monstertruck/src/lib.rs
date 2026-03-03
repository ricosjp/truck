//! # `monstertruck`
//!
//! A Rust CAD kernel -- B-rep modeling, NURBS, meshing, and GPU rendering.
//!
//! Meta-crate that re-exports all `monstertruck-*` sub-crates via feature flags.
//! Enable only what you need, or use `full` for everything.
//!
//! ```toml
//! [dependencies]
//! monstertruck = { version = "0.1", features = ["full"] }
//! ```

pub use monstertruck_core as core;

#[cfg(feature = "traits")]
pub use monstertruck_traits as traits;

#[cfg(feature = "derive")]
pub use monstertruck_derive as derive;

#[cfg(feature = "geometry")]
pub use monstertruck_geometry as geometry;

#[cfg(feature = "topology")]
pub use monstertruck_topology as topology;

#[cfg(feature = "mesh")]
pub use monstertruck_mesh as mesh;

#[cfg(feature = "modeling")]
pub use monstertruck_modeling as modeling;

#[cfg(feature = "meshing")]
pub use monstertruck_meshing as meshing;

#[cfg(feature = "solid")]
pub use monstertruck_solid as solid;

#[cfg(feature = "assembly")]
pub use monstertruck_assembly as assembly;

#[cfg(feature = "step")]
pub use monstertruck_step as step;

#[cfg(feature = "gpu")]
pub use monstertruck_gpu as gpu;

#[cfg(feature = "render")]
pub use monstertruck_render as render;
