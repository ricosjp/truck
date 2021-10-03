use super::*;
mod hashed_point_cloud;
use hashed_point_cloud::HashedPointCloud;
mod sort_end_points;

/// Investigates positional relation between polygon mesh and point cloud.
pub trait WithPointCloud {
    /// Whether all faces of the polygon mesh `self` has intersection with the neighborhood of `point_cloud`.
    /// # Arguments
    /// - `tol`: the radius of the neighborhoods of points in point cloud.
    /// # Panics
    /// `tol` must be more than `TOLERANCE`.
    /// # Examples
    /// ```
    /// use truck_meshalgo::prelude::*;
    /// let positions = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(0.0, 0.0, 2.0),
    ///     Point3::new(1.0, 0.0, 2.0),
    ///     Point3::new(0.0, 1.0, 2.0),
    /// ];
    /// let faces = Faces::from_iter(vec![[0, 1, 2], [3, 4, 5]]);
    /// let mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
    ///
    /// let mut point_cloud = vec![Point3::new(0.25, 0.25, 0.0)];
    /// assert!(!mesh.is_clung_to_by(&point_cloud, 0.001));
    /// point_cloud.push(Point3::new(0.25, 0.25, 2.0));
    /// assert!(mesh.is_clung_to_by(&point_cloud, 0.001));
    /// ```
    fn is_clung_to_by(&self, point_cloud: &Vec<Point3>, tol: f64) -> bool;
    /// Whether the neighborhood of the polygon mesh `self` includes `point_cloud`.
    /// # Panics
    /// `tol` must be more than `TOLERANCE`.
    fn neighborhood_include(&self, point_cloud: &Vec<Point3>, tol: f64) -> bool;
    /// Whether the polygon mesh `self` and `point_cloud` collides.
    /// # Panics
    /// `tol` must be more than `TOLERANCE`.
    fn collide_with_neiborhood_of(&self, point_cloud: &Vec<Point3>, tol: f64) -> bool;
}

impl WithPointCloud for PolygonMesh {
    #[inline(always)]
    fn is_clung_to_by(&self, point_cloud: &Vec<Point3>, tol: f64) -> bool {
        nonpositive_tolerance!(tol);
        HashedPointCloud::from_points(point_cloud, tol * 2.0)
            .distance2(self)
            < tol * tol
    }
    #[inline(always)]
    fn collide_with_neiborhood_of(&self, point_cloud: &Vec<Point3>, tol: f64) -> bool {
        HashedPointCloud::from_points(point_cloud, tol * 2.0).is_colliding(self, tol)
    }
    #[inline(always)]
    fn neighborhood_include(&self, point_cloud: &Vec<Point3>, tol: f64) -> bool {
        sort_end_points::pointcloud_in_polygon_neighborhood(self, point_cloud, tol)
    }
}

// https://iquilezles.org/www/articles/distfunctions/distfunctions.htm
fn distance2_point_triangle(point: Point3, triangle: [Point3; 3]) -> f64 {
    let ab = triangle[1] - triangle[0];
    let ap = point - triangle[0];
    let bc = triangle[2] - triangle[1];
    let bp = point - triangle[1];
    let ca = triangle[0] - triangle[2];
    let cp = point - triangle[2];
    let nor = ab.cross(ca);

    let coef = f64::signum(ab.cross(nor).dot(ap))
        + f64::signum(bc.cross(nor).dot(bp))
        + f64::signum(ca.cross(nor).dot(cp));
    if coef < 2.0 || nor.magnitude().so_small() {
        let a = (ap - ab * f64::clamp(ab.dot(ap) / ab.dot(ab), 0.0, 1.0)).magnitude2();
        let b = (bp - bc * f64::clamp(bc.dot(bp) / bc.dot(bc), 0.0, 1.0)).magnitude2();
        let c = (cp - ca * f64::clamp(ca.dot(cp) / ca.dot(ca), 0.0, 1.0)).magnitude2();
        f64::min(f64::min(a, b), c)
    } else {
        nor.dot(ap) * nor.dot(ap) / nor.magnitude2()
    }
}
