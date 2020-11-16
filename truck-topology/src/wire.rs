use crate::*;
use std::collections::vec_deque;
use std::collections::{HashSet, VecDeque};
use std::iter::Peekable;

impl<P, C> Wire<P, C> {
    /// Creates the empty wire.
    #[inline(always)]
    pub fn new() -> Wire<P, C> {
        Wire {
            edge_list: VecDeque::new(),
        }
    }
    /// Creates the empty wire with space for at least `capacity` edges.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Wire<P, C> {
        Wire {
            edge_list: VecDeque::with_capacity(capacity),
        }
    }

    /// Returns an iterator over the edges. Practically, an alias of `iter()`.
    #[inline(always)]
    pub fn edge_iter(&self) -> EdgeIter<P, C> { self.iter() }
    /// Returns a mutable iterator over the edges. Practically, an alias of `iter_mut()`.
    #[inline(always)]
    pub fn edge_iter_mut(&mut self) -> EdgeIterMut<P, C> { self.iter_mut() }

    /// Creates a consuming iterator. Practically, an alias of `into_iter()`.
    #[inline(always)]
    pub fn edge_into_iter(self) -> EdgeIntoIter<P, C> { self.into_iter() }

    /// Returns an iterator over the vertices.
    #[inline(always)]
    pub fn vertex_iter(&self) -> VertexIter<P, C> {
        VertexIter {
            edge_iter: self.edge_iter().peekable(),
            unconti_next: None,
            cyclic: self.is_cyclic(),
        }
    }

    /// Returns the front edge. If `self` is empty wire, returns None.  
    /// Practically, an alias of the inherited method `VecDeque::front()`.
    #[inline(always)]
    pub fn front_edge(&self) -> Option<&Edge<P, C>> { self.front() }

    /// Returns the front vertex. If `self` is empty wire, returns None.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(&[(), (), ()]);
    /// let mut wire = Wire::new();
    /// assert_eq!(wire.front_vertex(), None);
    /// wire.push_back(Edge::new(&v[1], &v[2], ()));
    /// wire.push_front(Edge::new(&v[0], &v[1], ()));
    /// assert_eq!(wire.front_vertex(), Some(&v[0]));
    /// ```
    #[inline(always)]
    pub fn front_vertex(&self) -> Option<&Vertex<P>> { self.front().map(|edge| edge.front()) }

    /// Returns the back edge. If `self` is empty wire, returns None.  
    /// Practically, an alias of the inherited method `VecDeque::back()`
    #[inline(always)]
    pub fn back_edge(&self) -> Option<&Edge<P, C>> { self.back() }

    /// Returns the back edge. If `self` is empty wire, returns None.
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(&[(), (), ()]);
    /// let mut wire = Wire::new();
    /// assert_eq!(wire.back_vertex(), None);
    /// wire.push_back(Edge::new(&v[1], &v[2], ()));
    /// wire.push_front(Edge::new(&v[0], &v[1], ()));
    /// assert_eq!(wire.back_vertex(), Some(&v[2]));
    /// ```
    #[inline(always)]
    pub fn back_vertex(&self) -> Option<&Vertex<P>> { self.back().map(|edge| edge.back()) }

    /// Returns vertices at both ends.
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 3]);
    /// let mut wire = Wire::new();
    /// assert_eq!(wire.back_vertex(), None);
    /// wire.push_back(Edge::new(&v[1], &v[2], ()));
    /// wire.push_front(Edge::new(&v[0], &v[1], ()));
    /// assert_eq!(wire.ends_vertices(), Some((&v[0], &v[2])));
    /// ```
    #[inline(always)]
    pub fn ends_vertices(&self) -> Option<(&Vertex<P>, &Vertex<P>)> {
        match (self.front_vertex(), self.back_vertex()) {
            (Some(got0), Some(got1)) => Some((got0, got1)),
            _ => None,
        }
    }

    /// Moves all the faces of `other` into `self`, leaving `other` empty.
    #[inline(always)]
    pub fn append(&mut self, other: &mut Wire<P, C>) { self.edge_list.append(&mut other.edge_list) }

    /// Splits the `Wire` into two at the given index.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 7]);
    /// let mut wire = Wire::new();
    /// for i in 0..6 {
    ///    wire.push_back(Edge::new(&v[i], &v[i + 1], ()));
    /// }
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
    pub fn split_off(&mut self, at: usize) -> Wire<P, C> {
        Wire {
            edge_list: self.edge_list.split_off(at),
        }
    }

    /// Inverts the wire.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 4]);
    /// let mut wire = Wire::from(vec![
    ///     Edge::new(&v[3], &v[2], ()),
    ///     Edge::new(&v[2], &v[1], ()),
    ///     Edge::new(&v[1], &v[0], ()),
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
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 4]);
    /// let mut wire = Wire::from(vec![
    ///     Edge::new(&v[3], &v[2], ()),
    ///     Edge::new(&v[2], &v[1], ()),
    ///     Edge::new(&v[1], &v[0], ()),
    /// ]);
    /// let inverse = wire.inverse();
    /// wire.invert();
    /// for (edge0, edge1) in wire.edge_iter().zip(inverse.edge_iter()) {
    ///     assert_eq!(edge0, edge1);
    /// }
    /// ```
    #[inline(always)]
    pub fn inverse(&self) -> Wire<P, C> {
        let edge_list = self.edge_iter().rev().map(|edge| edge.inverse()).collect();
        Wire { edge_list }
    }

    /// Returns whether all the adjacent pairs of edges have shared vertices or not.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 4]);
    /// let mut wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[2], &v[3], ()),
    /// ]);
    /// assert!(!wire.is_continuous());
    /// wire.insert(1, Edge::new(&v[1], &v[2], ()));
    /// assert!(wire.is_continuous());
    /// ```
    /// ```
    /// use truck_topology::*;
    /// // The empty wire is continuous
    /// assert!(Wire::<(), ()>::new().is_continuous());
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
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 4]);
    /// let mut wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[2], &v[3], ()),
    /// ]);
    /// assert!(!wire.is_cyclic());
    /// wire.push_back(Edge::new(&v[3], &v[0], ()));
    /// assert!(wire.is_cyclic());
    /// ```
    /// ```
    /// use truck_topology::*;
    /// // The empty wire is cyclic.
    /// assert!(Wire::<(), ()>::new().is_cyclic());
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
    /// use std::iter::FromIterator;
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 4]);
    /// let edge0 = Edge::new(&v[0], &v[1], ());
    /// let edge1 = Edge::new(&v[1], &v[2], ());
    /// let edge2 = Edge::new(&v[2], &v[3], ());
    /// let edge3 = Edge::new(&v[3], &v[1], ());
    /// let edge4 = Edge::new(&v[3], &v[0], ());
    ///
    /// let wire0 = Wire::from_iter(vec![&edge0, &edge1, &edge2, &edge3]);
    /// let wire1 = Wire::from(vec![edge0, edge1, edge2, edge4]);
    ///
    /// assert!(!wire0.is_simple());
    /// assert!(wire1.is_simple());
    /// ```
    /// ```
    /// use truck_topology::*;
    /// // The empty wire is simple.
    /// assert!(Wire::<(), ()>::new().is_simple());
    /// ```
    pub fn is_simple(&self) -> bool {
        let mut set = HashSet::new();
        for vertex in self.vertex_iter() {
            if !set.insert(vertex.id()) {
                return false;
            }
        }
        true
    }

    /// Determines whether all the wires in `wires` has no same vertices.
    pub fn disjoint_wires(wires: &Vec<Wire<P, C>>) -> bool {
        let mut set = HashSet::new();
        for vertex in wires.iter().flat_map(|wire| wire.vertex_iter()) {
            if set.get(&vertex.id()).is_some() {
                return false;
            }
        }
        for vertex in wires.iter().flat_map(|wire| wire.vertex_iter()) {
            set.insert(vertex.id());
        }
        true
    }
}

impl<P: Tolerance, C: Curve<Point=P>> Wire<P, C> {
    /// Returns the consistence of the geometry of end vertices
    /// and the geometry of edge.
    #[inline(always)]
    pub fn is_geometric_consistent(&self) -> bool {
        self.iter().all(|edge| edge.is_geometric_consistent())
    }
}

impl<T, P, C> From<T> for Wire<P, C>
where T: Into<VecDeque<Edge<P, C>>>
{
    #[inline(always)]
    fn from(edge_list: T) -> Wire<P, C> {
        Wire {
            edge_list: edge_list.into(),
        }
    }
}

impl<P, C> std::iter::FromIterator<Edge<P, C>> for Wire<P, C> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = Edge<P, C>>>(iter: I) -> Wire<P, C> {
        let edge_list = VecDeque::from_iter(iter);
        Wire::from(edge_list)
    }
}

impl<'a, P, C> std::iter::FromIterator<&'a Edge<P, C>> for Wire<P, C> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = &'a Edge<P, C>>>(iter: I) -> Wire<P, C> {
        let edge_list = VecDeque::from_iter(iter.into_iter().map(|edge| edge.clone()));
        Wire::from(edge_list)
    }
}

impl<P, C> IntoIterator for Wire<P, C> {
    type Item = Edge<P, C>;
    type IntoIter = std::collections::vec_deque::IntoIter<Edge<P, C>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.edge_list.into_iter() }
}

impl<'a, P, C> IntoIterator for &'a Wire<P, C> {
    type Item = &'a Edge<P, C>;
    type IntoIter = std::collections::vec_deque::Iter<'a, Edge<P, C>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.edge_list.iter() }
}

/// The reference iterator over all edges in a wire.
pub type EdgeIter<'a, P, C> = vec_deque::Iter<'a, Edge<P, C>>;
/// The mutable reference iterator over all edges in a wire.
pub type EdgeIterMut<'a, P, C> = vec_deque::IterMut<'a, Edge<P, C>>;
/// The into iterator over all edges in a wire.
pub type EdgeIntoIter<P, C> = vec_deque::IntoIter<Edge<P, C>>;

/// The iterator over all the vertices included in a wire.
/// # Details
/// Fundamentally, the iterator runs over all the vertices in a wire.
/// ```
/// use truck_topology::*;
/// let v = Vertex::news(&[(); 6]);
/// let wire = Wire::from(vec![
///     Edge::new(&v[0], &v[1], ()),
///     Edge::new(&v[2], &v[3], ()),
///     Edge::new(&v[4], &v[5], ()),
/// ]);
/// let mut viter = wire.vertex_iter();
/// assert_eq!(viter.next().as_ref(), Some(&v[0]));
/// assert_eq!(viter.next().as_ref(), Some(&v[1]));
/// assert_eq!(viter.next().as_ref(), Some(&v[2]));
/// assert_eq!(viter.next().as_ref(), Some(&v[3]));
/// assert_eq!(viter.next().as_ref(), Some(&v[4]));
/// assert_eq!(viter.next().as_ref(), Some(&v[5]));
/// assert_eq!(viter.next(), None);
/// assert_eq!(viter.next(), None); // VertexIter is a FusedIterator.
/// ```
/// If a pair of adjacent edges share one vertex, the iterator run only one time at the shared vertex.
/// ```
/// use truck_topology::*;
/// let v = Vertex::news(&[(); 6]);
/// let wire = Wire::from(vec![
///     Edge::new(&v[0], &v[1], ()),
///     Edge::new(&v[2], &v[3], ()),
///     Edge::new(&v[3], &v[4], ()),
///     Edge::new(&v[4], &v[5], ()),
/// ]);
/// let mut viter = wire.vertex_iter();
/// assert_eq!(viter.next().as_ref(), Some(&v[0]));
/// assert_eq!(viter.next().as_ref(), Some(&v[1]));
/// assert_eq!(viter.next().as_ref(), Some(&v[2]));
/// assert_eq!(viter.next().as_ref(), Some(&v[3]));
/// assert_eq!(viter.next().as_ref(), Some(&v[4]));
/// assert_eq!(viter.next().as_ref(), Some(&v[5]));
/// assert_eq!(viter.next(), None);
/// ```
/// If the wire is cyclic, the iterator does not arrive at the last vertex.
/// ```
/// # use truck_topology::*;
/// let v = Vertex::news(&[(); 5]);
/// let wire = Wire::from(vec![
///     Edge::new(&v[0], &v[1], ()),
///     Edge::new(&v[1], &v[2], ()),
///     Edge::new(&v[3], &v[4], ()),
///     Edge::new(&v[4], &v[0], ()),
/// ]);
/// let mut viter = wire.vertex_iter();
/// assert_eq!(viter.next().as_ref(), Some(&v[0]));
/// assert_eq!(viter.next().as_ref(), Some(&v[1]));
/// assert_eq!(viter.next().as_ref(), Some(&v[2]));
/// assert_eq!(viter.next().as_ref(), Some(&v[3]));
/// assert_eq!(viter.next().as_ref(), Some(&v[4]));
/// assert_eq!(viter.next(), None);
/// ```
#[derive(Clone, Debug)]
pub struct VertexIter<'a, P, C> {
    edge_iter: Peekable<EdgeIter<'a, P, C>>,
    unconti_next: Option<Vertex<P>>,
    cyclic: bool,
}

impl<'a, P, C> Iterator for VertexIter<'a, P, C> {
    type Item = Vertex<P>;

    fn next(&mut self) -> Option<Vertex<P>> {
        if self.unconti_next.is_some() {
            let res = self.unconti_next.clone();
            self.unconti_next = None;
            res
        } else if let Some(edge) = self.edge_iter.next() {
            if let Some(next) = self.edge_iter.peek() {
                if edge.back() != next.front() {
                    self.unconti_next = Some(edge.back().clone());
                }
            } else {
                if !self.cyclic {
                    self.unconti_next = Some(edge.back().clone());
                }
            }
            Some(edge.front().clone())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let min_size = self.edge_iter.len();
        let max_size = self.edge_iter.len() * 2;
        (min_size, Some(max_size))
    }

    fn last(self) -> Option<Vertex<P>> {
        let closed = self.cyclic;
        self.edge_iter.last().map(|edge| {
            if closed {
                edge.front().clone()
            } else {
                edge.back().clone()
            }
        })
    }
}

impl<'a, P, C> std::iter::FusedIterator for VertexIter<'a, P, C> {}

impl<P, C> Extend<Edge<P, C>> for Wire<P, C> {
    fn extend<T: IntoIterator<Item = Edge<P, C>>>(&mut self, iter: T) {
        for edge in iter {
            self.push_back(edge);
        }
    }
}

impl<P, C> std::ops::Deref for Wire<P, C> {
    type Target = VecDeque<Edge<P, C>>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.edge_list }
}

impl<P, C> std::ops::DerefMut for Wire<P, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.edge_list }
}

impl<P, C> Clone for Wire<P, C> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            edge_list: self.edge_list.clone(),
        }
    }
}

impl<P, C> PartialEq for Wire<P, C> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool { self.edge_list == other.edge_list }
}

impl<P, C> Eq for Wire<P, C> {}
