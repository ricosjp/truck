use crate::errors::Error;
use crate::{Edge, Result, Vertex, Wire};
use std::collections::vec_deque;
use std::collections::{HashSet, VecDeque};

impl Wire {
    /// Create the empty wire.
    #[inline(always)]
    pub fn new() -> Wire {
        Wire {
            edge_list: VecDeque::new(),
        }
    }
    /// Create the empty wire with space for at least `capacity` edges.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Wire {
        Wire {
            edge_list: VecDeque::with_capacity(capacity),
        }
    }

    /// Provides a reference to the edge at the given index.
    /// Element at index 0 is the front of the queue.
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&Edge> { self.edge_list.get(index) }

    #[inline(always)]
    pub fn capacity(&self) -> usize { self.edge_list.capacity() }

    #[inline(always)]
    pub fn reserve_exact(&mut self, additional: usize) { self.edge_list.reserve_exact(additional) }
    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) { self.edge_list.reserve(additional) }

    #[inline(always)]
    pub fn shrink_to_fit(&mut self) { self.edge_list.shrink_to_fit() }

    #[inline(always)]
    pub fn truncate(&mut self, len: usize) { self.edge_list.truncate(len) }
    #[inline(always)]
    pub fn edge_iter(&self) -> EdgeIter { self.edge_list.iter() }

    /// get the vertex iterator
    /// # Exapmles
    /// ```
    /// use truck_topology::{Vertex, Edge, Wire};
    /// let v = Vertex::news(4);
    /// let mut wire = Wire::new();
    /// for i in 0..2 {
    ///     wire.push_back(Edge::new(v[i], v[i + 1]));
    /// }
    ///
    /// let mut iter = wire.vertex_iter();
    /// for i in 0..3 {
    ///     assert_eq!(iter.next().unwrap(), v[i]);
    /// }
    /// assert!(iter.next().is_none());
    /// ```
    #[inline(always)]
    pub fn vertex_iter(&self) -> VertexIter {
        VertexIter {
            edge_iter: self.edge_iter(),
            next: None,
            closed: self.is_closed(),
        }
    }

    #[inline(always)]
    pub fn as_slices(&self) -> (&[Edge], &[Edge]) { self.edge_list.as_slices() }

    /// the number of the edges in wire.
    #[inline(always)]
    pub fn len(&self) -> usize { self.edge_list.len() }
    /// whether empty or not
    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.edge_list.is_empty() }

    #[inline(always)]
    pub fn clear(&mut self) { self.edge_list.clear() }

    #[inline(always)]
    pub fn contains(&self, edge: &Edge) -> bool { self.edge_list.contains(edge) }
    /// get the front edge. If `self` is empty wire, return None.
    #[inline(always)]
    pub fn front_edge(&self) -> Option<Edge> { self.edge_list.front().map(|edge| *edge) }

    /// get the front vertex. If `self` is empty wire, return None.
    #[inline(always)]
    pub fn front_vertex(&self) -> Option<Vertex> { self.front_edge().map(|edge| edge.front()) }

    /// get the back edge. If `self` is empty wire, return None.
    #[inline(always)]
    pub fn back_edge(&self) -> Option<Edge> { self.edge_list.back().map(|edge| *edge) }

    /// get the back edge. If `self` is empty wire, return None.
    #[inline(always)]
    pub fn back_vertex(&self) -> Option<Vertex> { self.back_edge().map(|edge| edge.back()) }

    #[inline(always)]
    pub fn pop_front(&mut self) -> Option<Edge> { self.edge_list.pop_front() }

    #[inline(always)]
    pub fn pop_back(&mut self) -> Option<Edge> { self.edge_list.pop_back() }
    /// push front the edge.
    /// # Panic
    /// The front vertex of `self` must be the same as the back one of `edge`.
    pub fn push_front(&mut self, edge: Edge) {
        match self.try_push_front(edge) {
            Ok(()) => {}
            Err(error) => panic!("{}", error),
        }
    }

    /// push front the edge.
    /// # Failure
    /// The front vertex of `self` must be the same as the back one of `edge`.
    pub fn try_push_front(&mut self, edge: Edge) -> Result<()> {
        if self.is_empty() || self.front_vertex() == Some(edge.back()) {
            Ok(self.push_front_unchecked(edge))
        } else {
            Err(Error::CannotAddEdge)
        }
    }

    /// push front the edge.
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.
    /// This method does NOT check whether the front vertex of `self` is the same as the back one of `edge`.
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn push_front_unchecked(&mut self, edge: Edge) { self.edge_list.push_front(edge) }

    /// push back the edge.
    /// # Panic
    /// The back vertex of `self` must be the same as the front one of `edge`.
    pub fn push_back(&mut self, edge: Edge) {
        match self.try_push_back(edge) {
            Ok(()) => {}
            Err(error) => panic!("{}", error),
        }
    }

    /// push back the edge.
    /// # Failure
    /// The back vertex of `self` must be the same as the front one of `edge`.
    pub fn try_push_back(&mut self, edge: Edge) -> Result<()> {
        if self.is_empty() || self.back_vertex() == Some(edge.front()) {
            Ok(self.push_back_unchecked(edge))
        } else {
            Err(Error::CannotAddEdge)
        }
    }

    /// push back the edge.
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.
    /// This method does NOT check whether the back vertex of `self` is the same as the front one of `edge`.
    /// The programmer must guarantee this condition before using this method.
    pub fn push_back_unchecked(&mut self, edge: Edge) { self.edge_list.push_back(edge) }

    pub fn try_insert_edge(&mut self, index: usize, edge: Edge) -> Result<()> {
        if index == 0 {
            self.try_push_front(edge)
        } else if index + 1 >= self.len() {
            self.try_push_back(edge)
        } else {
            let front = self[index - 1].back() == edge.front();
            let back = self[index].front() == edge.back();
            if front && back {
                self.insert_unchecked(index, edge);
                Ok(())
            } else {
                Err(Error::CannotAddEdge)
            }
        }
    }
    pub fn insert_edge(&mut self, index: usize, edge: Edge) {
        match self.try_insert_edge(index, edge) {
            Ok(()) => {}
            Err(error) => panic!("{}", error),
        }
    }
    #[inline(always)]
    pub fn insert_unchecked(&mut self, index: usize, edge: Edge) {
        self.edge_list.insert(index, edge)
    }

    pub fn insert_vertex(&mut self, index: usize, vertex: Vertex) {
        let edge0 = Edge::new(self[index].front(), vertex);
        let edge1 = Edge::new(vertex, self[index].back());
        self.edge_list.remove(index);
        self.edge_list.insert(index, edge1);
        self.edge_list.insert(index, edge0);
    }
    pub fn split_off(&mut self, at: usize) -> Wire {
        Wire {
            edge_list: self.edge_list.split_off(at),
        }
    }

    /// append two wires.
    /// # Panic
    /// The back vertex of `self` must be the same as the front one of `wire`.
    pub fn append(&mut self, wire: &mut Wire) {
        match self.try_append(wire) {
            Ok(()) => {}
            Err(error) => panic!("{}", error),
        }
    }

    /// append two wires.
    /// # Failure
    /// The back vertex of `self` must be the same as the front one of `wire`.
    pub fn try_append(&mut self, wire: &mut Wire) -> Result<()> {
        if self.is_empty() || self.back_vertex() == wire.front_vertex() {
            Ok(self.append_unchecked(wire))
        } else {
            Err(Error::CannotAddEdge)
        }
    }

    /// append two wires.
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.
    /// This method does NOT check whether the back vertex of `self` is the same as the front one of `wire`.
    /// The programmer must guarantee this condition before using this method.
    pub fn append_unchecked(&mut self, wire: &mut Wire) {
        self.edge_list.append(&mut wire.edge_list)
    }

    pub fn resize_with(&mut self, new_len: usize, mut generator: impl FnMut() -> Edge) {
        for _ in 0..new_len {
            self.push_back(generator())
        }
    }

    pub fn rotate_left(&mut self, mid: usize) -> Result<()> {
        if self.is_closed() {
            self.edge_list.rotate_left(mid);
            Ok(())
        } else {
            Err(Error::NotClosedWire)
        }
    }

    pub fn rotate_right(&mut self, mid: usize) -> Result<()> {
        if self.is_closed() {
            self.edge_list.rotate_right(mid);
            Ok(())
        } else {
            Err(Error::NotClosedWire)
        }
    }

    #[inline(always)]
    pub fn by_slice(arr: &[Edge]) -> Wire {
        let mut wire = Wire::new();
        for edge in arr {
            wire.push_back(*edge);
        }
        wire
    }

    #[inline(always)]
    pub fn try_by_slice(arr: &[Edge]) -> Result<Wire> {
        let mut wire = Wire::new();
        for edge in arr {
            wire.try_push_back(*edge)?;
        }
        Ok(wire)
    }

    /// whether closed or not. i.e. whether the front vertex and the back vertex are same.
    #[inline(always)]
    pub fn is_closed(&self) -> bool {
        !self.is_empty() && self.front_vertex() == self.back_vertex()
    }

    #[inline(always)]
    pub fn inverse(&mut self) -> &mut Self {
        let new_edges: VecDeque<Edge> = self.edge_iter().rev().map(|edge| edge.inverse()).collect();
        self.edge_list = new_edges;
        self
    }

    /// whether simple or not. i.e. wheter there exists duplicate verticies in wire except for the front
    /// and the end.
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
    /// let wire0 = Wire::by_slice(&[edge0, edge1, edge2, edge3]);
    /// let wire1 = Wire::by_slice(&[edge0, edge1, edge2, edge4]);
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

    pub fn replace(&self, search: Vertex, replace: Vertex) -> Wire {
        let mut edge_list = VecDeque::<Edge>::new();
        for edge in self.edge_iter() {
            if search == edge.front() {
                if edge.back() != search && edge.back() != replace {
                    edge_list.push_back(Edge::new(replace, edge.back()));
                }
            } else if search == edge.back() {
                if edge.front() != replace {
                    edge_list.push_back(Edge::new(edge.front(), replace));
                }
            }
        }
        Wire { edge_list }
    }
}

impl std::ops::Index<usize> for Wire {
    type Output = Edge;

    fn index(&self, idx: usize) -> &Edge { &self.edge_list[idx] }
}

impl std::convert::TryFrom<Vec<Edge>> for Wire {
    type Error = Error;

    fn try_from(edges: Vec<Edge>) -> Result<Wire> {
        for pairs in edges.windows(2) {
            if pairs[0].back() != pairs[1].front() {
                return Err(Error::CannotAddEdge);
            }
        }
        Ok(Wire {
            edge_list: edges.into(),
        })
    }
}

impl std::convert::TryFrom<VecDeque<Edge>> for Wire {
    type Error = Error;

    fn try_from(edge_list: VecDeque<Edge>) -> Result<Wire> {
        if edge_list.is_empty() {
            return Ok(Wire::new());
        }
        let mut edge_iter = edge_list.iter();
        let mut prev = edge_iter.next().unwrap();
        for edge in edge_iter {
            if prev.back() != edge.front() {
                return Err(Error::CannotAddEdge);
            }
            prev = edge;
        }
        Ok(Wire { edge_list })
    }
}

impl std::iter::FromIterator<Edge> for Wire {
    fn from_iter<I: IntoIterator<Item = Edge>>(iter: I) -> Wire {
        use std::convert::TryFrom;
        let edge_list = VecDeque::from_iter(iter);
        match Wire::try_from(edge_list) {
            Ok(wire) => wire,
            Err(error) => panic!("{}", error),
        }
    }
}

impl std::iter::IntoIterator for Wire {
    type Item = Edge;
    type IntoIter = std::collections::vec_deque::IntoIter<Edge>;
    fn into_iter(self) -> Self::IntoIter { self.edge_list.into_iter() }
}

pub type EdgeIter<'a> = vec_deque::Iter<'a, Edge>;

pub struct VertexIter<'a> {
    edge_iter: EdgeIter<'a>,
    next: Option<Vertex>,
    closed: bool,
}

impl<'a> std::iter::Iterator for VertexIter<'a> {
    type Item = Vertex;

    fn next(&mut self) -> Option<Vertex> {
        if let Some(edge) = self.edge_iter.next() {
            self.next = Some(edge.back());
            Some(edge.front())
        } else {
            if self.closed || self.next.is_none() {
                None
            } else {
                let res = self.next.unwrap();
                self.next = None;
                Some(res)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = if self.closed {
            self.edge_iter.len()
        } else {
            self.edge_iter.len() + 1
        };
        (size, Some(size))
    }

    fn last(self) -> Option<Vertex> { self.edge_iter.last().map(|edge| edge.back()) }
}

impl Extend<Edge> for Wire {
    fn extend<T: IntoIterator<Item = Edge>>(&mut self, iter: T) {
        for edge in iter {
            self.push_back(edge);
        }
    }
}
