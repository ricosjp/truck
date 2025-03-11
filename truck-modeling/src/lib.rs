//! Integrated modeling algorithms by geometry and topology
//!
//! There are some examples in `truck-modeling/examples`.

#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]
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
    pub use truck_base::{
        assert_near, assert_near2, bounding_box::BoundingBox, cgmath64::*, prop_assert_near,
        prop_assert_near2, tolerance::*,
    };
    pub use truck_geotrait::*;
}
pub use base::*;

/// geometrical elements
pub mod geometry;
pub use geometry::*;

/// topological elements
pub mod topology {
    use crate::{Curve, Point3, Surface};
    truck_topology::prelude!(Point3, Curve, Surface, pub);
}
pub use topology::*;

/// topological utility: [`Mapped`], [`Sweep`], and [`ClosedSweep`].
///
/// [`Mapped`]: ./topo_traits/trait.Mapped.html
/// [`Sweep`]: ./topo_traits/trait.Sweep.html
/// [`ClosedSweep`]: ./topo_traits/trait.ClosedSweep.html
pub mod topo_traits {
    /// Creates closure for transformation
    pub trait GeometricMapping<T>: Copy {
        /// Creates closure for transformation
        fn mapping(self) -> impl Fn(&T) -> T;
    }

    /// Creates closure for connect two geometries
    pub trait Connector<T, H>: Copy {
        /// Creates closure for connect two geometries
        fn connector(self) -> impl Fn(&T, &T) -> H;
    }

    /// Mapping, duplicates and moves a topological element.
    pub trait Mapped<T>: Sized {
        /// Returns a new topology whose points are mapped by `point_closure`,
        /// curves are mapped by `curve_closure`,
        /// and surfaces are mapped by `surface_closure`.
        #[doc(hidden)]
        fn mapped(&self, trans: T) -> Self;
    }

    /// Abstract sweeping, builds a circle-arc, a prism, a half torus, and so on.
    pub trait Sweep<T, Pc, Cc, Swept> {
        /// Transform topologies and connect vertices and edges in boundaries.
        fn sweep(&self, trans: T, point_connector: Pc, curve_connector: Cc) -> Swept;
    }

    /// Abstract multi sweeping, builds a circle-arc, a prism, a half torus, and so on.
    pub trait MultiSweep<T, Pc, Cc, Swept> {
        /// Transform topologies and connect vertices and edges in boundaries.
        fn multi_sweep(
            &self,
            trans: T,
            point_connector: Pc,
            curve_connector: Cc,
            division: usize,
        ) -> Swept;
    }

    /// closed sweep, builds a closed torus, and so on.
    pub trait ClosedSweep<T, Pc, Cc, Swept>: MultiSweep<T, Pc, Cc, Swept> {
        /// Transform topologies and connect vertices and edges in boundaries.
        fn closed_sweep(
            &self,
            trans: T,
            point_connector: Pc,
            curve_connector: Cc,
            division: usize,
        ) -> Swept;
    }
}
pub use topo_traits::*;

/// `Result` with crate's errors.
pub type Result<T> = std::result::Result<T, errors::Error>;

/// the building model utility API
pub mod builder;
mod closed_sweep;
/// declare errors
pub mod errors;
mod geom_impls;
mod mapped;
mod multi_sweep;
/// primitive shapes
pub mod primitive;
mod sweep;
mod topo_impls;
