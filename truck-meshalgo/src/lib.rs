//! Geometric mesh algorithms, analyzers, filters, tessellations, and GPU acceleration.
//!
//! # Hardware Acceleration & Intelligent Dispatch
//!
//! `truck-meshalgo` provides hardware-accelerated geometric algorithms targeting both
//! NVIDIA Ampere CUDA (SM 8.6 PTX) and cross-platform Vulkan compute shaders (`wgpu`).
//!
//! Geometric operations automatically route to the fastest execution target (Single-Thread CPU,
//! Multi-Thread CPU via Rayon, Vulkan Compute, or CUDA Ampere PTX) based on empirical startup
//! micro-benchmarks calibrated to the host system and cached persistently.
//!
//! # Multi-GPU Concurrency
//!
//! When multiple physical GPUs are present (e.g. an RTX 3080 and RTX 3060), the dispatcher
//! automatically detects all installed devices in [`dispatch::GpuMetadata`] and partitions high-throughput
//! batch workloads (distance queries, in-out containment, raycasting, and clearance evaluation)
//! across all GPUs concurrently using `rayon::join`, dynamically weighted by their SM counts.
//!
//! # Explicit Warmup
//!
//! To eliminate the one-time cold-start latency (CUDA driver primary context retention,
//! PTX JIT compilation, and Vulkan pipeline state creation) on the first dispatched call,
//! interactive applications can invoke [`warmup`]:
//!
//! ```no_run
//! // Spawn on a background thread during application startup or splash screen:
//! std::thread::spawn(|| {
//!     truck_meshalgo::warmup();
//! });
//! ```

#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]
#![allow(clippy::manual_div_ceil)]
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

/// polygon mesh analyzers, including
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

/// GPU-accelerated algorithms based on wgpu / Vulkan
#[cfg(feature = "gpu")]
pub mod gpu;

/// Intelligent kernel dispatcher and dynamic calibration engine
pub mod dispatch;
pub use dispatch::warmup;

/// This module contains all traits and re-exports `truck_polymesh`.
pub mod prelude {
    #[cfg(feature = "analyzers")]
    pub use crate::analyzers::*;
    pub use crate::dispatch::*;
    #[cfg(feature = "filters")]
    pub use crate::filters::*;
    #[cfg(feature = "gpu")]
    pub use crate::gpu::*;
    pub use crate::rexport_polymesh::*;
    #[cfg(feature = "tessellation")]
    pub use crate::tessellation::*;
    #[cfg(feature = "vtk")]
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::vtk::*;
}
