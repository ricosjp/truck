use crate::*;
use rustc_hash::FxHashSet as HashSet;
use std::collections::vec_deque;
use std::collections::VecDeque;
use std::iter::Peekable;
use truck_base::entry_map::FxEntryMap as EntryMap;

impl<P, C> Wire<P, C> {
    /// Creates the empty wire.
    #[inline(always)]
    pub fn new() -> Wire<P, C> { Self::default() }

    /// Creates the empty wire with space for at least `capacity` edges.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Wire<P, C> {
        Wire {
            edge_list: VecDeque::with_capacity(capacity),
        }
    }

    /// Returns an iterator over the edges. Practically, an alias of `iter()`.
    #[inline(always)]
    pub fn edge_iter(&self) -> EdgeIter<'_, P, C> { self.iter() }
    /// Returns a mutable iterator over the edges. Practically, an alias of `iter_mut()`.
    #[inline(always)]
    pub fn edge_iter_mut(&mut self) -> EdgeIterMut<'_, P, C> { self.iter_mut() }

    /// Creates a consuming iterator. Practically, an alias of `into_iter()`.
    #[inline(always)]
    pub fn edge_into_iter(self) -> EdgeIntoIter<P, C> { self.into_iter() }

    /// Returns an iterator over the vertices.
    #[inline(always)]
    pub fn vertex_iter(&self) -> VertexIter<'_, P, C> {
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
    /// use truck_topology::*;
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
    /// use truck_topology::*;
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
        let mut set = HashSet::default();
        self.vertex_iter()
            .all(move |vertex| set.insert(vertex.id()))
    }

    /// Determines whether all the wires in `wires` has no same vertices.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    ///
    /// let v = Vertex::news(&[(), (), (), (), ()]);
    /// let edge0 = Edge::new(&v[0], &v[1], ());
    /// let edge1 = Edge::new(&v[1], &v[2], ());
    /// let edge2 = Edge::new(&v[2], &v[3], ());
    /// let edge3 = Edge::new(&v[3], &v[4], ());
    ///
    /// let wire0 = Wire::from(vec![edge0, edge1]);
    /// let wire1 = Wire::from(vec![edge2]);
    /// let wire2 = Wire::from(vec![edge3]);
    ///
    /// assert!(Wire::disjoint_wires(&[wire0.clone(), wire2]));
    /// assert!(!Wire::disjoint_wires(&[wire0, wire1]));
    /// ```
    pub fn disjoint_wires(wires: &[Wire<P, C>]) -> bool {
        let mut set = HashSet::default();
        wires.iter().all(move |wire| {
            let mut vec = Vec::new();
            let res = wire.vertex_iter().all(|v| {
                vec.push(v.id());
                !set.contains(&v.id())
            });
            set.extend(vec);
            res
        })
    }

    /// Swap one edge into two edges.
    ///
    /// # Arguments
    /// - `idx`: Index of edge in wire
    /// - `edges`: Inserted edges
    ///
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), (), (), ()]);
    /// let edge0 = Edge::new(&v[0], &v[1], 0);
    /// let edge1 = Edge::new(&v[1], &v[3], 1);
    /// let edge2 = Edge::new(&v[3], &v[4], 2);
    /// let edge3 = Edge::new(&v[1], &v[2], 3);
    /// let edge4 = Edge::new(&v[2], &v[3], 4);
    /// let mut wire0 = Wire::from(vec![
    ///     edge0.clone(), edge1, edge2.clone()
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///     edge0, edge3.clone(), edge4.clone(), edge2
    /// ]);
    /// assert_ne!(wire0, wire1);
    /// wire0.swap_edge_into_wire(1, Wire::from(vec![edge3, edge4]));
    /// assert_eq!(wire0, wire1);
    /// ```
    ///
    /// # Panics
    /// Panic occars if `idx >= self.len()`.
    ///
    /// # Failure
    /// Returns `false` and `self` will not be changed if the end vertices of `self[idx]` and the ones of `wire` is not the same.
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), (), (), ()]);
    /// let edge0 = Edge::new(&v[0], &v[1], 0);
    /// let edge1 = Edge::new(&v[1], &v[3], 1);
    /// let edge2 = Edge::new(&v[3], &v[4], 2);
    /// let edge3 = Edge::new(&v[1], &v[2], 3);
    /// let edge4 = Edge::new(&v[2], &v[1], 4);
    /// let mut wire0 = Wire::from(vec![
    ///     edge0.clone(), edge1, edge2.clone()
    /// ]);
    /// let backup = wire0.clone();
    /// // The end vertices of wire[1] == edge1 is (v[1], v[3]).
    /// // The end points of new wire [edge3, edge4] is (v[1], v[1]).
    /// // Since the back vertices are different, returns false and do nothing.
    /// assert!(!wire0.swap_edge_into_wire(1, Wire::from(vec![edge3, edge4])));
    /// assert_eq!(wire0, backup);
    /// ```
    pub fn swap_edge_into_wire(&mut self, idx: usize, wire: Wire<P, C>) -> bool {
        if wire.is_empty() || self[idx].ends() != wire.ends_vertices().unwrap() {
            return false;
        }
        let mut new_wire: Vec<_> = self.drain(0..idx).collect();
        new_wire.extend(wire);
        self.pop_front();
        new_wire.extend(self.drain(..));
        *self = new_wire.into();
        true
    }
    /// Concat edges
    pub(super) fn swap_subwire_into_edges(&mut self, mut idx: usize, edge: Edge<P, C>) {
        if idx + 1 == self.len() {
            self.rotate_left(1);
            idx -= 1;
        }
        let mut new_wire: Vec<_> = self.drain(0..idx).collect();
        new_wire.push(edge);
        self.pop_front();
        self.pop_front();
        new_wire.extend(self.drain(..));
        *self = new_wire.into();
    }

    pub(super) fn sub_try_mapped<'a, Q, D, KF, KV>(
        &'a self,
        edge_map: &mut EntryMap<EdgeID<C>, Option<Edge<Q, D>>, KF, KV, &'a Edge<P, C>>,
    ) -> Option<Wire<Q, D>>
    where
        KF: FnMut(&'a Edge<P, C>) -> EdgeID<C>,
        KV: FnMut(&'a Edge<P, C>) -> Option<Edge<Q, D>>,
    {
        self.edge_iter()
            .map(|edge| {
                let new_edge = edge_map.entry_or_insert(edge).as_ref()?;
                match edge.orientation() {
                    true => Some(new_edge.clone()),
                    false => Some(new_edge.inverse()),
                }
            })
            .collect()
    }

    /// Returns a new wire whose curves are mapped by `curve_mapping` and
    /// whose points are mapped by `point_mapping`.
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    pub fn try_mapped<Q, D>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Option<Q>,
        mut curve_mapping: impl FnMut(&C) -> Option<D>,
    ) -> Option<Wire<Q, D>> {
        let mut vertex_map = EntryMap::new(Vertex::id, move |v| v.try_mapped(&mut point_mapping));
        let mut edge_map = EntryMap::new(
            Edge::id,
            edge_entry_map_try_closure(&mut vertex_map, &mut curve_mapping),
        );
        self.sub_try_mapped(&mut edge_map)
    }

    pub(super) fn sub_mapped<'a, Q, D, KF, KV>(
        &'a self,
        edge_map: &mut EntryMap<EdgeID<C>, Edge<Q, D>, KF, KV, &'a Edge<P, C>>,
    ) -> Wire<Q, D>
    where
        KF: FnMut(&'a Edge<P, C>) -> EdgeID<C>,
        KV: FnMut(&'a Edge<P, C>) -> Edge<Q, D>,
    {
        self.edge_iter()
            .map(|edge| {
                let new_edge = edge_map.entry_or_insert(edge);
                match edge.orientation() {
                    true => new_edge.clone(),
                    false => new_edge.inverse(),
                }
            })
            .collect()
    }
    /// Returns a new wire whose curves are mapped by `curve_mapping` and
    /// whose points are mapped by `point_mapping`.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[0, 1, 2, 3, 4]);
    /// let wire0: Wire<usize, usize> = vec![
    ///     Edge::new(&v[0], &v[1], 100),
    ///     Edge::new(&v[2], &v[1], 110).inverse(),
    ///     Edge::new(&v[3], &v[4], 120),
    ///     Edge::new(&v[4], &v[0], 130),
    /// ].into();
    /// let wire1 = wire0.mapped(
    ///     &move |i: &usize| *i as f64 + 0.5,
    ///     &move |j: &usize| *j as f64 + 1000.5,
    /// );
    ///
    /// // Check the points
    /// for (v0, v1) in wire0.vertex_iter().zip(wire1.vertex_iter()) {
    ///     let i = v0.get_point();
    ///     let j = v1.get_point();
    ///     assert_eq!(i as f64 + 0.5, j);
    /// }
    ///
    /// // Check the curves and orientation
    /// for (edge0, edge1) in wire0.edge_iter().zip(wire1.edge_iter()) {
    ///     let i = edge0.get_curve();
    ///     let j = edge1.get_curve();
    ///     assert_eq!(i as f64 + 1000.5, j);
    ///     assert_eq!(edge0.orientation(), edge1.orientation());
    /// }
    ///
    /// // Check the connection
    /// assert_eq!(wire1[0].back(), wire1[1].front());
    /// assert_ne!(wire1[1].back(), wire1[2].front());
    /// assert_eq!(wire1[2].back(), wire1[3].front());
    /// assert_eq!(wire1[3].back(), wire1[0].front());
    /// ```
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    pub fn mapped<Q, D>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Q,
        mut curve_mapping: impl FnMut(&C) -> D,
    ) -> Wire<Q, D> {
        let mut vertex_map = EntryMap::new(Vertex::id, move |v| v.mapped(&mut point_mapping));
        let mut edge_map = EntryMap::new(
            Edge::id,
            edge_entry_map_closure(&mut vertex_map, &mut curve_mapping),
        );
        self.sub_mapped(&mut edge_map)
    }

    /// Returns the consistence of the geometry of end vertices
    /// and the geometry of edge.
    #[inline(always)]
    pub fn is_geometric_consistent(&self) -> bool
    where
        P: Tolerance,
        C: BoundedCurve<Point = P>, {
        self.iter().all(|edge| edge.is_geometric_consistent())
    }

    /// Creates display struct for debugging the wire.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use WireDisplayFormat as WDF;
    /// let v = Vertex::news(&[0, 1, 2, 3, 4]);
    /// let wire: Wire<usize, usize> = vec![
    ///     Edge::new(&v[0], &v[1], 100),
    ///     Edge::new(&v[2], &v[1], 110).inverse(),
    ///     Edge::new(&v[3], &v[4], 120),
    /// ].into();
    ///
    /// let vertex_format = VertexDisplayFormat::AsPoint;
    /// let edge_format = EdgeDisplayFormat::VerticesTuple { vertex_format };
    ///
    /// assert_eq!(
    ///     &format!("{:?}", wire.display(WDF::EdgesListTuple {edge_format})),
    ///     "Wire([(0, 1), (1, 2), (3, 4)])",
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", wire.display(WDF::EdgesList {edge_format})),
    ///     "[(0, 1), (1, 2), (3, 4)]",
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", wire.display(WDF::VerticesList {vertex_format})),
    ///     "[0, 1, 2, 3, 4]",
    /// );
    /// ```
    #[inline(always)]
    pub fn display(&self, format: WireDisplayFormat) -> DebugDisplay<'_, Self, WireDisplayFormat> {
        DebugDisplay {
            entity: self,
            format,
        }
    }
}

pub(super) fn edge_entry_map_try_closure<'a, P, C, Q, D, KF, VF>(
    vertex_map: &'a mut EntryMap<VertexID<P>, Option<Vertex<Q>>, KF, VF, &'a Vertex<P>>,
    curve_mapping: &'a mut impl FnMut(&C) -> Option<D>,
) -> impl FnMut(&'a Edge<P, C>) -> Option<Edge<Q, D>> + 'a
where
    KF: FnMut(&'a Vertex<P>) -> VertexID<P>,
    VF: FnMut(&'a Vertex<P>) -> Option<Vertex<Q>>,
{
    move |edge| {
        let vf = edge.absolute_front();
        let vertex0 = vertex_map.entry_or_insert(vf).clone()?;
        let vb = edge.absolute_back();
        let vertex1 = vertex_map.entry_or_insert(vb).clone()?;
        let curve = curve_mapping(&*edge.curve.lock().unwrap())?;
        Some(Edge::debug_new(&vertex0, &vertex1, curve))
    }
}

pub(super) fn edge_entry_map_closure<'a, P, C, Q, D, KF, VF>(
    vertex_map: &'a mut EntryMap<VertexID<P>, Vertex<Q>, KF, VF, &'a Vertex<P>>,
    curve_mapping: &'a mut impl FnMut(&C) -> D,
) -> impl FnMut(&'a Edge<P, C>) -> Edge<Q, D> + 'a
where
    KF: FnMut(&'a Vertex<P>) -> VertexID<P>,
    VF: FnMut(&'a Vertex<P>) -> Vertex<Q>,
{
    move |edge| {
        let vf = edge.absolute_front();
        let vertex0 = vertex_map.entry_or_insert(vf).clone();
        let vb = edge.absolute_back();
        let vertex1 = vertex_map.entry_or_insert(vb).clone();
        let curve = curve_mapping(&*edge.curve.lock().unwrap());
        Edge::debug_new(&vertex0, &vertex1, curve)
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

impl<P, C> FromIterator<Edge<P, C>> for Wire<P, C> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = Edge<P, C>>>(iter: I) -> Wire<P, C> {
        Wire::from(VecDeque::from_iter(iter))
    }
}

impl<'a, P, C> FromIterator<&'a Edge<P, C>> for Wire<P, C> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = &'a Edge<P, C>>>(iter: I) -> Wire<P, C> {
        Wire::from(VecDeque::from_iter(iter.into_iter().map(Edge::clone)))
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
            } else if !self.cyclic {
                self.unconti_next = Some(edge.back().clone());
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

impl<P, C> Default for Wire<P, C> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            edge_list: Default::default(),
        }
    }
}

impl<'a, P: Debug, C: Debug> Debug for DebugDisplay<'a, Wire<P, C>, WireDisplayFormat> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.format {
            WireDisplayFormat::EdgesListTuple { edge_format } => f
                .debug_tuple("Wire")
                .field(&Self {
                    entity: self.entity,
                    format: WireDisplayFormat::EdgesList { edge_format },
                })
                .finish(),
            WireDisplayFormat::EdgesList { edge_format } => f
                .debug_list()
                .entries(
                    self.entity
                        .edge_iter()
                        .map(|edge| edge.display(edge_format)),
                )
                .finish(),
            WireDisplayFormat::VerticesList { vertex_format } => {
                let vertices: Vec<_> = self.entity.vertex_iter().collect();
                f.debug_list()
                    .entries(vertices.iter().map(|vertex| vertex.display(vertex_format)))
                    .finish()
            }
        }
    }
}
