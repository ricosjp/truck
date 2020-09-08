use crate::*;

/// a geometry of vertex
pub trait Point: Clone {
    /// Returns whether `self` is near `other`.
    fn near(&self, other: &Self) -> bool;
}

/// a geometry of edge
pub trait Curve: Clone {
    /// The points obtained by assigning parameters to the curve.
    type Point: Point;
    /// The front end point of the curve.
    fn front(&self) -> Self::Point;
    /// The back end point of the curve.
    fn back(&self) -> Self::Point;
    /// Returns whether `self` is a part of the curve `other`.
    fn is_arc_of(&self, longer: &Self) -> bool;
}

impl<C: Curve> Edge<C::Point, C> {
    /// Returns the conformability of the geometry and the topology.
    /// i.e. returns the following points are the same one.
    /// * the end points calculated by the geometric curve.
    /// * the end points contained by the topological end vertices.
    pub fn is_conformable(&self) -> bool {
        let (gfront, gback) = {
            let curve = self.try_lock_curve().unwrap();
            (curve.front(), curve.back())
        };
        let tfront = self.front().try_lock_point().unwrap().clone();
        let tback = self.back().try_lock_point().unwrap().clone();
        gfront.near(&tfront) && gback.near(&tback)        
    }
}

macro_rules! impl_point_for_integers {
    ($int: ty) => {
        impl Point for $int {
            fn near(&self, other: &Self) -> bool { self == other }
        }
    };
    ($a: ty, $($b: ty), *) => {
        impl_point_for_integers!($a);
        impl_point_for_integers!($($b), *);
    };
}

impl_point_for_integers!(usize, u32, u64, isize, i32, i64);

