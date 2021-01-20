//! Defines polyline-polygon data structure and some algorithms handling mesh.
//!
//! # Warning
//! This crate is WIP, despite the fact that it is used extensively in the sample code.
//! Specifically, member variables of `PolygonMesh` can be hidden at any time.
//! `MeshHandler`, which is hidden in the documentation, may be deprecated and
//! mesh handling may be done as a trait implemented to `PolygonMesh`.
//! We will move up one minor version when we make these changes.

/*
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
*/

extern crate truck_topology as topology;
use serde::{Deserialize, Serialize};

/// re-export `truck_base`.
pub mod base {
    pub use truck_base::{bounding_box::*, cgmath64::*, geom_traits::*, tolerance::*};
}
pub use base::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Vertex {
    pub pos: usize,
    pub uv: Option<usize>,
    pub nor: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Faces {
    tri_faces: Vec<[Vertex; 3]>,
    quad_faces: Vec<[Vertex; 4]>,
    other_faces: Vec<Vec<Vertex>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolygonMesh {
    positions: Vec<Point3>,
    uv_coords: Vec<Vector2>,
    normals: Vec<Vector3>,
    faces: Faces,
}

/// structured quadrangle mesh
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructuredMesh {
    positions: Vec<Vec<Point3>>,
    uv_division: Option<(Vec<f64>, Vec<f64>)>,
    normals: Option<Vec<Vec<Vector3>>>,
}

pub type Result<T> = std::result::Result<T, errors::Error>;

pub mod prelude;
/// Defines errors
pub mod errors;
//mod extract_topology;
mod eliminate_waste;
mod meshing_shape;
/// I/O of wavefront obj
pub mod obj;
mod polygon_mesh;
mod normal_filters;
mod splitting;
mod structured_mesh;
//mod structuring;

#[inline(always)]
fn get_tri<T: Clone>(face: &[T], idx0: usize, idx1: usize, idx2: usize) -> [T; 3] {
    [face[idx0].clone(), face[idx1].clone(), face[idx2].clone()]
}

trait CosAngle {
    fn cos_angle(&self, other: &Self) -> f64;
}

impl CosAngle for Vector3 {
    fn cos_angle(&self, other: &Self) -> f64 {
        self.dot(*other) / (self.magnitude() * other.magnitude())
    }
}
