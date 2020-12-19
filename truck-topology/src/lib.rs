//! # Overview
//! `truck_topology` is a crate for describing topological information in a boundary representation.
//! ## Example
//! The following sample code is a description of a topological tetrahedron as a solid model
//! by this package.
//! ```
//! use truck_topology::*;
//! use std::iter::FromIterator;
//!
//! // Create vertices. A tetrahedron has four vertices.
//! let v = Vertex::news(&[(), (), (), ()]);
//!
//! // Create edges. Vertex is implemented the Copy trait.
//! let edge = [
//!     Edge::new(&v[0], &v[1], ()),
//!     Edge::new(&v[0], &v[2], ()),
//!     Edge::new(&v[0], &v[3], ()),
//!     Edge::new(&v[1], &v[2], ()),
//!     Edge::new(&v[1], &v[3], ()),
//!     Edge::new(&v[2], &v[3], ()),
//! ];
//!
//! // Create boundaries of faces as the wire.
//! // Edge is implemented the Copy trait.
//! let wire = vec![
//!     Wire::from_iter(vec![&edge[0], &edge[3], &edge[1].inverse()]),
//!     Wire::from_iter(vec![&edge[1], &edge[5], &edge[2].inverse()]),
//!     Wire::from_iter(vec![&edge[2], &edge[4].inverse(), &edge[0].inverse()]),
//!     Wire::from_iter(vec![&edge[3], &edge[5], &edge[4].inverse()]),
//! ];
//!
//! // Create faces by the boundary wires.
//! // The boundary of face must be simple and closed.
//! let mut face: Vec<Face<_, _, _>> = wire.into_iter().map(|wire| Face::new(vec![wire], ())).collect();
//! face[3].invert();
//!
//! // Create shell of faces. Shell can be created by the Vec<Face>.
//! let shell: Shell<_, _, _> = face.into();
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

use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LockResult, Mutex, MutexGuard, TryLockResult};
use truck_base::{geom_traits::*, id::ID, tolerance::*};

/// Vertex, the minimum topological unit.
///
/// The constructor `Vertex::new()` creates a different vertex each time.
/// These vertices are uniquely identified by their `id`.
/// ```
/// # use truck_topology::Vertex;
/// let v0 = Vertex::new(()); // one vertex
/// let v1 = Vertex::new(()); // another vertex
/// assert_ne!(v0, v1); // two vertices are different
/// ```
#[derive(Debug)]
pub struct Vertex<P> {
    point: Arc<Mutex<P>>,
}

/// Edge, which consists two vertices.
///
/// The constructors `Edge::new()`, `Edge::try_new()`, and `Edge::new_unchecked()`
/// create a different edge each time, even if the end vertices are the same one.
/// An edge is uniquely identified by their `id`.
/// ```
/// # use truck_topology::*;
/// let v = Vertex::news(&[(), ()]);
/// let edge0 = Edge::new(&v[0], &v[1], ());
/// let edge1 = Edge::new(&v[0], &v[1], ());
/// assert_ne!(edge0.id(), edge1.id());
/// ```
#[derive(Debug)]
pub struct Edge<P, C> {
    vertices: (Vertex<P>, Vertex<P>),
    orientation: bool,
    curve: Arc<Mutex<C>>,
}

/// Wire, a path or cycle which consists some edges.
///
/// The entity of this struct is `VecDeque<Edge>` and almost methods are inherited from
/// `VecDeque<Edge>` by `Deref` and `DerefMut` traits.
#[derive(Hash, Debug)]
pub struct Wire<P, C> {
    edge_list: VecDeque<Edge<P, C>>,
}

/// Face, attatched to a simple and closed wire.
///
/// The constructors `Face::new()`, `Face::try_new()`, and `Face::new_unchecked()`
/// create a different faces each time, even if the boundary wires are the same one.
/// A face is uniquely identified by their `id`.
/// ```
/// use truck_topology::*;
/// use std::iter::FromIterator;
/// let v = Vertex::news(&[(), ()]);
/// let edge0 = Edge::new(&v[0], &v[1], ());
/// let edge1 = Edge::new(&v[1], &v[0], ());
/// let wire = Wire::from_iter(vec![&edge0, &edge1]);
/// let face0 = Face::new(vec![wire.clone()], ());
/// let face1 = Face::new(vec![wire], ());
/// assert_ne!(face0.id(), face1.id());
/// ```
#[derive(Debug)]
pub struct Face<P, C, S> {
    boundaries: Vec<Wire<P, C>>,
    orientation: bool,
    surface: Arc<Mutex<S>>,
}

/// Shell, a connected compounded faces.
///
/// The entity of this struct is `Vec<Face>` and almost methods are inherited from
/// `Vec<Face>` by `Deref` and `DerefMut` traits.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Shell<P, C, S> {
    face_list: Vec<Face<P, C, S>>,
}

/// Solid, attached to a closed shells.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Solid<P, C, S> {
    boundaries: Vec<Shell<P, C, S>>,
}

/// `Result` with crate's errors.
pub type Result<T> = std::result::Result<T, crate::errors::Error>;

trait RemoveTry<T> {
    fn remove_try(self) -> T;
}

impl<T> RemoveTry<T> for Result<T> {
    #[inline(always)]
    fn remove_try(self) -> T { self.unwrap_or_else(|e| panic!("{}", e)) }
}

/// The id of vertex. `Copy` trait is implemented.
/// # Details
/// Since this struct is implemented `Copy` trait,
/// it is useful to use as a key of hashmaps.
/// ```
/// use truck_topology::*;
/// use std::collections::HashMap;
///
/// let v = Vertex::new(0);
/// let v_id = v.id();
///
/// let mut entity_map = HashMap::new();
/// let mut id_map = HashMap::new();
///
/// entity_map.insert(v.clone(), 0); // v must be cloned for sign up the hashmap.
/// id_map.insert(v_id, 0); // v_id is implemented Copy trait!
/// ```
/// The id does not changed even if the value of point changes.
/// ```
/// use truck_topology::*;
/// let v = Vertex::new(0);
///
/// let entity = *v.try_lock_point().unwrap();
/// let v_id: VertexID<usize> = v.id();
///
/// // Change the point!
/// *v.try_lock_point().unwrap() = 1;
///
/// assert_ne!(entity, *v.try_lock_point().unwrap());
/// assert_eq!(v_id, v.id());
/// ```
pub type VertexID<P> = ID<Mutex<P>>;

/// The id that does not depend on the direction of the edge.
/// # Examples
/// ```
/// use truck_topology::*;
/// let v = Vertex::news(&[(), ()]);
/// let edge0 = Edge::new(&v[0], &v[1], ());
/// let edge1 = edge0.inverse();
/// assert_ne!(edge0, edge1);
/// assert_eq!(edge0.id(), edge1.id());
/// ```
pub type EdgeID<C> = ID<Mutex<C>>;

/// The id that does not depend on the direction of the face.
/// # Examples
/// ```
/// use truck_topology::*;
/// let v = Vertex::news(&[(); 3]);
/// let wire = Wire::from(vec![
///     Edge::new(&v[0], &v[1], ()),
///     Edge::new(&v[1], &v[2], ()),
///     Edge::new(&v[2], &v[0], ()),
/// ]);
/// let face0 = Face::new(vec![wire.clone()], ());
/// let face1 = face0.inverse();
/// let face2 = Face::new(vec![wire], ());
/// assert_ne!(face0, face1);
/// assert_ne!(face0, face2);
/// assert_eq!(face0.id(), face1.id());
/// assert_ne!(face0.id(), face2.id());
/// ```
pub type FaceID<S> = ID<Mutex<S>>;

#[doc(hidden)]
pub mod edge;
/// classifies the errors that can occur in this crate.
pub mod errors;
/// Defines the boundary iterator.
pub mod face;
/// classifies shell conditions and defines the face iterators.
pub mod shell;
#[doc(hidden)]
pub mod solid;
#[doc(hidden)]
pub mod vertex;
/// define the edge iterators and the vertex iterator.
pub mod wire;
#[doc(hidden)]
pub mod wrappup;
