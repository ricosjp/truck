use super::*;
use crate::common::{HashedPointCloud, Triangulate};

pub trait Distance {
    /// `self` and `other` are near shape with the tolerance `tol`.
    /// Here, "near shape with the tolerance `tol`" means Hausdorff distance is smaller than `tol`.
    fn is_near_shape(&self, other: &Self, tol: f64) -> bool;
}

impl Distance for PolygonMesh {
    fn is_near_shape(&self, other: &Self, tol: f64) -> bool {
        let hashed0 = HashedPointCloud::from_points(self.positions(), tol * 2.0);
        let dist2_0 = Triangulate(other).into_iter().fold(None, |dist2, tri| {
            let tmp = hashed0.distance2([
                other.positions()[tri[0].pos],
                other.positions()[tri[1].pos],
                other.positions()[tri[2].pos],
            ]);
            match (dist2, tmp) {
                (Some(a), Some(b)) => Some(f64::min(a, b)),
                (None, _) => tmp,
                (_, None) => dist2,
            }
        });
        let hashed1 = HashedPointCloud::from_points(other.positions(), tol * 2.0);
        let dist2_1 = Triangulate(self).into_iter().fold(None, |dist2, tri| {
            let tmp = hashed1.distance2([
                other.positions()[tri[0].pos],
                other.positions()[tri[1].pos],
                other.positions()[tri[2].pos],
            ]);
            match (dist2, tmp) {
                (Some(a), Some(b)) => Some(f64::min(a, b)),
                (None, _) => tmp,
                (_, None) => dist2,
            }
        });
        (match (dist2_0, dist2_1) {
            (Some(a), Some(b)) => f64::min(a, b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => std::f64::INFINITY,
        }) < tol * tol
    }
}
