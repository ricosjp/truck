#[macro_use]
extern crate lazy_static;

use std::collections::VecDeque;

/// Vertex, the minimum topological unit.  
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Edge {
    vertices: (Vertex, Vertex),
    orientation: bool,
    id: usize,
}

/// Wire, a simple path or cycle which consists some edges.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Wire {
    edge_list: VecDeque<Edge>,
}

/// Face, attatched to a closed wire.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Face {
    boundary: Wire,
    id: usize,
}

/// Shell, a connected compounded faces.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Shell {
    face_list: Vec<Face>,
}

/// Solid, attached to a closed shells.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Solid {
    boundaries: Vec<Shell>,
}

pub type Result<T> = std::result::Result<T, crate::errors::Error>;

pub mod edge;
pub mod errors;
pub mod face;
pub mod id;
pub mod shell;
pub mod solid;
pub mod vertex;
pub mod wire;
