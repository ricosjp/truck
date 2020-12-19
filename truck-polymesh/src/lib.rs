extern crate truck_topology as topology;

/// re-export `truck_base`.
pub mod base {
    pub use truck_base::{bounding_box::*, cgmath64::*, geom_traits::*, tolerance::*};
}
pub use base::*;

/// mesh data
#[derive(Clone, Debug, Default)]
pub struct PolygonMesh {
    /// List of positions
    pub positions: Vec<Point3>,
    /// List of texture matrix
    pub uv_coords: Vec<Vector2>,
    /// List of normal vectors
    pub normals: Vec<Vector3>,
    /// triangle faces
    pub tri_faces: Vec<[[usize; 3]; 3]>,
    /// quadrangle faces
    pub quad_faces: Vec<[[usize; 3]; 4]>,
    /// `n`-gon faces where `n` is more than 4.
    pub other_faces: Vec<Vec<[usize; 3]>>,
}

/// structured quadrangle mesh
#[derive(Clone, Debug)]
pub struct StructuredMesh {
    pub positions: Vec<Vec<Point3>>,
    pub uv_division: (Vec<f64>, Vec<f64>),
    pub normals: Vec<Vec<Vector3>>,
}

/// the decorator for mesh handling
#[derive(Clone, Debug)]
pub struct MeshHandler {
    mesh: PolygonMesh,
}

pub mod errors;
mod extract_topology;
mod healing;
mod mesh_handler;
mod meshing_shape;
pub mod obj;
mod polygon_mesh;
mod smoothing;
mod splitting;
mod structured_mesh;
mod structuring;

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
