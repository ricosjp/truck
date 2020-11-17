pub use cgmath::Rad;

// geometrical elements
pub mod geometry {
    pub use truck_geometry::*;
    pub type NURBSCurve = truck_geometry::NURBSCurve<Vector4>;
    pub type NURBSSurface = truck_geometry::NURBSSurface<Vector4>;
    pub type CurveCollector = truck_geometry::CurveCollector<Vector4>;
}
pub use geometry::*;

// topological elements
pub mod topology {
    use crate::geometry::*;
    pub type Vertex = truck_topology::Vertex<Point3>;
    pub type Edge = truck_topology::Edge<Point3, NURBSCurve>;
    pub type Wire = truck_topology::Wire<Point3, NURBSCurve>;
    pub type Face = truck_topology::Face<Point3, NURBSCurve, NURBSSurface>;
    pub type Shell = truck_topology::Shell<Point3, NURBSCurve, NURBSSurface>;
    pub type Solid = truck_topology::Solid<Point3, NURBSCurve, NURBSSurface>;

    pub type VertexID = truck_topology::VertexID<Point3>;
    pub type EdgeID = truck_topology::EdgeID<NURBSCurve>;
    pub type FaceID = truck_topology::FaceID<NURBSSurface>;

    pub use truck_topology::{errors::Error, shell::ShellCondition, Mapped, Result, Sweep};
}
pub use topology::*;

pub mod builder;
pub mod closed_sweep;
mod geom_impls;
