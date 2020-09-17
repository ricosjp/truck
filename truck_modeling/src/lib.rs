extern crate truck_integral;
use cgmath::prelude::*;
pub use cgmath::Rad;
use truck_integral::EdgeEx;

// geometrical elements
pub mod geometry {
    pub use geometry::{
        errors::Error, traits::*, BoundingBox, KnotVec, Matrix2, Matrix3, Matrix4, Point2, Point3,
        Result, Vector2, Vector3, Vector4,
    };
    use truck_integral::*;
    pub type BSplineCurve = geometry::BSplineCurve<Vector4>;
    pub type BSplineSurface = geometry::BSplineSurface<Vector4>;
    pub type CurveCollector = geometry::CurveCollector<Vector4>;
}
use geometry::*;

// topological elements
pub mod topology {
    use crate::geometry::*;
    use truck_integral::*;
    pub type Vertex = topology::Vertex<Vector4>;
    pub type Edge = topology::Edge<Vector4, BSplineCurve>;
    pub type Wire = topology::Wire<Vector4, BSplineCurve>;
    pub type Face = topology::Face<Vector4, BSplineCurve, BSplineSurface>;
    pub type Shell = topology::Shell<Vector4, BSplineCurve, BSplineSurface>;
    pub type Solid = topology::Solid<Vector4, BSplineCurve, BSplineSurface>;

    pub type VertexID = topology::VertexID<Vector4>;
    pub type EdgeID = topology::EdgeID<BSplineCurve>;
    pub type FaceID = topology::FaceID<BSplineSurface>;

    pub use topology::{errors::Error, Mapped, Result, Sweep};
}
use topology::*;

pub mod builder;
pub mod closed_sweep;
mod geom_impls;
