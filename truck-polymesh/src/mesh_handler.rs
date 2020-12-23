use crate::*;

/// constructor
impl MeshHandler {
    /// constructor
    pub fn new(mesh: PolygonMesh) -> MeshHandler { MeshHandler { mesh: mesh } }
}

impl From<MeshHandler> for PolygonMesh {
    fn from(handler: MeshHandler) -> PolygonMesh { handler.mesh }
}

impl From<PolygonMesh> for MeshHandler {
    fn from(mesh: PolygonMesh) -> MeshHandler { MeshHandler::new(mesh) }
}