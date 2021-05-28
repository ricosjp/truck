use super::*;

pub trait PositionalRelation {
    fn collision(&self, other: &PolygonMesh) -> Vec<(Point3, Point3)>;
}

impl PositionalRelation for PolygonMesh {
    fn collision(&self, other: &PolygonMesh) -> Vec<(Point3, Point3)> {
        crate::common::collision::collision(self, other)
    }
}
