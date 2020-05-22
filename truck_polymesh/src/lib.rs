extern crate truck_geometry as geometry;
extern crate truck_topology as topology;
use geometry::{Vector2, Vector3};

/// mesh data
#[derive(Clone, Debug, Default)]
pub struct PolygonMesh {
    /// List of positions
    pub vertices: Vec<Vector3>,
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
    points: Vec<Vec<Vector3>>,
    uv_division: (Vec<f64>, Vec<f64>),
    normals: Vec<Vec<Vector3>>,
}

#[derive(Clone, Debug)]
pub struct MeshHandler {
    mesh: PolygonMesh,
}

impl MeshHandler {
    pub fn new(mesh: PolygonMesh) -> MeshHandler {
        MeshHandler {
            mesh: mesh,
        }
    }
}

impl std::convert::From<MeshHandler> for PolygonMesh {
    fn from(handler: MeshHandler) -> PolygonMesh { handler.mesh }
}

impl std::convert::From<PolygonMesh> for MeshHandler {
    fn from(mesh: PolygonMesh) -> MeshHandler { MeshHandler::new(mesh) }
}

pub mod structured_mesh;
pub mod errors;
pub mod extract_topology;
pub mod healing;
pub mod meshing_shape;
pub mod smoothing;
pub mod splitting;
pub mod structuring;
