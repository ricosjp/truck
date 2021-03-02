use crate::errors::Error;
use crate::*;

impl<P, C> Edge<P, C> {
    /// Generates the edge from `front` to `back`.  
    /// # Failures
    /// If `front == back`, then returns `Error::SameVertex`.
    /// ```
    /// # use truck_topology::*;
    /// # use truck_topology::errors::Error;
    /// let v = Vertex::news(&[(), ()]);
    /// assert!(Edge::try_new(&v[0], &v[1], ()).is_ok());
    /// assert_eq!(Edge::try_new(&v[0], &v[0], ()), Err(Error::SameVertex));
    /// ```
    #[inline(always)]
    pub fn try_new(front: &Vertex<P>, back: &Vertex<P>, curve: C) -> Result<Edge<P, C>> {
        if front == back {
            Err(Error::SameVertex)
        } else {
            Ok(Edge::new_unchecked(front, back, curve))
        }
    }
    /// Generates the edge from `front` to `back`.
    /// # Panic
    /// The condition `front == back` is not allowed.
    /// ```should_panic
    /// use truck_topology::*;
    /// let v = Vertex::new(());
    /// Edge::new(&v, &v, ()); // panic occurs
    /// ```
    #[inline(always)]
    pub fn new(front: &Vertex<P>, back: &Vertex<P>, curve: C) -> Edge<P, C> {
        Edge::try_new(front, back, curve).remove_try()
    }
    /// Generates the edge from `front` to `back`.
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.  
    /// This method does NOT check the condition `front == back`.  
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn new_unchecked(front: &Vertex<P>, back: &Vertex<P>, curve: C) -> Edge<P, C> {
        Edge {
            vertices: (front.clone(), back.clone()),
            orientation: true,
            curve: Arc::new(Mutex::new(curve)),
        }
    }

    /// Generates the edge from `front` to `back`.
    /// # Remarks
    /// This method check the condition `front == back` in the debug mode.  
    /// The programmer must guarantee this condition before using this method.
    #[inline(always)]
    pub fn debug_new(front: &Vertex<P>, back: &Vertex<P>, curve: C) -> Edge<P, C> {
        match cfg!(debug_assertions) {
            true => Edge::new(front, back, curve),
            false => Edge::new_unchecked(front, back, curve),
        }
    }

    /// Returns the orientation of the curve.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge0 = Edge::new(&v[0], &v[1], ());
    /// let edge1 = edge0.inverse();
    /// assert!(edge0.orientation());
    /// assert!(!edge1.orientation());
    /// ```
    #[inline(always)]
    pub fn orientation(&self) -> bool { self.orientation }

    /// Inverts the direction of edge.
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge = Edge::new(&v[0], &v[1], ());
    /// let mut inv_edge = edge.clone();
    /// inv_edge.invert();
    ///
    /// // Two edges are the same edge.
    /// edge.is_same(&inv_edge);
    ///
    /// // the front and back are exchanged.
    /// assert_eq!(edge.front(), inv_edge.back());
    /// assert_eq!(edge.back(), inv_edge.front());
    /// ```
    #[inline(always)]
    pub fn invert(&mut self) -> &mut Self {
        self.orientation = !self.orientation;
        self
    }

    /// Creates the inverse oriented edge.
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge = Edge::new(&v[0], &v[1], ());
    /// let inv_edge = edge.inverse();
    ///
    /// // Two edges are the same edge.
    /// assert!(edge.is_same(&inv_edge));
    ///
    /// // Two edges has the same id.
    /// assert_eq!(edge.id(), inv_edge.id());
    ///
    /// // the front and back are exchanged.
    /// assert_eq!(edge.front(), inv_edge.back());
    /// assert_eq!(edge.back(), inv_edge.front());
    /// ```
    #[inline(always)]
    pub fn inverse(&self) -> Edge<P, C> {
        let mut res = self.clone();
        res.invert();
        res
    }

    /// Returns the front vertex
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge = Edge::new(&v[0], &v[1], ());
    /// assert_eq!(edge.front(), &v[0]);
    /// ```
    #[inline(always)]
    pub fn front(&self) -> &Vertex<P> {
        match self.orientation {
            true => &self.vertices.0,
            false => &self.vertices.1,
        }
    }

    /// Returns the back vertex
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge = Edge::new(&v[0], &v[1], ());
    /// assert_eq!(edge.back(), &v[1]);
    /// ```
    #[inline(always)]
    pub fn back(&self) -> &Vertex<P> {
        match self.orientation {
            true => &self.vertices.1,
            false => &self.vertices.0,
        }
    }

    /// Returns the vertices at both ends.
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge = Edge::new(&v[0], &v[1], ());
    /// assert_eq!(edge.ends(), (&v[0], &v[1]));
    /// ```
    #[inline(always)]
    pub fn ends(&self) -> (&Vertex<P>, &Vertex<P>) {
        match self.orientation {
            true => (&self.vertices.0, &self.vertices.1),
            false => (&self.vertices.1, &self.vertices.0),
        }
    }

    /// Returns the front vertex which is generated by constructor
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge = Edge::new(&v[0], &v[1], ()).inverse();
    /// assert_eq!(edge.front(), &v[1]);
    /// assert_eq!(edge.absolute_front(), &v[0]);
    /// ```
    #[inline(always)]
    pub fn absolute_front(&self) -> &Vertex<P> { &self.vertices.0 }
    /// Returns the back vertex which is generated by constructor
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge = Edge::new(&v[0], &v[1], ()).inverse();
    /// assert_eq!(edge.back(), &v[0]);
    /// assert_eq!(edge.absolute_back(), &v[1]);
    /// ```
    #[inline(always)]
    pub fn absolute_back(&self) -> &Vertex<P> { &self.vertices.1 }

    /// Returns the vertices at both absolute ends.
    /// ```
    /// # use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let mut edge = Edge::new(&v[0], &v[1], ());
    /// edge.invert();
    /// assert_eq!(edge.ends(), (&v[1], &v[0]));
    /// assert_eq!(edge.absolute_ends(), (&v[0], &v[1]));
    /// ```
    #[inline(always)]
    pub fn absolute_ends(&self) -> (&Vertex<P>, &Vertex<P>) { (&self.vertices.0, &self.vertices.1) }

    /// Returns whether two edges are the same. Returns `true` even if the orientaions are different.
    /// ```
    /// use truck_topology::{Vertex, Edge};
    /// let v = Vertex::news(&[(), ()]);
    /// let edge0 = Edge::new(&v[0], &v[1], ());
    /// let edge1 = Edge::new(&v[0], &v[1], ());
    /// let edge2 = edge0.clone();
    /// let edge3 = edge0.inverse();
    /// assert!(!edge0.is_same(&edge1)); // edges whose ids are different are not the same.
    /// assert!(edge0.is_same(&edge2)); // The cloned edge is the same edge.
    /// assert!(edge0.is_same(&edge3)); // The inversed edge is the "same" edge
    /// ```
    #[inline(always)]
    pub fn is_same(&self, other: &Edge<P, C>) -> bool {
        std::ptr::eq(Arc::as_ptr(&self.curve), Arc::as_ptr(&other.curve))
    }

    /// Tries to lock the mutex of the contained curve.
    /// The thread will not blocked.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge0 = Edge::new(&v[0], &v[1], 0);
    /// let edge1 = edge0.clone();
    ///
    /// // Two vertices have the same content.
    /// assert_eq!(*edge0.try_lock_curve().unwrap(), 0);
    /// assert_eq!(*edge1.try_lock_curve().unwrap(), 0);
    ///
    /// {
    ///     let mut curve = edge0.try_lock_curve().unwrap();
    ///     *curve = 1;
    /// }
    /// // The contents of two vertices are synchronized.
    /// assert_eq!(*edge0.try_lock_curve().unwrap(), 1);
    /// assert_eq!(*edge1.try_lock_curve().unwrap(), 1);
    ///
    /// // The thread is not blocked even if the curve is already locked.
    /// let lock = edge0.try_lock_curve();
    /// assert!(edge1.try_lock_curve().is_err());    
    /// ```
    #[inline(always)]
    pub fn try_lock_curve(&self) -> TryLockResult<MutexGuard<C>> { self.curve.try_lock() }
    /// Acquires the mutex of the contained curve,
    /// blocking the current thread until it is able to do so.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge0 = Edge::new(&v[0], &v[1], 0);
    /// let edge1 = edge0.clone();
    ///
    /// // Two edges have the same content.
    /// assert_eq!(*edge0.lock_curve().unwrap(), 0);
    /// assert_eq!(*edge1.lock_curve().unwrap(), 0);
    ///
    /// {
    ///     let mut curve = edge0.lock_curve().unwrap();
    ///     *curve = 1;
    /// }
    /// // The contents of two edges are synchronized.
    /// assert_eq!(*edge0.lock_curve().unwrap(), 1);
    /// assert_eq!(*edge1.lock_curve().unwrap(), 1);
    ///
    /// // Check the behavior of `lock`.
    /// std::thread::spawn(move || {
    ///     *edge0.lock_curve().unwrap() = 2;
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*edge1.lock_curve().unwrap(), 2);    
    /// ```
    #[inline(always)]
    pub fn lock_curve(&self) -> LockResult<MutexGuard<C>> { self.curve.lock() }

    /// Returns the id that does not depend on the direction of the edge.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge0 = Edge::new(&v[0], &v[1], ());
    /// let edge1 = edge0.inverse();
    /// assert_ne!(edge0, edge1);
    /// assert_eq!(edge0.id(), edge1.id());
    /// ```
    #[inline(always)]
    pub fn id(&self) -> EdgeID<C> { ID::new(Arc::as_ptr(&self.curve)) }
}

impl<P, C: Clone + Invertible> Edge<P, C> {
    /// Returns the cloned curve in edge.
    /// If edge is inverted, then the returned curve is also inverted.
    #[inline(always)]
    pub fn oriented_curve(&self) -> C {
        match self.orientation {
            true => self.lock_curve().unwrap().clone(),
            false => self.lock_curve().unwrap().inverse(),
        }
    }
}

impl<P, C: Curve<Point = P>> Edge <P, C> {
    /// Returns the consistence of the geometry of end vertices
    /// and the geometry of edge.
    #[inline(always)]
    pub fn is_geometric_consistent(&self) -> bool where P: Tolerance {
        let curve = self.lock_curve().unwrap();
        let geom_front = curve.front();
        let geom_back = curve.back();
        let top_front = self.absolute_front().lock_point().unwrap();
        let top_back = self.absolute_back().lock_point().unwrap();
        geom_front.near(&*top_front) && geom_back.near(&*top_back)
    }
}

impl<P, C> Clone for Edge<P, C> {
    #[inline(always)]
    fn clone(&self) -> Edge<P, C> {
        Edge {
            vertices: self.vertices.clone(),
            orientation: self.orientation,
            curve: Arc::clone(&self.curve),
        }
    }
}

impl<P, C> PartialEq for Edge<P, C> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(Arc::as_ptr(&self.curve), Arc::as_ptr(&other.curve))
            && self.orientation == other.orientation
    }
}

impl<P, C> Eq for Edge<P, C> {}

impl<P, C> Hash for Edge<P, C> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.curve), state);
        self.orientation.hash(state);
    }
}
