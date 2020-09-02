use crate::*;

impl PolygonMesh {
    pub fn bounding_box(&self) -> BoundingBox<Point3> {
        self.positions.iter().collect()
    }
}
