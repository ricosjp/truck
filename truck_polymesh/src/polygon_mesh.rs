use crate::*;

impl PolygonMesh {
    pub fn bounding_box(&self) -> BoundingBox<Vector3> {
        self.positions.iter().collect()
    }
}
