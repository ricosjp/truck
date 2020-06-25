//! # Overview
//! `truck_topology` is a crate for describing topological information in a boundary representation.
//! ## Example
//! The following sample code is a description of a topological tetrahedron as a solid model
//! by this package.
//! ```
//! use truck_topology::*;
//!
//! // Create vertices. A tetrahedron has four vertices.
//! let v = Vertex::news(4);
//!
//! // Create edges. Vertex is implemented the Copy trait.
//! let edge = [
//!     Edge::new(v[0], v[1]),
//!     Edge::new(v[0], v[2]),
//!     Edge::new(v[0], v[3]),
//!     Edge::new(v[1], v[2]),
//!     Edge::new(v[1], v[3]),
//!     Edge::new(v[2], v[3]),
//! ];
//!
//! // Create boundaries of faces as the wire.
//! // Edge is implemented the Copy trait.
//! let wire = vec![
//!     Wire::from(vec![edge[0], edge[3], edge[1].inverse()]),
//!     Wire::from(vec![edge[1], edge[5], edge[2].inverse()]),
//!     Wire::from(vec![edge[2], edge[4].inverse(), edge[0].inverse()]),
//!     Wire::from(vec![edge[3], edge[5], edge[4].inverse()]),
//! ];
//!
//! // Create faces by the boundary wires.
//! // The boundary of face must be simple and closed.
//! let mut face: Vec<Face> = wire.into_iter().map(|wire| Face::new(wire)).collect();
//! face[3].invert();
//!
//! // Create shell of faces. Shell can be created by the Vec<Face>.
//! let shell: Shell = face.into();
//!
//! // Create a tetrahedron solid by the boundary shell.
//! // The boundaries of a solid must be closed and oriented.
//! let solid = Solid::new(vec![shell]);
//! ```
//! ## Elements and containers
//! Main structures in `truck_topology` consist 4 topological elements and 2 topological containers.
//! ### topological elements
//! The following structures are topological elements.
//!
//! * [`Vertex`](./struct.Vertex.html)
//! * [`Edge`](./struct.Edge.html)
//! * [`Face`](./struct.Face.html)
//! * [`Solid`](./struct.Solid.html)
//!
//! Except `Solid`, each topological element has a unique `id` for each instance.
//! In higher-level packages, by mapping this `id` to geometric information, you can draw a solid shape.
//! ### topological containers
//! The following structures are topological container.
//!
//! * [`Wire`](./struct.Wire.html)
//! * [`Shell`](./struct.Shell.html)
//!
//! The entities of `Wire` and `Shell` are `std::collections::VecDeque<Edge>` and `std::vec::Vec<Face>`,
//! respectively, and many methods inherited by `Deref` and `DerefMut`.
//! These containers are used for creating higher-dimentional topological elements and checked the
//! regularity (e.g. connectivity, closedness, and so on) before creating these elements.

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

#[macro_use]
extern crate lazy_static;

use std::collections::VecDeque;

/// Vertex, the minimum topological unit.
///
/// The constructor `Vertex::new()` creates a different vertex each time.
/// These vertices are uniquely identified by their `id`.
/// ```
/// # use truck_topology::Vertex;
/// let v0 = Vertex::new(); // one vertex
/// let v1 = Vertex::new(); // another vertex
/// assert_ne!(v0.id(), v1.id()); // two vertices are different
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Vertex {
    id: usize,
}

/// Edge, which consists two vertices.
///
/// The constructors `Edge::new()`, `Edge::try_new()`, and `Edge::new_unchecked()`
/// create a different edge each time, even if the end vertices are the same one.
/// An edge is uniquely identified by their `id`.
/// ```
/// # use truck_topology::*;
/// let v = Vertex::news(2);
/// let edge0 = Edge::new(v[0], v[1]);
/// let edge1 = Edge::new(v[0], v[1]);
/// assert_ne!(edge0.id(), edge1.id());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Edge {
    vertices: (Vertex, Vertex),
    orientation: bool,
    id: usize,
}

/// Wire, a path or cycle which consists some edges.
///
/// The entity of this struct is `VecDeque<Edge>` and almost methods are inherited from
/// `VecDeque<Edge>` by `Deref` and `DerefMut` traits.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Wire {
    edge_list: VecDeque<Edge>,
}

/// Face, attatched to a simple and closed wire.
///
/// The constructors `Face::new()`, `Face::try_new()`, and `Face::new_unchecked()`
/// create a different faces each time, even if the boundary wires are the same one.
/// A face is uniquely identified by their `id`.
/// ```
/// # use truck_topology::*;
/// # let v = Vertex::news(2);
/// # let edge0 = Edge::new(v[0], v[1]);
/// # let edge1 = Edge::new(v[1], v[0]);
/// # let wire = Wire::from(vec![edge0, edge1]);
/// // let wire: Wire = ...;
/// let face0 = Face::new(wire.clone());
/// let face1 = Face::new(wire);
/// assert_ne!(face0.id(), face1.id());
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Face {
    boundary: Wire,
    orientation: bool,
    id: usize,
}

/// Shell, a connected compounded faces.
///
/// The entity of this struct is `Vec<Face>` and almost methods are inherited from
/// `Vec<Face>` by `Deref` and `DerefMut` traits.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Shell {
    face_list: Vec<Face>,
}

/// Solid, attached to a closed shells.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Solid {
    boundaries: Vec<Shell>,
}

/// `Result` with crate's errors.
pub type Result<T> = std::result::Result<T, crate::errors::Error>;

trait RemoveTry<T> {
    fn remove_try(self) -> T;
}

impl<T> RemoveTry<T> for Result<T> {
    fn remove_try(self) -> T {
        match self {
            Ok(got) => got,
            Err(error) => panic!("{}", error),
        }
    }
}

#[doc(hidden)]
pub mod edge;
/// classifies the errors that can occur in this crate.
pub mod errors;
/// Defines the boundary iterator.
pub mod face;
#[doc(hidden)]
pub mod id;
/// classifies shell conditions and defines the face iterators.
pub mod shell;
#[doc(hidden)]
pub mod solid;
#[doc(hidden)]
pub mod vertex;
/// define the edge iterators and the vertex iterator.
pub mod wire;
