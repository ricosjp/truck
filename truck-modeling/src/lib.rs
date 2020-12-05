pub use truck_base::{cgmath64::*, tolerance::*};

// geometrical elements
pub mod geometry {
    use super::*;
    pub use truck_geometry::KnotVec;
    /// 4-dimensional B-spline curve
    pub type BSplineCurve = truck_geometry::BSplineCurve<Vector4>;
    /// 4-dimensional B-spline surface
    pub type BSplineSurface = truck_geometry::BSplineSurface<Vector4>;
    /// 3-dimensional NURBS curve
    pub type NURBSCurve = truck_geometry::NURBSCurve<Vector4>;
    /// 3-dimensional NURBS surface
    pub type NURBSSurface = truck_geometry::NURBSSurface<Vector4>;
}
pub use geometry::*;

// topological elements
pub mod topology {
    use super::*;
    /// Vertex, the minimum topological unit.
    pub type Vertex = truck_topology::Vertex<Point3>;
    /// Edge, which consists two vertices.
    pub type Edge = truck_topology::Edge<Point3, NURBSCurve>;
    /// Wire, a path or cycle which consists some edges.
    pub type Wire = truck_topology::Wire<Point3, NURBSCurve>;
    /// Face, attatched to a simple and closed wire.
    pub type Face = truck_topology::Face<Point3, NURBSCurve, NURBSSurface>;
    /// Shell, a connected compounded faces.
    pub type Shell = truck_topology::Shell<Point3, NURBSCurve, NURBSSurface>;
    /// Solid, attached to a closed shells.
    pub type Solid = truck_topology::Solid<Point3, NURBSCurve, NURBSSurface>;

    /// The id of vertex. `Copy` trait is implemented.
    pub type VertexID = truck_topology::VertexID<Point3>;
    /// The id that does not depend on the direction of the edge.
    pub type EdgeID = truck_topology::EdgeID<NURBSCurve>;
    /// The id that does not depend on the direction of the face.
    pub type FaceID = truck_topology::FaceID<NURBSSurface>;

    pub use truck_topology::{errors::Error, shell::ShellCondition, Result};
}
pub use topology::*;

/// the building model utility API
pub mod builder;
pub mod mapped;
pub mod sweep;
pub mod closed_sweep;
mod geom_impls;
