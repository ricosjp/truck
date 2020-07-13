#[macro_use]
extern crate truck_geometry as geometry;
extern crate truck_topology as topology;
use geometry::{Vector2, Vector3};

/// mesh data
#[derive(Clone, Debug, Default)]
pub struct PolygonMesh {
    /// List of positions
    pub positions: Vec<Vector3>,
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
    positions: Vec<Vec<Vector3>>,
    uv_division: (Vec<f64>, Vec<f64>),
    normals: Vec<Vec<Vector3>>,
}

/// the decorator for mesh handling
#[derive(Clone, Debug)]
pub struct MeshHandler {
    mesh: PolygonMesh,
}

pub mod errors;
pub mod extract_topology;
pub mod healing;
pub mod mesh_handler;
pub mod meshing_shape;
pub mod polygon_mesh;
pub mod smoothing;
pub mod splitting;
pub mod structured_mesh;
pub mod structuring;

#[inline(always)]
fn get_tri<T: Clone>(face: &[T], idx0: usize, idx1: usize, idx2: usize) -> [T; 3] {
    [face[idx0].clone(), face[idx1].clone(), face[idx2].clone()]
}
