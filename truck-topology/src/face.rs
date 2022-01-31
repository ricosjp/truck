use crate::errors::Error;
use crate::wire::EdgeIter;
use crate::*;
use rustc_hash::FxHashMap as HashMap;

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
            boundaries,
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

    #[inline(always)]
    fn renew_pointer(&mut self)
    where S: Clone {
        let surface = self.get_surface();
        self.surface = Arc::new(Mutex::new(surface));
    }

    /// Adds a boundary to the face.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), (), (), (), ()]);
    /// let wire0 = Wire::from(vec![
    ///      Edge::new(&v[0], &v[1], ()),
    ///      Edge::new(&v[1], &v[2], ()),
    ///      Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///      Edge::new(&v[3], &v[4], ()),
    ///      Edge::new(&v[4], &v[5], ()),
    ///      Edge::new(&v[5], &v[3], ()),
    /// ]);
    /// let mut face0 = Face::new(vec![wire0.clone()], ());
    /// face0.try_add_boundary(wire1.clone()).unwrap();
    /// let face1 = Face::new(vec![wire0, wire1], ());
    /// assert_eq!(face0.boundaries(), face1.boundaries());
    /// ```
    /// # Remarks
    /// 1. If the face is inverted, then the added wire is inverted as absolute bounday.
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), (), (), (), ()]);
    /// let wire0 = Wire::from(vec![
    ///      Edge::new(&v[0], &v[1], ()),
    ///      Edge::new(&v[1], &v[2], ()),
    ///      Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///      Edge::new(&v[3], &v[4], ()),
    ///      Edge::new(&v[5], &v[4], ()).inverse(),
    ///      Edge::new(&v[5], &v[3], ()),
    /// ]);
    /// let mut face = Face::new(vec![wire0], ());
    /// face.invert();
    /// face.try_add_boundary(wire1.clone()).unwrap();
    ///
    /// // The boundary is added in compatible with the face orientation.
    /// assert_eq!(face.boundaries()[1], wire1);
    ///
    /// // The absolute bounday is inverted!
    /// let iter0 = face.absolute_boundaries()[1].edge_iter();
    /// let iter1 = wire1.edge_iter().rev();
    /// for (edge0, edge1) in iter0.zip(iter1) {
    ///     assert_eq!(edge0.id(), edge1.id());
    ///     assert_eq!(edge0.orientation(), !edge1.orientation());
    /// }
    /// ```
    /// 2. This method renew the face id.
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), (), (), (), ()]);
    /// let wire0 = Wire::from(vec![
    ///      Edge::new(&v[0], &v[1], ()),
    ///      Edge::new(&v[1], &v[2], ()),
    ///      Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///      Edge::new(&v[3], &v[4], ()),
    ///      Edge::new(&v[5], &v[4], ()).inverse(),
    ///      Edge::new(&v[5], &v[3], ()),
    /// ]);
    /// let mut face0 = Face::new(vec![wire0], ());
    /// let face1 = face0.clone();
    /// assert_eq!(face0.id(), face1.id());
    /// face0.try_add_boundary(wire1).unwrap();
    /// assert_ne!(face0.id(), face1.id());
    /// ```
    #[inline(always)]
    pub fn try_add_boundary(&mut self, mut wire: Wire<P, C>) -> Result<()>
    where S: Clone {
        if wire.is_empty() {
            return Err(Error::EmptyWire);
        } else if !wire.is_closed() {
            return Err(Error::NotClosedWire);
        } else if !wire.is_simple() {
            return Err(Error::NotSimpleWire);
        }
        if !self.orientation {
            wire.invert();
        }
        self.boundaries.push(wire);
        self.renew_pointer();
        if !Wire::disjoint_wires(&self.boundaries) {
            self.boundaries.pop();
            return Err(Error::NotDisjointWires);
        }
        Ok(())
    }

    /// Adds a boundary to the face.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), (), (), (), ()]);
    /// let wire0 = Wire::from(vec![
    ///      Edge::new(&v[0], &v[1], ()),
    ///      Edge::new(&v[1], &v[2], ()),
    ///      Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///      Edge::new(&v[3], &v[4], ()),
    ///      Edge::new(&v[4], &v[5], ()),
    ///      Edge::new(&v[5], &v[3], ()),
    /// ]);
    /// let mut face0 = Face::new(vec![wire0.clone()], ());
    /// face0.add_boundary(wire1.clone());
    /// let face1 = Face::new(vec![wire0, wire1], ());
    /// assert_eq!(face0.boundaries(), face1.boundaries());
    /// ```
    /// # Remarks
    /// 1. If the face is inverted, then the added wire is inverted as absolute bounday.
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), (), (), (), ()]);
    /// let wire0 = Wire::from(vec![
    ///      Edge::new(&v[0], &v[1], ()),
    ///      Edge::new(&v[1], &v[2], ()),
    ///      Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///      Edge::new(&v[3], &v[4], ()),
    ///      Edge::new(&v[5], &v[4], ()).inverse(),
    ///      Edge::new(&v[5], &v[3], ()),
    /// ]);
    /// let mut face = Face::new(vec![wire0], ());
    /// face.invert();
    /// face.add_boundary(wire1.clone());
    ///
    /// // The boundary is added in compatible with the face orientation.
    /// assert_eq!(face.boundaries()[1], wire1);
    ///
    /// // The absolute bounday is inverted!
    /// let iter0 = face.absolute_boundaries()[1].edge_iter();
    /// let iter1 = wire1.edge_iter().rev();
    /// for (edge0, edge1) in iter0.zip(iter1) {
    ///     assert_eq!(edge0.id(), edge1.id());
    ///     assert_eq!(edge0.orientation(), !edge1.orientation());
    /// }
    /// ```
    /// 2. This method renew the face id.
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), (), (), (), ()]);
    /// let wire0 = Wire::from(vec![
    ///      Edge::new(&v[0], &v[1], ()),
    ///      Edge::new(&v[1], &v[2], ()),
    ///      Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///      Edge::new(&v[3], &v[4], ()),
    ///      Edge::new(&v[5], &v[4], ()).inverse(),
    ///      Edge::new(&v[5], &v[3], ()),
    /// ]);
    /// let mut face0 = Face::new(vec![wire0], ());
    /// let face1 = face0.clone();
    /// assert_eq!(face0.id(), face1.id());
    /// face0.add_boundary(wire1);
    /// assert_ne!(face0.id(), face1.id());
    /// ```
    #[inline(always)]
    pub fn add_boundary(&mut self, wire: Wire<P, C>)
    where S: Clone {
        self.try_add_boundary(wire).remove_try()
    }

    /// Returns a new face whose surface is mapped by `surface_mapping`,
    /// curves are mapped by `curve_mapping` and points are mapped by `point_mapping`.
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    pub fn try_mapped<Q, D, T>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Option<Q>,
        mut curve_mapping: impl FnMut(&C) -> Option<D>,
        mut surface_mapping: impl FnMut(&S) -> Option<T>,
    ) -> Option<Face<Q, D, T>> {
        let wires = self
            .absolute_boundaries()
            .iter()
            .map(|wire| wire.try_mapped(&mut point_mapping, &mut curve_mapping))
            .collect::<Option<Vec<_>>>()?;
        let surface = surface_mapping(&*self.surface.lock().unwrap())?;
        let mut face = Face::debug_new(wires, surface);
        if !self.orientation() {
            face.invert();
        }
        Some(face)
    }

    /// Returns a new face whose surface is mapped by `surface_mapping`,
    /// curves are mapped by `curve_mapping` and points are mapped by `point_mapping`.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[0, 1, 2, 3, 4, 5, 6]);
    /// let wire0 = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], 100),
    ///     Edge::new(&v[1], &v[2], 200),
    ///     Edge::new(&v[2], &v[3], 300),
    ///     Edge::new(&v[3], &v[0], 400),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///     Edge::new(&v[4], &v[5], 500),
    ///     Edge::new(&v[6], &v[5], 600).inverse(),
    ///     Edge::new(&v[6], &v[4], 700),
    /// ]);
    /// let face0 = Face::new(vec![wire0, wire1], 10000);
    /// let face1 = face0.mapped(
    ///     &move |i: &usize| *i + 10,
    ///     &move |j: &usize| *j + 1000,
    ///     &move |k: &usize| *k + 100000,
    /// );
    /// # for wire in face1.boundaries() {
    /// #    assert!(wire.is_closed());
    /// #    assert!(wire.is_simple());
    /// # }
    ///
    /// assert_eq!(
    ///     face0.get_surface() + 100000,
    ///     face1.get_surface(),
    /// );
    /// let biters0 = face0.boundary_iters();
    /// let biters1 = face1.boundary_iters();
    /// for (biter0, biter1) in biters0.into_iter().zip(biters1) {
    ///     for (edge0, edge1) in biter0.zip(biter1) {
    ///         assert_eq!(
    ///             edge0.front().get_point() + 10,
    ///             edge1.front().get_point(),
    ///         );
    ///         assert_eq!(
    ///             edge0.back().get_point() + 10,
    ///             edge1.back().get_point(),
    ///         );
    ///         assert_eq!(edge0.orientation(), edge1.orientation());
    ///         assert_eq!(
    ///             edge0.get_curve() + 1000,
    ///             edge1.get_curve(),
    ///         );
    ///     }
    /// }
    /// ```
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    pub fn mapped<Q, D, T>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Q,
        mut curve_mapping: impl FnMut(&C) -> D,
        mut surface_mapping: impl FnMut(&S) -> T,
    ) -> Face<Q, D, T> {
        let wires: Vec<_> = self
            .absolute_boundaries()
            .iter()
            .map(|wire| wire.mapped(&mut point_mapping, &mut curve_mapping))
            .collect();
        let surface = surface_mapping(&*self.surface.lock().unwrap());
        let mut face = Face::debug_new(wires, surface);
        if !self.orientation() {
            face.invert();
        }
        face
    }

    /// Returns the orientation of face.
    ///
    /// The result of this method is the same with `self.boundaries() == self.absolute_boundaries().clone()`.
    /// Moreover, if this method returns false, `self.boundaries() == self.absolute_boundaries().inverse()`.
    #[inline(always)]
    pub fn orientation(&self) -> bool { self.orientation }

    /// Returns the clone of surface of face.
    #[inline(always)]
    pub fn get_surface(&self) -> S
    where S: Clone {
        self.surface.lock().unwrap().clone()
    }

    /// Sets the surface of face.
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
    /// assert_eq!(face0.get_surface(), 0);
    /// assert_eq!(face1.get_surface(), 0);
    ///
    /// // Set surface
    /// face0.set_surface(1);
    ///
    /// // The contents of two vertices are synchronized.
    /// assert_eq!(face0.get_surface(), 1);
    /// assert_eq!(face1.get_surface(), 1);
    /// ```
    #[inline(always)]
    pub fn set_surface(&self, surface: S) { *self.surface.lock().unwrap() = surface; }

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
    pub fn id(&self) -> FaceID<S> { ID::new(Arc::as_ptr(&self.surface)) }

    /// Returns how many same faces.
    ///
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 3]);
    /// let wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    /// ]);
    ///
    /// // Create one face
    /// let face0 = Face::new(vec![wire.clone()], ());
    /// assert_eq!(face0.count(), 1);
    /// // Create another face, independent from face0
    /// let face1 = Face::new(vec![wire.clone()], ());
    /// assert_eq!(face0.count(), 1);
    /// // Clone face0, the result will be 2.
    /// let face2 = face0.clone();
    /// assert_eq!(face0.count(), 2);
    /// assert_eq!(face2.count(), 2);
    /// // drop face2, the result will be 1.
    /// drop(face2);
    /// assert_eq!(face0.count(), 1);
    /// ```
    #[inline(always)]
    pub fn count(&self) -> usize { Arc::strong_count(&self.surface) }

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
        let mut hashmap = HashMap::default();
        let edge_iter = self.boundary_iters().into_iter().flatten();
        edge_iter.for_each(|edge| {
            hashmap.insert(edge.id(), edge);
        });
        let mut edge_iter = other.boundary_iters().into_iter().flatten();
        edge_iter.any(|edge| hashmap.insert(edge.id(), edge).is_some())
    }

    /// Cuts a face with only one boundary by an edge.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), (), ()]);
    /// let wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[3], ()),
    ///     Edge::new(&v[3], &v[0], ()),
    /// ]);
    /// let mut face0 = Face::new(vec![wire], ());
    ///
    /// let face1 = face0.cut_by_edge(Edge::new(&v[1], &v[3], ())).unwrap();
    ///
    /// // The front vertex of face0's boundary becomes the back of cutting edge.
    /// let v0: Vec<Vertex<()>> = face0.boundaries()[0].vertex_iter().collect();
    /// assert_eq!(v0, vec![v[3].clone(), v[0].clone(), v[1].clone()]);
    ///
    /// let v1: Vec<Vertex<()>> = face1.boundaries()[0].vertex_iter().collect();
    /// assert_eq!(v1, vec![v[1].clone(), v[2].clone(), v[3].clone()]);
    /// ```
    /// # Failures
    /// Returns `None` if:
    /// - `self` has several boundaries, or
    /// - `self` does not include vertices of the end vertices of `edge`.
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 6]);
    /// let wire0 = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///     Edge::new(&v[3], &v[4], ()),
    ///     Edge::new(&v[4], &v[5], ()),
    ///     Edge::new(&v[5], &v[3], ()),
    /// ]);
    /// let mut face = Face::new(vec![wire0, wire1], ());
    /// assert!(face.cut_by_edge(Edge::new(&v[1], &v[2], ())).is_none());
    /// ```
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), (), (), (), ()]);
    /// let wire = Wire::from(vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[3], ()),
    ///     Edge::new(&v[3], &v[0], ()),
    /// ]);
    /// let mut face0 = Face::new(vec![wire], ());
    /// assert!(face0.cut_by_edge(Edge::new(&v[1], &v[4], ())).is_none());
    pub fn cut_by_edge(&mut self, edge: Edge<P, C>) -> Option<Self>
    where S: Clone {
        if self.boundaries.len() != 1 {
            return None;
        }
        let wire = &mut self.boundaries[0];
        let i = wire
            .edge_iter()
            .enumerate()
            .find(|(_, e)| e.front() == edge.back())
            .map(|(i, _)| i)?;
        let j = wire
            .edge_iter()
            .enumerate()
            .find(|(_, e)| e.back() == edge.front())
            .map(|(i, _)| i)?;
        wire.rotate_left(i);
        let j = (j + wire.len() - i) % wire.len();
        let mut new_wire = wire.split_off(j + 1);
        wire.push_back(edge.clone());
        new_wire.push_back(edge.inverse());
        self.renew_pointer();
        debug_assert!(Face::try_new(self.boundaries.clone(), ()).is_ok());
        debug_assert!(Face::try_new(vec![new_wire.clone()], ()).is_ok());
        Some(Face {
            boundaries: vec![new_wire],
            orientation: self.orientation,
            surface: Arc::new(Mutex::new(self.get_surface())),
        })
    }

    /// Glue two faces at boundaries.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(); 8]);
    /// let edge = vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    ///     Edge::new(&v[3], &v[4], ()),
    ///     Edge::new(&v[4], &v[5], ()),
    ///     Edge::new(&v[5], &v[3], ()),
    ///     Edge::new(&v[6], &v[2], ()),
    ///     Edge::new(&v[1], &v[6], ()),
    ///     Edge::new(&v[7], &v[5], ()),
    ///     Edge::new(&v[4], &v[7], ()),
    /// ];
    /// let wire0 = Wire::from(vec![
    ///     edge[0].clone(),
    ///     edge[1].clone(),
    ///     edge[2].clone(),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///     edge[3].clone(),
    ///     edge[4].clone(),
    ///     edge[5].clone(),
    /// ]);
    /// let wire2 = Wire::from(vec![
    ///     edge[6].clone(),
    ///     edge[1].inverse(),
    ///     edge[7].clone(),
    /// ]);
    /// let wire3 = Wire::from(vec![
    ///     edge[8].clone(),
    ///     edge[4].inverse(),
    ///     edge[9].clone(),
    /// ]);
    /// let face0 = Face::new(vec![wire0, wire1], ());
    /// let face1 = Face::new(vec![wire2, wire3], ());
    /// let face = face0.glue_at_boundaries(&face1).unwrap();
    /// let boundaries = face.boundary_iters();
    /// assert_eq!(boundaries.len(), 2);
    /// assert_eq!(boundaries[0].len(), 4);
    /// assert_eq!(boundaries[1].len(), 4);
    /// ```
    pub fn glue_at_boundaries(&self, other: &Self) -> Option<Self>
    where
        S: Clone + PartialEq,
        Wire<P, C>: Debug, {
        let surface = self.get_surface();
        if surface != other.get_surface() || self.orientation() != other.orientation() {
            return None;
        }
        let mut vemap: HashMap<VertexID<P>, &Edge<P, C>> = self
            .absolute_boundaries()
            .iter()
            .flatten()
            .map(|edge| (edge.front().id(), edge))
            .collect();
        other
            .absolute_boundaries()
            .iter()
            .flatten()
            .try_for_each(|edge| {
                if let Some(edge0) = vemap.get(&edge.back().id()) {
                    if edge.front() == edge0.back() {
                        if edge.is_same(edge0) {
                            vemap.remove(&edge.back().id());
                            return Some(());
                        } else {
                            return None;
                        }
                    }
                }
                vemap.insert(edge.front().id(), edge);
                Some(())
            })?;
        if vemap.is_empty() {
            return None;
        }
        let mut boundaries = Vec::new();
        while !vemap.is_empty() {
            let mut wire = Wire::new();
            let v = *vemap.iter().next().unwrap().0;
            let mut edge = vemap.remove(&v).unwrap();
            wire.push_back(edge.clone());
            while let Some(edge0) = vemap.remove(&edge.back().id()) {
                wire.push_back(edge0.clone());
                edge = edge0;
            }
            boundaries.push(wire);
        }
        debug_assert!(Face::try_new(boundaries.clone(), ()).is_ok());
        Some(Face {
            boundaries,
            orientation: self.orientation(),
            surface: Arc::new(Mutex::new(surface)),
        })
    }

    /// Creates display struct for debugging the face.
    ///
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use FaceDisplayFormat as FDF;
    /// let v = Vertex::news(&[0, 1, 2, 3, 4, 5]);
    /// let edge = vec![
    ///     Edge::new(&v[0], &v[1], ()),
    ///     Edge::new(&v[1], &v[2], ()),
    ///     Edge::new(&v[2], &v[0], ()),
    ///     Edge::new(&v[3], &v[4], ()),
    ///     Edge::new(&v[4], &v[5], ()),
    ///     Edge::new(&v[5], &v[3], ()),
    /// ];
    /// let wire0 = Wire::from(vec![
    ///     edge[0].clone(),
    ///     edge[1].clone(),
    ///     edge[2].clone(),
    /// ]);
    /// let wire1 = Wire::from(vec![
    ///     edge[3].clone(),
    ///     edge[4].clone(),
    ///     edge[5].clone(),
    /// ]);
    /// let face = Face::new(vec![wire0, wire1], 120);
    ///
    /// let vertex_format = VertexDisplayFormat::AsPoint;
    /// let edge_format = EdgeDisplayFormat::VerticesTuple { vertex_format };
    /// let wire_format = WireDisplayFormat::EdgesList { edge_format };
    ///
    /// assert_eq!(
    ///     format!("{:?}", face.display(FDF::Full { wire_format })),
    ///     format!("Face {{ id: {:?}, boundaries: [[(0, 1), (1, 2), (2, 0)], [(3, 4), (4, 5), (5, 3)]], entity: 120 }}", face.id()),
    /// );
    /// assert_eq!(
    ///     format!("{:?}", face.display(FDF::BoundariesAndID { wire_format })),
    ///     format!("Face {{ id: {:?}, boundaries: [[(0, 1), (1, 2), (2, 0)], [(3, 4), (4, 5), (5, 3)]] }}", face.id()),
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", face.display(FDF::BoundariesAndSurface { wire_format })),
    ///     "Face { boundaries: [[(0, 1), (1, 2), (2, 0)], [(3, 4), (4, 5), (5, 3)]], entity: 120 }",
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", face.display(FDF::LoopsListTuple { wire_format })),
    ///     "Face([[(0, 1), (1, 2), (2, 0)], [(3, 4), (4, 5), (5, 3)]])",
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", face.display(FDF::LoopsList { wire_format })),
    ///     "[[(0, 1), (1, 2), (2, 0)], [(3, 4), (4, 5), (5, 3)]]",
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", face.display(FDF::AsSurface)),
    ///     "120",
    /// );
    /// ```
    #[inline(always)]
    pub fn display(&self, format: FaceDisplayFormat) -> DebugDisplay<Self, FaceDisplayFormat> {
        DebugDisplay {
            entity: self,
            format,
        }
    }
}

impl<P, C, S: Clone + Invertible> Face<P, C, S> {
    /// Returns the cloned surface in face.
    /// If face is inverted, then the returned surface is also inverted.
    #[inline(always)]
    pub fn oriented_surface(&self) -> S {
        match self.orientation {
            true => self.surface.lock().unwrap().clone(),
            false => self.surface.lock().unwrap().inverse(),
        }
    }
}

impl<P, C, S> Face<P, C, S>
where
    P: Tolerance,
    C: BoundedCurve<Point = P>,
    S: IncludeCurve<C>,
{
    /// Returns the consistence of the geometry of end vertices
    /// and the geometry of edge.
    #[inline(always)]
    pub fn is_geometric_consistent(&self) -> bool {
        let surface = &*self.surface.lock().unwrap();
        self.boundary_iters().into_iter().flatten().all(|edge| {
            let edge_consist = edge.is_geometric_consistent();
            let curve = &*edge.curve.lock().unwrap();
            let curve_consist = surface.include(curve);
            edge_consist && curve_consist
        })
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
            true => self.edge_iter.next().cloned(),
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
            true => self.edge_iter.next_back().cloned(),
            false => self.edge_iter.next().map(|edge| edge.inverse()),
        }
    }
}

impl<'a, P, C> ExactSizeIterator for BoundaryIter<'a, P, C> {
    #[inline(always)]
    fn len(&self) -> usize { self.edge_iter.len() }
}

impl<'a, P, C> std::iter::FusedIterator for BoundaryIter<'a, P, C> {}

impl<'a, P: Debug, C: Debug, S: Debug> Debug
    for DebugDisplay<'a, Face<P, C, S>, FaceDisplayFormat>
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.format {
            FaceDisplayFormat::Full { wire_format } => f
                .debug_struct("Face")
                .field("id", &self.entity.id())
                .field(
                    "boundaries",
                    &self
                        .entity
                        .boundaries()
                        .iter()
                        .map(|wire| wire.display(wire_format))
                        .collect::<Vec<_>>(),
                )
                .field("entity", &MutexFmt(&self.entity.surface))
                .finish(),
            FaceDisplayFormat::BoundariesAndID { wire_format } => f
                .debug_struct("Face")
                .field("id", &self.entity.id())
                .field(
                    "boundaries",
                    &self
                        .entity
                        .boundaries()
                        .iter()
                        .map(|wire| wire.display(wire_format))
                        .collect::<Vec<_>>(),
                )
                .finish(),
            FaceDisplayFormat::BoundariesAndSurface { wire_format } => f
                .debug_struct("Face")
                .field(
                    "boundaries",
                    &self
                        .entity
                        .boundaries()
                        .iter()
                        .map(|wire| wire.display(wire_format))
                        .collect::<Vec<_>>(),
                )
                .field("entity", &MutexFmt(&self.entity.surface))
                .finish(),
            FaceDisplayFormat::LoopsListTuple { wire_format } => f
                .debug_tuple("Face")
                .field(
                    &self
                        .entity
                        .boundaries()
                        .iter()
                        .map(|wire| wire.display(wire_format))
                        .collect::<Vec<_>>(),
                )
                .finish(),
            FaceDisplayFormat::LoopsList { wire_format } => f
                .debug_list()
                .entries(
                    self.entity
                        .boundaries()
                        .iter()
                        .map(|wire| wire.display(wire_format)),
                )
                .finish(),
            FaceDisplayFormat::AsSurface => {
                f.write_fmt(format_args!("{:?}", &MutexFmt(&self.entity.surface)))
            }
        }
    }
}

#[test]
fn invert_mapped_face() {
    let v = Vertex::news(&[0, 1, 2, 3, 4, 5, 6]);
    let wire0 = Wire::from(vec![
        Edge::new(&v[0], &v[1], 100),
        Edge::new(&v[1], &v[2], 200),
        Edge::new(&v[2], &v[3], 300),
        Edge::new(&v[3], &v[0], 400),
    ]);
    let wire1 = Wire::from(vec![
        Edge::new(&v[4], &v[5], 500),
        Edge::new(&v[6], &v[5], 600).inverse(),
        Edge::new(&v[6], &v[4], 700),
    ]);
    let face0 = Face::new(vec![wire0, wire1], 10000).inverse();
    let face1 = face0.mapped(
        &move |i: &usize| *i + 10,
        &move |j: &usize| *j + 1000,
        &move |k: &usize| *k + 100000,
    );

    assert_eq!(face0.get_surface() + 100000, face1.get_surface(),);
    assert_eq!(face0.orientation(), face1.orientation());
    let biters0 = face0.boundary_iters();
    let biters1 = face1.boundary_iters();
    for (biter0, biter1) in biters0.into_iter().zip(biters1) {
        for (edge0, edge1) in biter0.zip(biter1) {
            assert_eq!(edge0.front().get_point() + 10, edge1.front().get_point(),);
            assert_eq!(edge0.back().get_point() + 10, edge1.back().get_point(),);
            assert_eq!(edge0.get_curve() + 1000, edge1.get_curve(),);
        }
    }
}
