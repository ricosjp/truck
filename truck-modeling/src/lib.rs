//! Integrated modeling algorithms by geometry and topology
//!
//! There are some examples in `truck-modeling/examples`.

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

/// re-export `truck_base`.
pub mod base {
    pub use truck_base::{assert_near, assert_near2, bounding_box::*, cgmath64::*, tolerance::*};
    pub use truck_geotrait::*;
}
pub use base::*;

/// geometrical elements
pub mod geometry;
pub use geometry::*;

/// topological elements
pub mod topology {
    use super::*;
    /// Vertex, the minimum topological unit.
    pub type Vertex = truck_topology::Vertex<Point3>;
    /// Edge, which consists two vertices.
    pub type Edge = truck_topology::Edge<Point3, Curve>;
    /// Wire, a path or cycle which consists some edges.
    pub type Wire = truck_topology::Wire<Point3, Curve>;
    /// Face, attatched to a simple and closed wire.
    pub type Face = truck_topology::Face<Point3, Curve, Surface>;
    /// Shell, a connected compounded faces.
    pub type Shell = truck_topology::Shell<Point3, Curve, Surface>;
    /// Solid, attached to a closed shells.
    pub type Solid = truck_topology::Solid<Point3, Curve, Surface>;

    /// The id of vertex. `Copy` trait is implemented.
    pub type VertexID = truck_topology::VertexID<Point3>;
    /// The id that does not depend on the direction of the edge.
    pub type EdgeID = truck_topology::EdgeID<Curve>;
    /// The id that does not depend on the direction of the face.
    pub type FaceID = truck_topology::FaceID<Surface>;

    pub use truck_topology::shell::ShellCondition;
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
        #[doc(hidden)]
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

    /// Abstract multi sweeping, builds a circle-arc, a prism, a half torus, and so on.
    pub trait MultiSweep<P, C, S> {
        /// The struct of sweeped topology.
        type Swept;
        /// Transform topologies and connect vertices and edges in boundaries.
        fn multi_sweep<
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
            division: usize,
        ) -> Self::Swept;
    }

    /// closed sweep, builds a closed torus, and so on.
    pub trait ClosedSweep<P, C, S>: MultiSweep<P, C, S> {
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
        ) -> Self::Swept;
    }
}
pub use topo_traits::*;

/// `Result` with crate's errors.
pub type Result<T> = std::result::Result<T, errors::Error>;

/// the building model utility API
pub mod builder;
/// shape operators: `and`, `or`, `not`.
pub mod shapeops {
    use crate::Solid;
    const SHAPEOPS_TOLERANCE: f64 = 0.05;
    /// and operator.
    pub fn and(solid0: &Solid, solid1: &Solid) -> Solid {
        truck_shapeops::and(solid0, solid1, SHAPEOPS_TOLERANCE)
            .expect("Shape is invalid or too complicated.")
    }
    /// or operator.
    pub fn or(solid0: &Solid, solid1: &Solid) -> Solid {
        truck_shapeops::or(solid0, solid1, SHAPEOPS_TOLERANCE)
            .expect("Shape is invalid or too complicated.")
    }
    /// not operator.
    pub fn not(solid: &Solid) -> Solid {
        let mut solid = solid.clone();
        solid.not();
        solid
    }
    /// and operator with some options.
    pub fn try_and_with_tolerance(solid0: &Solid, solid1: &Solid, tol: f64) -> Option<Solid> {
        truck_shapeops::and(solid0, solid1, tol)
    }
    /// or operator with some options.
    pub fn try_or_with_tolerance(solid0: &Solid, solid1: &Solid, tol: f64) -> Option<Solid> {
        truck_shapeops::or(solid0, solid1, tol)
    }
}
mod closed_sweep;
/// declare errors
pub mod errors;
mod geom_impls;
mod mapped;
mod multi_sweep;
mod sweep;
mod topo_impls;
