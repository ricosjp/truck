//! `truck-modeling` is a crate for modeling shapes by integrating geometry and topology.

#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

pub use truck_base::{cgmath64::*, geom_traits::*, tolerance::*};

/// geometrical elements
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

/// topological elements
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

/// topological utility: [`Mapped`], [`Sweep`], and [`ClosedSweep`].
/// 
/// [`Mapped`]: ./topo_traits/trait.Mapped.html
/// [`Sweep`]: ./topo_traits/trait.Sweep.html
/// [`ClosedSweep`]: ./topo_traits/trait.ClosedSweep.html
pub mod topo_traits {
    /// Mapping, duplicates and moves a topological element.
    pub trait Mapped<P, C, S>: Sized {
        /// Returns a new topology whose points are mapped by `point_closure`,
        /// curves are mapped by `curve_closure`,
        /// and surfaces are mapped by `surface_closure`.
        fn mapped<FP: Fn(&P) -> P, FC: Fn(&C) -> C, FS: Fn(&S) -> S>(
            &self,
            point_mapping: &FP,
            curve_mapping: &FC,
            surface_mapping: &FS,
        ) -> Self;

        /// Returns another topology whose points, curves, and surfaces are cloned.
        fn topological_clone(&self) -> Self
        where
            P: Clone,
            C: Clone,
            S: Clone, {
            self.mapped(&Clone::clone, &Clone::clone, &Clone::clone)
        }
    }

    /// Abstract sweeping, builds a circle-arc, a prism, a half torus, and so on.
    pub trait Sweep<P, C, S> {
        /// The struct of sweeped topology.
        type Swept;
        /// Transform topologies and connect vertices and edges in boundaries.
        fn sweep<
            FP: Fn(&P) -> P,
            FC: Fn(&C) -> C,
            FS: Fn(&S) -> S,
            CP: Fn(&P, &P) -> C,
            CE: Fn(&C, &C) -> S,
        >(
            &self,
            point_mapping: &FP,
            curve_mapping: &FC,
            surface_mapping: &FS,
            connect_points: &CP,
            connect_curve: &CE,
        ) -> Self::Swept;
    }

    /// closed sweep, builds a closed torus, and so on. 
    pub trait ClosedSweep<P, C, S> {
        /// The struct of sweeped topology.
        type ClosedSwept;
        /// Transform topologies and connect vertices and edges in boundaries.
        fn closed_sweep<
            FP: Fn(&P) -> P,
            FC: Fn(&C) -> C,
            FS: Fn(&S) -> S,
            CP: Fn(&P, &P) -> C,
            CE: Fn(&C, &C) -> S,
        >(
            &self,
            point_mapping: &FP,
            curve_mapping: &FC,
            surface_mapping: &FS,
            connect_points: &CP,
            connect_curves: &CE,
            division: usize,
        ) -> Self::ClosedSwept;
    }
}
pub use topo_traits::*;

/// the building model utility API
pub mod builder;
mod closed_sweep;
mod geom_impls;
mod mapped;
mod sweep;
mod topo_impls;
