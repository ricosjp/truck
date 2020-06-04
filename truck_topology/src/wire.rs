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

    /// the number of the edges in wire.
    #[inline(always)]
    pub fn len(&self) -> usize { self.edge_list.len() }

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

    /// whether empty or not
    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.edge_list.is_empty() }

    /// whether closed or not. i.e. whether the front vertex and the back vertex are same.
    #[inline(always)]
    pub fn is_closed(&self) -> bool {
        !self.is_empty() && self.front_vertex() == self.back_vertex()
    }

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

    /// push front the edge.
    /// # Panic
    /// The front vertex of `self` must be the same as the back one of `edge`.
    pub fn push_front(&mut self, edge: Edge) {
        if self.is_empty() || self.front_vertex() == Some(edge.back()) {
            self.edge_list.push_front(edge)
        } else {
            panic!("{}", Error::CannotAddEdge);
        }
    }

    /// push front the edge.
    /// # Failure
    /// The front vertex of `self` must be the same as the back one of `edge`.
    pub fn try_push_front(&mut self, edge: Edge) -> Result<()> {
        if self.is_empty() || self.front_vertex() == Some(edge.back()) {
            Ok(self.edge_list.push_front(edge))
        } else {
            Err(Error::CannotAddEdge)
        }
    }

    /// push front the edge.
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.
    /// This method does NOT check whether the front vertex of `self` is the same as the back one of `edge`.
    /// The programmer must guarantee this condition before using this method.
    pub fn push_front_unchecked(&mut self, edge: Edge) { self.edge_list.push_front(edge) }

    /// push back the edge.
    /// # Panic
    /// The back vertex of `self` must be the same as the front one of `edge`.
    pub fn push_back(&mut self, edge: Edge) {
        if self.is_empty() || self.back_vertex() == Some(edge.front()) {
            self.edge_list.push_back(edge);
        } else {
            panic!("{}", Error::CannotAddEdge)
        }
    }

    /// push back the edge.
    /// # Failure
    /// The back vertex of `self` must be the same as the front one of `edge`.
    pub fn try_push_back(&mut self, edge: Edge) -> Result<()> {
        if self.is_empty() || self.back_vertex() == Some(edge.front()) {
            Ok(self.edge_list.push_back(edge))
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

    /// append two wires.
    /// # Panic
    /// The back vertex of `self` must be the same as the front one of `wire`.
    pub fn append(&mut self, wire: &mut Wire) {
        if self.is_empty() || self.back_vertex() == wire.front_vertex() {
            self.edge_list.append(&mut wire.edge_list)
        } else {
            panic!("{}", Error::CannotAddEdge)
        }
    }

    /// append two wires.
    /// # Failure
    /// The back vertex of `self` must be the same as the front one of `wire`.
    pub fn try_append(&mut self, wire: &mut Wire) -> Result<()> {
        if self.is_empty() || self.back_vertex() == wire.front_vertex() {
            Ok(self.edge_list.append(&mut wire.edge_list))
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
            top: self.edge_list.front().map(|edge| edge.front()),
        }
    }

    #[inline(always)]
    pub fn inverse(&mut self) -> &mut Self {
        let new_edges: VecDeque<Edge> = self
            .edge_iter()
            .rev()
            .map(|edge| edge.inverse())
            .collect();
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
        if self.is_closed() {
            for edge in self.edge_iter() {
                if !set.insert(edge.front()) {
                    return false;
                }
            }
        } else {
            for vertex in self.vertex_iter() {
                if !set.insert(vertex) {
                    return false;
                }
            }
        }
        true
    }
}

impl std::ops::Index<usize> for Wire {
    type Output = Edge;

    fn index(&self, idx: usize) -> &Edge { &self.edge_list[idx] }
}

impl std::convert::TryFrom<Vec<Edge>> for Wire {
    type Error = crate::errors::Error;

    fn try_from(edges: Vec<Edge>) -> Result<Wire> {
        let mut wire = Wire::new();
        for edge in edges.into_iter() {
            wire.try_push_back(edge)?;
        }
        Ok(wire)
    }
}

impl std::iter::FromIterator<Edge> for Wire {
    fn from_iter<I: IntoIterator<Item = Edge>>(iter: I) -> Wire {
        let mut wire = Wire::new();
        for edge in iter {
            wire.push_back(edge);
        }
        wire
    }
}

type EdgeIter<'a> = vec_deque::Iter<'a, Edge>;

pub struct VertexIter<'a> {
    edge_iter: EdgeIter<'a>,
    top: Option<Vertex>,
}

impl<'a> std::iter::Iterator for VertexIter<'a> {
    type Item = Vertex;

    fn next(&mut self) -> Option<Vertex> {
        match self.top {
            Some(vertex) => {
                self.top = None;
                Some(vertex)
            }
            None => self.edge_iter.next().map(|edge| edge.back()),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = if self.top.is_some() {
            self.edge_iter.len() + 1
        } else {
            self.edge_iter.len()
        };
        (size, Some(size))
    }

    fn last(self) -> Option<Vertex> { self.edge_iter.last().map(|edge| edge.back()) }
}
