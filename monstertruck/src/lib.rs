//! # `monstertruck`
//!
//! A Rust CAD kernel -- B-rep modeling, NURBS, meshing, and GPU rendering.
//!
//! **This is a placeholder release (0.0.1).** The full meta-crate with
//! feature-gated re-exports of all `monstertruck-*` sub-crates will arrive
//! in 0.1.0.
//!
//! In the meantime, depend on the sub-crates directly:
//!
//! ```toml
//! [dependencies]
//! monstertruck-modeling = "0.1"
//! monstertruck-meshing = "0.1"
//! ```

pub use monstertruck_core as core;
