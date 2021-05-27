use super::*;
use crate::common::HashedPointCloud;

pub trait PositionalRelation {
    /// Returns `true` if all points in `point_cloud` is in the neighborhood of `self`
    fn is_clung_to_by(&self, point_cloud: &Vec<Point3>, tol: f64) -> bool;
    fn collision(&self, other: &PolygonMesh) -> Vec<(Point3, Point3)>;
}

impl PositionalRelation for PolygonMesh {
    fn is_clung_to_by(&self, point_cloud: &Vec<Point3>, tol: f64) -> bool {
        HashedPointCloud::from_points(point_cloud, tol * 2.0)
            .distance2(self)
            .map(|dist2| dist2 < tol * tol)
            == Some(true)
    }
    fn collision(&self, other: &PolygonMesh) -> Vec<(Point3, Point3)> {
        crate::common::collision::collision(self, other)
    }
}
