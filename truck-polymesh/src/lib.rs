//! Defines polyline-polygon data structure and some algorithms handling mesh.

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

use serde::{Deserialize, Serialize};

/// re-export `truck_base`.
pub mod base {
    pub use truck_base::{bounding_box::BoundingBox, cgmath64::*, hash, hash::HashGen, tolerance::*};
    pub use truck_geotrait::*;
}
pub use base::*;

/// attribution container for polygin mesh
pub trait Attributes<V> {
    /// attribution
    type Output;
    /// get attribution corresponding to vertex
    fn get(&self, vertex: V) -> Option<Self::Output>;
}

/// standard attributions
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StandardAttributes {
    /// positions
    pub positions: Vec<Point3>,
    /// texture uv coordinates
    pub uv_coords: Vec<Vector2>,
    /// normals at vertices
    pub normals: Vec<Vector3>,
}

/// standard attribution
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StandardAttribute {
    /// position
    pub position: Point3,
    /// texture uv coordinate
    pub uv_coord: Option<Vector2>,
    /// normal at vertex
    pub normal: Option<Vector3>,
}

/// Index vertex of a face of the polygon mesh
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct StandardVertex {
    /// index of vertex's position
    pub pos: usize,
    /// index of vertex's texture coordinate
    pub uv: Option<usize>,
    /// index of vertex's normal
    pub nor: Option<usize>,
}

/// Faces of polygon mesh
///
/// To optimize for the case where the polygon mesh consists only triangles and quadrangle,
/// there are vectors which consist by each triangles and quadrilaterals, internally.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Faces<V = StandardVertex> {
    tri_faces: Vec<[V; 3]>,
    quad_faces: Vec<[V; 4]>,
    other_faces: Vec<Vec<V>>,
}

/// Polygon mesh
///
/// The polygon data is held in a method compliant with wavefront obj.
/// Position, uv (texture) coordinates, and normal vectors are held in separate arrays,
/// and each face vertex accesses those values by an indices triple.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PolygonMesh<V = StandardVertex, A = StandardAttributes> {
    attributes: A,
    faces: Faces<V>,
}

/// structured quadrangle mesh
#[derive(Clone, Debug, Serialize)]
pub struct StructuredMesh {
    positions: Vec<Vec<Point3>>,
    uv_division: Option<(Vec<f64>, Vec<f64>)>,
    normals: Option<Vec<Vec<Vector3>>>,
}

/// polyline curve
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolylineCurve<P>(pub Vec<P>);

mod attributes;
/// Defines errors
pub mod errors;
mod expand;
/// Defines triangle
pub mod faces;
mod meshing_shape;
/// wavefront obj I/O
pub mod obj;
/// Defines [`PolygonMeshEditor`](./polygon_mesh/struct.PolygonMeshEditor.html).
pub mod polygon_mesh;
/// Defines generalized polyline curve.
pub mod polyline_curve;
/// STL I/O
pub mod stl;
mod structured_mesh;
