use crate::*;

impl PolygonMesh {
    pub fn bounding_box(&self) -> BoundingBox {
        self.positions.iter().collect()
    }
}
