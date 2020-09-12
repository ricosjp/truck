extern crate truck_geometry as geometry;
extern crate truck_topology as topology;
pub use geometry::*;
pub use topology::*;

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
    fn is_arc_of(&self, longer: &Self, hint: f64) -> Option<f64>;
    /// Returns the inverse of a curve
    fn inverse(&self) -> Self;
}

/// a geometry of face
pub trait Surface: Clone {
    type Curve: Curve;
    
}

pub mod point;
pub mod curve;
