extern crate truck_geometry as geometry;
extern crate truck_topology as topology;

/// mesh data
#[derive(Clone, Debug, Default)]
pub struct PolygonMesh {
    pub vertices: Vec<[f64; 3]>,
    pub uv_coords: Vec<[f64; 2]>,
    pub normals: Vec<[f64; 3]>,
    pub tri_faces: Vec<[[usize; 3]; 3]>,
    pub quad_faces: Vec<[[usize; 3]; 4]>,
}

#[derive(Clone, Debug)]
pub struct MeshHandler {
    mesh: PolygonMesh,
    gcurv: Vec<f64>,
}

impl MeshHandler {
    pub fn new(mesh: PolygonMesh) -> MeshHandler {
        MeshHandler {
            mesh: mesh,
            gcurv: Vec::new(),
        }
    }
}

impl std::convert::From<MeshHandler> for PolygonMesh {
    fn from(handler: MeshHandler) -> PolygonMesh { handler.mesh }
}

impl std::convert::From<PolygonMesh> for MeshHandler {
    fn from(mesh: PolygonMesh) -> MeshHandler { MeshHandler::new(mesh) }
}

pub mod errors;
pub mod extract_topology;
pub mod healing;
pub mod meshing_shape;
pub mod smoothing;
pub mod splitting;
pub mod structuring;
