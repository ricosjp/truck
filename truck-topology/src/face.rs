use crate::errors::Error;
use crate::wire::EdgeIter;
use crate::*;
use std::collections::HashMap;

impl<P, C, S> Face<P, C, S> {
    /// Creates a new face by a wire.
    /// # Failure
    /// All wires in `boundaries` must be non-empty, simple and closed. If not, returns the following errors:
    /// * If a wire is empty, then returns [`Error::EmptyWire`].
    /// * If a wire is not closed, then returns [`Error::NotClosedWire`].
    /// * If a wire is closed but not simple, then returns [`Error::NotSimpleWire`].
    ///
    /// [`Error::EmptyWire`]: errors/enum.Error.html#variant.EmptyWire
    /// [`Error::NotClosedWire`]: errors/enum.Error.html#variant.NotClosedWire
    /// [`Error::NotSimpleWire`]: errors/enum.Error.html#variant.NotSimpleWire
    /// # Examples
    /// ```
    /// # use truck_topology::*;
    /// # use errors::Error;
    /// let v = Vertex::news(&[(); 4]);
    /// let mut wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[3], ()),
    ///     Edge::new(&v[3], &v[0], ()),
    /// ]);
    /// assert!(Face::try_new(vec![wire], ()).is_ok());
    /// ```
    #[inline(always)]
    pub fn try_new(boundaries: Vec<Wire<P, C>>, surface: S) -> Result<Face<P, C, S>> {
        for wire in &boundaries {
            if wire.is_empty() {
                return Err(Error::EmptyWire);
            } else if !wire.is_closed() {
                return Err(Error::NotClosedWire);
            } else if !wire.is_simple() {
                return Err(Error::NotSimpleWire);
            }
        }
        if !Wire::disjoint_wires(&boundaries) {
            Err(Error::NotSimpleWire)
        } else {
            Ok(Face::new_unchecked(boundaries, surface))
        }
    }

    /// Creates a new face by a wire.
    /// # Panic
    /// All wires in `boundaries` must be non-empty, simple and closed.
    #[inline(always)]
    pub fn new(boundaries: Vec<Wire<P, C>>, surface: S) -> Face<P, C, S> {
        Face::try_new(boundaries, surface).remove_try()
    }

    /// Creates a new face by a wire.
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.  
    /// This method does NOT check the regularity conditions of `Face::try_new()`.  
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn new_unchecked(boundaries: Vec<Wire<P, C>>, surface: S) -> Face<P, C, S> {
        Face {
            boundaries: boundaries,
            orientation: true,
            surface: Arc::new(Mutex::new(surface)),
        }
    }

    /// Creates a new face by a wire.
    /// # Remarks
    /// This method check the regularity conditions of `Face::try_new()` in the debug mode.  
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn debug_new(boundaries: Vec<Wire<P, C>>, surface: S) -> Face<P, C, S> {
        match cfg!(debug_assertions) {
            true => Face::new(boundaries, surface),
            false => Face::new_unchecked(boundaries, surface),
        }
    }

    /// Returns the boundaries of the face.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 3]);
    /// let wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let mut face = Face::new(vec![wire], ());
    /// let boundaries = face.boundaries();
    /// for (i, vert) in boundaries[0].vertex_iter().enumerate() {
    ///     assert_eq!(vert, v[i]);
    /// }
    ///
    /// // If invert the face, the boundaries is also inverted.
    /// face.invert();
    /// assert_eq!(boundaries[0].inverse(), face.boundaries()[0]);
    /// ```
    #[inline(always)]
    pub fn boundaries(&self) -> Vec<Wire<P, C>> {
        match self.orientation {
            true => self.boundaries.clone(),
            false => self.boundaries.iter().map(|wire| wire.inverse()).collect(),
        }
    }

    /// Consumes `self` and returns the entity of its boundaries.
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(&[(), (), ()]);
    /// let wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let mut face = Face::new(vec![wire], ());
    /// let boundaries = face.clone().into_boundaries();
    /// for (i, vert) in boundaries[0].vertex_iter().enumerate() {
    ///     assert_eq!(vert, v[i]);
    /// }
    ///
    /// // If invert the face, the boundaries is also inverted.
    /// face.invert();
    /// assert_eq!(boundaries[0].inverse(), face.into_boundaries()[0]);
    /// ```
    #[inline(always)]
    pub fn into_boundaries(self) -> Vec<Wire<P, C>> {
        match self.orientation {
            true => self.boundaries,
            false => self.boundaries(),
        }
    }

    /// Returns the reference of the boundaries wire which is generated by constructor.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), ()]);
    /// let wire = Wire::from(vec![
    ///      Edge::new(&v[0], &v[1], ()),
    ///      Edge::new(&v[1], &v[2], ()),
    ///      Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let mut face = Face::new(vec![wire], ());
    /// let boundaries = face.boundaries();
    /// face.invert();
    ///
    /// // The result of face.boudnary() is already inversed.
    /// assert_eq!(face.boundaries()[0], boundaries[0].inverse());
    ///
    /// // The absolute boundaries does never change.
    /// assert_eq!(face.absolute_boundaries(), &boundaries);
    /// ```
    #[inline(always)]
    pub fn absolute_boundaries(&self) -> &Vec<Wire<P, C>> { &self.boundaries }

    /// Returns an iterator over all edges in the boundaries.
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), ()]);
    /// let wire = Wire::from(vec![
    ///      Edge::new(&v[0], &v[1], ()),
    ///      Edge::new(&v[1], &v[2], ()),
    ///      Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let mut face = Face::new(vec![wire], ());
    /// face.invert();
    /// let boundaries = face.boundaries().clone();
    /// let edge_iter0 = boundaries.iter().flat_map(Wire::edge_iter);
    /// let edge_iter1 = face.boundary_iters().into_iter().flatten();
    /// for (edge0, edge1) in edge_iter0.zip(edge_iter1) {
    ///     assert_eq!(edge0, &edge1);
    /// }
    /// ```
    #[inline(always)]
    pub fn boundary_iters(&self) -> Vec<BoundaryIter<P, C>> {
        self.boundaries
            .iter()
            .map(|wire| BoundaryIter {
                edge_iter: wire.edge_iter(),
                orientation: self.orientation,
            })
            .collect()
    }

    /// Adds a boundary to the face.
    #[inline(always)]
    pub fn try_add_boundary(&mut self, wire: Wire<P, C>) -> Result<()> {
        if wire.is_empty() {
            return Err(Error::EmptyWire);
        } else if !wire.is_closed() {
            return Err(Error::NotClosedWire);
        } else if !wire.is_simple() {
            return Err(Error::NotSimpleWire);
        }
        self.boundaries.push(wire);
        if !Wire::disjoint_wires(&self.boundaries) {
            self.boundaries.pop();
            return Err(Error::NotDisjointWires);
        }
        Ok(())
    }

    /// Adds a boundary to the face.
    #[inline(always)]
    pub fn add_boundary(&mut self, wire: Wire<P, C>) { self.try_add_boundary(wire).remove_try() }

    /// Returns the orientation of face.
    ///
    /// The result of this method is the same with `self.boundaries() == self.absolute_boundaries().clone()`.
    /// Moreover, if this method returns false, `self.boundaries() == self.absolute_boundaries().inverse()`.
    #[inline(always)]
    pub fn orientation(&self) -> bool { self.orientation }

    /// Returns the id of face.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), ()]);
    /// let wire = Wire::from(vec![
    ///      Edge::new(&v[0], &v[1], ()),
    ///      Edge::new(&v[1], &v[2], ()),
    ///      Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let face0 = Face::new(vec![wire], 0);
    /// let face1 = face0.clone();
    ///
    /// // Two faces have the same content.
    /// assert_eq!(*face0.try_lock_surface().unwrap(), 0);
    /// assert_eq!(*face1.try_lock_surface().unwrap(), 0);
    ///
    /// {
    ///     let mut surface = face0.try_lock_surface().unwrap();
    ///     *surface = 1;
    /// }
    /// // The contents of two vertices are synchronized.
    /// assert_eq!(*face0.try_lock_surface().unwrap(), 1);
    /// assert_eq!(*face1.try_lock_surface().unwrap(), 1);
    ///
    /// // The thread is not blocked even if the surface is already locked.
    /// let lock = face0.try_lock_surface();
    /// assert!(face1.try_lock_surface().is_err());
    /// ```
    #[inline(always)]
    pub fn try_lock_surface(&self) -> TryLockResult<MutexGuard<S>> { self.surface.try_lock() }
    /// Returns the id of face.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), ()]);
    /// let wire = Wire::from(vec![
    ///      Edge::new(&v[0], &v[1], ()),
    ///      Edge::new(&v[1], &v[2], ()),
    ///      Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let face0 = Face::new(vec![wire], 0);
    /// let face1 = face0.clone();
    ///
    /// // Two faces have the same content.
    /// assert_eq!(*face0.lock_surface().unwrap(), 0);
    /// assert_eq!(*face1.lock_surface().unwrap(), 0);
    ///
    /// {
    ///     let mut surface = face0.lock_surface().unwrap();
    ///     *surface = 1;
    /// }
    /// // The contents of two vertices are synchronized.
    /// assert_eq!(*face0.lock_surface().unwrap(), 1);
    /// assert_eq!(*face1.lock_surface().unwrap(), 1);
    ///
    /// // Check the behavior of `lock`.
    /// std::thread::spawn(move || {
    ///     *face0.lock_surface().unwrap() = 2;
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*face1.lock_surface().unwrap(), 2);    
    /// ```
    #[inline(always)]
    pub fn lock_surface(&self) -> LockResult<MutexGuard<S>> { self.surface.lock() }

    /// Inverts the direction of the face.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_topology::errors::Error;
    /// let v = Vertex::news(&[(), (), ()]);
    /// let wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let mut face = Face::new(vec![wire], ());
    /// let org_face = face.clone();
    /// let org_bdry = face.boundaries();
    /// face.invert();
    ///
    /// // Two faces are the same face.
    /// face.is_same(&org_face);
    ///
    /// // The boundaries is inverted.
    /// let inversed_edge_iter = org_bdry[0].inverse().edge_into_iter();
    /// let face_edge_iter = &mut face.boundary_iters()[0];
    /// for (edge0, edge1) in inversed_edge_iter.zip(face_edge_iter) {
    ///     assert_eq!(edge0, edge1);
    /// }
    /// ```
    #[inline(always)]
    pub fn invert(&mut self) -> &mut Self {
        self.orientation = !self.orientation;
        self
    }

    /// Returns whether two faces are the same. Returns `true` even if the orientaions are different.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 3]);
    /// let wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let face0 = Face::new(vec![wire], ());
    /// let face1 = face0.inverse();
    /// assert_ne!(face0, face1);
    /// assert!(face0.is_same(&face1));
    /// ```
    #[inline(always)]
    pub fn is_same(&self, other: &Self) -> bool {
        std::ptr::eq(Arc::as_ptr(&self.surface), Arc::as_ptr(&other.surface))
    }

    /// Returns the id that does not depend on the direction of the face.
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
    #[inline(always)]
    pub fn id(&self) -> FaceID<S> {
        FaceID {
            entity: Arc::as_ptr(&self.surface),
        }
    }

    /// Returns the inverse face.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use truck_topology::errors::Error;
    /// let v = Vertex::news(&[(), (), ()]);
    /// let wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let mut face = Face::new(vec![wire], ());
    /// let inverted = face.inverse();
    ///
    /// // Two faces are the same face.
    /// assert!(face.is_same(&inverted));
    ///
    /// // Two faces has the same id.
    /// assert_eq!(face.id(), inverted.id());
    ///
    /// // The boundaries is inverted.
    /// let mut inversed_edge_iter = face.boundaries()[0].inverse().edge_into_iter();
    /// let face_edge_iter = &mut inverted.boundary_iters()[0];
    /// for (edge0, edge1) in inversed_edge_iter.zip(face_edge_iter) {
    ///     assert_eq!(edge0, edge1);
    /// }
    /// ```
    #[inline(always)]
    pub fn inverse(&self) -> Face<P, C, S> {
        let mut face = self.clone();
        face.invert();
        face
    }

    /// Returns whether two faces `self` and `other` have a shared edge.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use std::iter::FromIterator;
    /// let v = Vertex::news(&[(); 4]);
    /// let shared_edge = Edge::new(&v[0], &v[1], ());
    /// let another_edge = Edge::new(&v[0], &v[1], ());
    /// let inversed_edge = shared_edge.inverse();
    /// let wire = vec![
    ///     Wire::from_iter(vec![&Edge::new(&v[2], &v[0], ()), &shared_edge, &Edge::new(&v[1], &v[2], ())]),
    ///     Wire::from_iter(vec![&Edge::new(&v[2], &v[0], ()), &another_edge, &Edge::new(&v[1], &v[2], ())]),
    ///     Wire::from_iter(vec![&Edge::new(&v[3], &v[0], ()), &shared_edge, &Edge::new(&v[1], &v[3], ())]),
    ///     Wire::from_iter(vec![&Edge::new(&v[3], &v[1], ()), &inversed_edge, &Edge::new(&v[0], &v[3], ())]),
    /// ];
    /// let face: Vec<_> = wire.into_iter().map(|w| Face::new(vec![w], ())).collect();
    /// assert!(face[0].border_on(&face[2]));
    /// assert!(!face[1].border_on(&face[2]));
    /// assert!(face[0].border_on(&face[3]));
    /// ```
    pub fn border_on(&self, other: &Face<P, C, S>) -> bool {
        let mut hashmap = HashMap::new();
        for edge in self.boundaries.iter().flat_map(|wire| wire.edge_iter()) {
            hashmap.insert(edge.id(), edge);
        }
        for edge in other.boundaries.iter().flat_map(|wire| wire.edge_iter()) {
            if hashmap.insert(edge.id(), edge).is_some() {
                return true;
            }
        }
        false
    }
}

impl<P, C, S> Clone for Face<P, C, S> {
    #[inline(always)]
    fn clone(&self) -> Face<P, C, S> {
        Face {
            boundaries: self.boundaries.clone(),
            orientation: self.orientation,
            surface: Arc::clone(&self.surface),
        }
    }
}

impl<P, C, S> PartialEq for Face<P, C, S> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(Arc::as_ptr(&self.surface), Arc::as_ptr(&other.surface))
            && self.orientation == other.orientation
    }
}

impl<P, C, S> Eq for Face<P, C, S> {}

impl<P, C, S> Hash for Face<P, C, S> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.surface), state);
        self.orientation.hash(state);
    }
}

/// An iterator over the edges in the boundaries of a face.
/// # Examples
/// ```
/// use truck_topology::*;
/// let v = Vertex::news(&[(); 4]);
/// let wire = Wire::from(vec![
///     Edge::new(&v[0], &v[1], ()),
///     Edge::new(&v[1], &v[2], ()),
///     Edge::new(&v[2], &v[3], ()),
///     Edge::new(&v[3], &v[0], ()),
/// ]);
/// let face = Face::new(vec![wire.clone()], ());
///
/// let iter = &mut face.boundary_iters()[0];
/// assert_eq!(iter.next().as_ref(), Some(&wire[0]));
/// assert_eq!(iter.next_back().as_ref(), Some(&wire[3])); // double ended
/// assert_eq!(iter.next().as_ref(), Some(&wire[1]));
/// assert_eq!(iter.next().as_ref(), Some(&wire[2]));
/// assert_eq!(iter.next_back().as_ref(), None);
/// assert_eq!(iter.next().as_ref(), None); // fused
/// ```
#[derive(Clone, Debug)]
pub struct BoundaryIter<'a, P, C> {
    edge_iter: EdgeIter<'a, P, C>,
    orientation: bool,
}

impl<'a, P, C> Iterator for BoundaryIter<'a, P, C> {
    type Item = Edge<P, C>;
    #[inline(always)]
    fn next(&mut self) -> Option<Edge<P, C>> {
        match self.orientation {
            true => self.edge_iter.next().map(|edge| edge.clone()),
            false => self.edge_iter.next_back().map(|edge| edge.inverse()),
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) { (self.len(), Some(self.len())) }

    #[inline(always)]
    fn last(mut self) -> Option<Edge<P, C>> { self.next_back() }
}

impl<'a, P, C> DoubleEndedIterator for BoundaryIter<'a, P, C> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Edge<P, C>> {
        match self.orientation {
            true => self.edge_iter.next_back().map(|edge| edge.clone()),
            false => self.edge_iter.next().map(|edge| edge.inverse()),
        }
    }
}

impl<'a, P, C> ExactSizeIterator for BoundaryIter<'a, P, C> {
    #[inline(always)]
    fn len(&self) -> usize { self.edge_iter.len() }
}

impl<'a, P, C> std::iter::FusedIterator for BoundaryIter<'a, P, C> {}

impl<S> Hash for FaceID<S> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) { std::ptr::hash(self.entity, state); }
}

impl<S> PartialEq for FaceID<S> {
    #[inline(always)]
    fn eq(&self, other: &FaceID<S>) -> bool { std::ptr::eq(self.entity, other.entity) }
}

impl<S> Eq for FaceID<S> {}

impl<S> Clone for FaceID<S> {
    #[inline(always)]
    fn clone(&self) -> Self {
        FaceID {
            entity: self.entity,
        }
    }
}

impl<S> Copy for FaceID<S> {}

impl<S> std::fmt::Debug for FaceID<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("{:p}", self.entity))
    }
}
