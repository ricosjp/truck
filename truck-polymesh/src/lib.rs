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

/// re-export `truck_base`.
pub mod base {
    pub use truck_base::{bounding_box::*, cgmath64::*, geom_traits::*, tolerance::*};
}
pub use base::*;

#[derive(Clone, Debug, Default)]
pub struct Faces<V> {
    tri_faces: Vec<[V; 3]>,
    quad_faces: Vec<[V; 4]>,
    other_faces: Vec<Vec<V>>,
}

#[derive(Clone, Debug)]
pub enum PolygonMesh {
    Positions {
        positions: Vec<Point3>,
        faces: Faces<usize>,
    },
    Textured {
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        faces: Faces<[usize; 2]>,
    },
    WithNormals {
        positions: Vec<Point3>,
        normals: Vec<Vector3>,
        faces: Faces<[usize; 2]>
    },
    Complete {
        positions: Vec<Point3>,
        uv_coords: Vec<Vector2>,
        normals: Vec<Vector3>,
        faces: Faces<[usize; 3]>,
    }
}

/// structured quadrangle mesh
#[derive(Clone, Debug)]
pub struct StructuredMesh {
    positions: Vec<Vec<Point3>>,
    uv_division: Option<(Vec<f64>, Vec<f64>)>,
    normals: Option<Vec<Vec<Vector3>>>,
}

#[derive(Clone, Copy, Debug)]
pub enum FacesRef<'a> {
    Positions(&'a Faces<usize>),
    Textured(&'a Faces<[usize; 2]>),
    WithNormals(&'a Faces<[usize; 2]>),
    Complete(&'a Faces<[usize; 3]>),
}

pub type Result<T> = std::result::Result<T, errors::Error>;

/// Defines errors
pub mod errors;
//mod extract_topology;
//mod healing;
//mod meshing_shape;
/// I/O of wavefront obj
pub mod obj;
mod polygon_mesh;
//mod smoothing;
//mod splitting;
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
