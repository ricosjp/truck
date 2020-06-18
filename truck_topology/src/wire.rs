use crate::{Edge, Vertex, Wire};
use std::collections::vec_deque;
use std::collections::{HashSet, VecDeque};
use std::iter::Peekable;

impl Wire {
    /// Creates the empty wire.
    #[inline(always)]
    pub fn new() -> Wire {
        Wire {
            edge_list: VecDeque::new(),
        }
    }
    /// Creates the empty wire with space for at least `capacity` edges.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Wire {
        Wire {
            edge_list: VecDeque::with_capacity(capacity),
        }
    }

    /// Returns an iterator over the edges. Practically, an alias of `iter()`.
    #[inline(always)]
    pub fn edge_iter(&self) -> EdgeIter { self.iter() }
    
    /// Returns a mutable iterator over the edges. Practically, an alias of `iter_mut()`.
    #[inline(always)]
    pub fn edge_iter_mut(&mut self) -> EdgeIterMut { self.iter_mut() }

    /// Creates a consuming iterator. Practically, an alias of `into_iter()`.
    #[inline(always)]
    pub fn edge_into_iter(self) -> EdgeIntoIter { self.into_iter() }

    /// Returns an iterator over the vertices.
    #[inline(always)]
    pub fn vertex_iter(&self) -> VertexIter {
        VertexIter {
            edge_iter: self.edge_iter().peekable(),
            unconti_next: None,
            cyclic: self.is_cyclic(),
        }
    }

    /// Returns the front edge. If `self` is empty wire, returns None.  
    /// Practically, an alias of the inherited method `VecDeque::front()`.
    #[inline(always)]
    pub fn front_edge(&self) -> Option<&Edge> { self.front() }

    /// Returns the front vertex. If `self` is empty wire, returns None.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(3);
    /// let mut wire = Wire::new();
    /// assert_eq!(wire.front_vertex(), None);
    /// wire.push_back(Edge::new(v[1], v[2]));
    /// wire.push_front(Edge::new(v[0], v[1]));
    /// assert_eq!(wire.front_vertex(), Some(v[0]));
    /// ```
    #[inline(always)]
    pub fn front_vertex(&self) -> Option<Vertex> { self.front().map(|edge| edge.front()) }

    /// Returns the back edge. If `self` is empty wire, returns None.  
    /// Practically, an alias of the inherited method `VecDeque::back()`
    #[inline(always)]
    pub fn back_edge(&self) -> Option<&Edge> { self.back() }

    /// Returns the back edge. If `self` is empty wire, returns None.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(3);
    /// let mut wire = Wire::new();
    /// assert_eq!(wire.back_vertex(), None);
    /// wire.push_back(Edge::new(v[1], v[2]));
    /// wire.push_front(Edge::new(v[0], v[1]));
    /// assert_eq!(wire.back_vertex(), Some(v[2]));
    /// ```
    #[inline(always)]
    pub fn back_vertex(&self) -> Option<Vertex> { self.back().map(|edge| edge.back()) }

    /// Returns vertices at both ends.
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(3);
    /// let mut wire = Wire::new();
    /// assert_eq!(wire.back_vertex(), None);
    /// wire.push_back(Edge::new(v[1], v[2]));
    /// wire.push_front(Edge::new(v[0], v[1]));
    /// assert_eq!(wire.ends_vertices(), Some((v[0], v[2])));
    /// ```
    #[inline(always)]
    pub fn ends_vertices(&self) -> Option<(Vertex, Vertex)> {
        match (self.front_vertex(), self.back_vertex()) {
            (Some(got0), Some(got1)) => Some((got0, got1)),
            _ => None,
        }
    }

    /// Inserts vertex at the `index`th edge.
    /// * Creates two new edges whose ends are `(wire[i].front(), vertex)` and `(vertex, wire[i].back())`.
    /// * Removes edge from the wire and insert the new edges as `index`th and `(index + 1)`th edge.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// # let v = Vertex::news(4);
    /// # let mut wire = Wire::from(vec![
    /// #   Edge::new(v[0], v[1]),
    /// #   Edge::new(v[1], v[2]),
    /// #   Edge::new(v[2], v[3]),
    /// # ]);
    /// // let mut wire: Wire = ...
    /// assert_eq!(wire.len(), 3);
    /// let (front, back) = wire[1].ends();
    /// let vertex = Vertex::new();
    /// wire.insert_vertex(1, vertex);
    /// assert_eq!(wire[1].ends(), (front, vertex));
    /// assert_eq!(wire[2].ends(), (vertex, back));
    /// ```
    /// # Panics
    /// Panics if `index >= self.len()`.
    pub fn insert_vertex(&mut self, index: usize, vertex: Vertex) {
        let edge0 = Edge::new(self[index].front(), vertex);
        let edge1 = Edge::new(vertex, self[index].back());
        self.edge_list.remove(index);
        self.edge_list.insert(index, edge1);
        self.edge_list.insert(index, edge0);
    }

    /// Moves all the faces of `other` into `self`, leaving `other` empty.
    #[inline(always)]
    pub fn append(&mut self, other: &mut Wire) { self.edge_list.append(&mut other.edge_list) }

    /// Splits the `Wire` into two at the given index.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// # let v = Vertex::news(7);
    /// # let mut wire = Wire::new();
    /// # for i in 0..6 {
    /// #   wire.push_back(Edge::new(v[i], v[i + 1]));
    /// # }
    /// // let mut wire: Wire = ...;
    /// assert_eq!(wire.len(), 6);
    /// let original_wire = wire.clone();
    /// let mut wire1 = wire.split_off(4);
    /// assert_eq!(wire.len(), 4);
    /// assert_eq!(wire1.len(), 2);
    /// wire.append(&mut wire1);
    /// assert_eq!(original_wire, wire);
    /// ```
    /// # Panics
    /// Panics if `at > self.len()`
    #[inline(always)]
    pub fn split_off(&mut self, at: usize) -> Wire {
        Wire {
            edge_list: self.edge_list.split_off(at),
        }
    }

    /// Inverts the wire.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(4);
    /// let mut wire = Wire::from(vec![
    ///     Edge::new(v[3], v[2]),
    ///     Edge::new(v[2], v[1]),
    ///     Edge::new(v[1], v[0]),
    /// ]);
    /// wire.invert();
    /// for (i, vert) in wire.vertex_iter().enumerate() {
    ///     assert_eq!(v[i], vert);
    /// }
    /// ```
    #[inline(always)]
    pub fn invert(&mut self) -> &mut Self {
        *self = self.inverse();
        self
    }

    /// Returns the inverse wire.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// # let v = Vertex::news(4);
    /// # let mut wire = Wire::from(vec![
    /// #    Edge::new(v[3], v[2]),
    /// #    Edge::new(v[2], v[1]),
    /// #    Edge::new(v[1], v[0]),
    /// # ]);
    /// // let wire: Wire = ...;
    /// let inverse = wire.inverse();
    /// wire.invert();
    /// for (edge0, edge1) in wire.edge_iter().zip(inverse.edge_iter()) {
    ///     assert_eq!(edge0, edge1);
    /// }
    /// ```
    #[inline(always)]
    pub fn inverse(&self) -> Wire {
        let edge_list = self.edge_iter().rev().map(|edge| edge.inverse()).collect();
        Wire { edge_list }
    }

    /// Returns whether all the adjacent pairs of edges have shared vertices or not.
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(4);
    /// let mut wire = Wire::from(vec![
    ///     Edge::new(v[0], v[1]),
    ///     Edge::new(v[2], v[3]),
    /// ]);
    /// assert!(!wire.is_continuous());
    /// wire.insert(1, Edge::new(v[1], v[2]));
    /// assert!(wire.is_continuous());
    /// ```
    pub fn is_continuous(&self) -> bool {
        let mut iter = self.edge_iter();
        if let Some(edge) = iter.next() {
            let mut prev = edge.back();
            for edge in iter {
                if prev != edge.front() {
                    return false;
                }
                prev = edge.back();
            }
        }
        true
    }

    /// Returns whether the front vertex of the wire is the same as the back one or not.
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(4);
    /// let mut wire = Wire::from(vec![
    ///     Edge::new(v[0], v[1]),
    ///     Edge::new(v[2], v[3]),
    /// ]);
    /// assert!(!wire.is_cyclic());
    /// wire.push_back(Edge::new(v[3], v[0]));
    /// assert!(wire.is_cyclic());
    /// ```
    #[inline(always)]
    pub fn is_cyclic(&self) -> bool { self.front_vertex() == self.back_vertex() }

    /// Returns whether the wire is closed or not.
    /// Here, "closed" means "continuous" and "cyclic".
    #[inline(always)]
    pub fn is_closed(&self) -> bool { self.is_continuous() && self.is_cyclic() }

    /// Returns whether simple or not.
    /// Here, "simple" means all the vertices in the wire are shared from only two edges at most.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(4);
    /// let edge0 = Edge::new(v[0], v[1]);
    /// let edge1 = Edge::new(v[1], v[2]);
    /// let edge2 = Edge::new(v[2], v[3]);
    /// let edge3 = Edge::new(v[3], v[1]);
    /// let edge4 = Edge::new(v[3], v[0]);
    ///
    /// let wire0 = Wire::from(vec![edge0, edge1, edge2, edge3]);
    /// let wire1 = Wire::from(vec![edge0, edge1, edge2, edge4]);
    ///
    /// assert!(!wire0.is_simple());
    /// assert!(wire1.is_simple());
    /// ```
    pub fn is_simple(&self) -> bool {
        let mut set = HashSet::new();
        for vertex in self.vertex_iter() {
            if !set.insert(vertex) {
                return false;
            }
        }
        true
    }
}

impl<T> From<T> for Wire
where T: Into<VecDeque<Edge>>
{
    fn from(edge_list: T) -> Wire {
        Wire {
            edge_list: edge_list.into(),
        }
    }
}

impl std::iter::FromIterator<Edge> for Wire {
    fn from_iter<I: IntoIterator<Item = Edge>>(iter: I) -> Wire {
        let edge_list = VecDeque::from_iter(iter);
        Wire::from(edge_list)
    }
}

impl<'a> std::iter::FromIterator<&'a Edge> for Wire {
    fn from_iter<I: IntoIterator<Item = &'a Edge>>(iter: I) -> Wire {
        let edge_list = VecDeque::from_iter(iter.into_iter().map(|edge| *edge));
        Wire::from(edge_list)
    }
}

impl std::iter::IntoIterator for Wire {
    type Item = Edge;
    type IntoIter = std::collections::vec_deque::IntoIter<Edge>;
    fn into_iter(self) -> Self::IntoIter { self.edge_list.into_iter() }
}

pub type EdgeIter<'a> = vec_deque::Iter<'a, Edge>;
pub type EdgeIterMut<'a> = vec_deque::IterMut<'a, Edge>;
pub type EdgeIntoIter = vec_deque::IntoIter<Edge>;

/// An iterator over all the vertices included in a wire.
/// # Details
/// Fundamentally, the iterator runs over all the vertices in a wire.
/// ```
/// # use truck_topology::*;
/// let v = Vertex::news(6);
/// let wire = Wire::from(vec![
///     Edge::new(v[0], v[1]),
///     Edge::new(v[2], v[3]),
///     Edge::new(v[4], v[5]),
/// ]);
/// let mut viter = wire.vertex_iter();
/// assert_eq!(viter.next(), Some(v[0]));
/// assert_eq!(viter.next(), Some(v[1]));
/// assert_eq!(viter.next(), Some(v[2]));
/// assert_eq!(viter.next(), Some(v[3]));
/// assert_eq!(viter.next(), Some(v[4]));
/// assert_eq!(viter.next(), Some(v[5]));
/// assert_eq!(viter.next(), None);
/// assert_eq!(viter.next(), None); // VertexIter is a FusedIterator.
/// ```
/// If a pair of adjacent edges share one vertex, the iterator run only one time at the shared vertex.
/// ```
/// # use truck_topology::*;
/// let v = Vertex::news(6);
/// let wire = Wire::from(vec![
///     Edge::new(v[0], v[1]),
///     Edge::new(v[2], v[3]),
///     Edge::new(v[3], v[4]),
///     Edge::new(v[4], v[5]),
/// ]);
/// let mut viter = wire.vertex_iter();
/// assert_eq!(viter.next(), Some(v[0]));
/// assert_eq!(viter.next(), Some(v[1]));
/// assert_eq!(viter.next(), Some(v[2]));
/// assert_eq!(viter.next(), Some(v[3]));
/// assert_eq!(viter.next(), Some(v[4]));
/// assert_eq!(viter.next(), Some(v[5]));
/// assert_eq!(viter.next(), None);
/// ```
/// If the wire is cyclic, the iterator does not arrive at the last vertex.
/// ```
/// # use truck_topology::*;
/// let v = Vertex::news(5);
/// let wire = Wire::from(vec![
///     Edge::new(v[0], v[1]),
///     Edge::new(v[1], v[2]),
///     Edge::new(v[3], v[4]),
///     Edge::new(v[4], v[0]),
/// ]);
/// let mut viter = wire.vertex_iter();
/// assert_eq!(viter.next(), Some(v[0]));
/// assert_eq!(viter.next(), Some(v[1]));
/// assert_eq!(viter.next(), Some(v[2]));
/// assert_eq!(viter.next(), Some(v[3]));
/// assert_eq!(viter.next(), Some(v[4]));
/// assert_eq!(viter.next(), None);
/// ```
#[derive(Clone, Debug)]
pub struct VertexIter<'a> {
    edge_iter: Peekable<EdgeIter<'a>>,
    unconti_next: Option<Vertex>,
    cyclic: bool,
}

impl<'a> std::iter::Iterator for VertexIter<'a> {
    type Item = Vertex;

    fn next(&mut self) -> Option<Vertex> {
        if self.unconti_next.is_some() {
            let res = self.unconti_next;
            self.unconti_next = None;
            res
        } else if let Some(edge) = self.edge_iter.next() {
            if let Some(next) = self.edge_iter.peek() {
                if edge.back() != next.front() {
                    self.unconti_next = Some(edge.back());
                }
            } else {
                if !self.cyclic {
                    self.unconti_next = Some(edge.back());
                }
            }
            Some(edge.front())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let min_size = self.edge_iter.len();
        let max_size = self.edge_iter.len() * 2;
        (min_size, Some(max_size))
    }

    fn last(self) -> Option<Vertex> {
        let closed = self.cyclic;
        self.edge_iter
            .last()
            .map(|edge| if closed { edge.front() } else { edge.back() })
    }
}

impl<'a> std::iter::FusedIterator for VertexIter<'a> {}

impl Extend<Edge> for Wire {
    fn extend<T: IntoIterator<Item = Edge>>(&mut self, iter: T) {
        for edge in iter {
            self.push_back(edge);
        }
    }
}

impl std::ops::Deref for Wire {
    type Target = VecDeque<Edge>;
    fn deref(&self) -> &VecDeque<Edge> { &self.edge_list }
}

impl std::ops::DerefMut for Wire {
    fn deref_mut(&mut self) -> &mut VecDeque<Edge> { &mut self.edge_list }
}
