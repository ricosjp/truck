use crate::*;

impl<P> Vertex<P> {
    /// constructor
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v0 = Vertex::new(()); // a vertex whose geometry is the empty tuple.
    /// let v1 = Vertex::new(()); // another vertex
    /// let v2 = v0.clone(); // a cloned vertex
    /// assert_ne!(v0, v1);
    /// assert_eq!(v0, v2);
    /// ```
    #[inline(always)]
    pub fn new(point: P) -> Vertex<P> {
        Vertex {
            point: Arc::new(Mutex::new(point)),
        }
    }

    /// Creates `len` distinct vertices and return them by vector.
    /// # Examples
    /// ```
    /// use truck_topology::Vertex;
    /// let v = Vertex::news(&[(), (), ()]);
    /// assert_eq!(v.len(), 3);
    /// assert_ne!(v[0], v[2]);
    /// ```
    #[inline(always)]
    pub fn news(points: impl AsRef<[P]>) -> Vec<Vertex<P>>
    where P: Copy {
        points.as_ref().iter().map(|p| Vertex::new(*p)).collect()
    }

    /// Returns the point of vertex.
    #[inline(always)]
    pub fn get_point(&self) -> P
    where P: Clone {
        self.point.lock().unwrap().clone()
    }

    /// Sets the point of vertex.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v0 = Vertex::new(0);
    /// let v1 = v0.clone();
    ///
    /// // Two vertices have the same content.
    /// assert_eq!(v0.get_point(), 0);
    /// assert_eq!(v1.get_point(), 0);
    ///
    /// // set point
    /// v0.set_point(1);
    ///
    /// // The contents of two vertices are synchronized.
    /// assert_eq!(v0.get_point(), 1);
    /// assert_eq!(v1.get_point(), 1);
    /// ```
    #[inline(always)]
    pub fn set_point(&self, point: P) { *self.point.lock().unwrap() = point; }

    /// Returns vertex whose point is converted by `point_mapping`.
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    #[inline(always)]
    pub fn try_mapped<Q>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Option<Q>,
    ) -> Option<Vertex<Q>> {
        Some(Vertex::new(point_mapping(&*self.point.lock().unwrap())?))
    }

    /// Returns vertex whose point is converted by `point_mapping`.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v0 = Vertex::new(2);
    /// let v1 = v0.mapped(|a| *a as f64 + 0.5);
    /// assert_eq!(v1.get_point(), 2.5);
    /// ```
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    #[inline(always)]
    pub fn mapped<Q>(&self, mut point_mapping: impl FnMut(&P) -> Q) -> Vertex<Q> {
        Vertex::new(point_mapping(&*self.point.lock().unwrap()))
    }

    /// Returns the id of the vertex.
    #[inline(always)]
    pub fn id(&self) -> VertexID<P> { ID::new(Arc::as_ptr(&self.point)) }

    /// Returns how many same vertices.
    ///
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// // Create one vertex
    /// let v0 = Vertex::new(());
    /// assert_eq!(v0.count(), 1);
    /// // Create another vertex, independent from v0
    /// let v1 = Vertex::new(());
    /// assert_eq!(v0.count(), 1);
    /// // Clone v0, count will be 2
    /// let v2 = v0.clone();
    /// assert_eq!(v0.count(), 2);
    /// assert_eq!(v2.count(), 2);
    /// // drop v2, count will be 1
    /// drop(v2);
    /// assert_eq!(v0.count(), 1);
    /// ```
    #[inline(always)]
    pub fn count(&self) -> usize { Arc::strong_count(&self.point) }

    /// Create display struct for debugging the vertex.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use VertexDisplayFormat as VDF;
    /// let v = Vertex::new([0, 2]);
    /// assert_eq!(
    ///     format!("{:?}", v.display(VDF::Full)),
    ///     format!("Vertex {{ id: {:?}, entity: [0, 2] }}", v.id()),
    /// );
    /// assert_eq!(
    ///     format!("{:?}", v.display(VDF::IDTuple)),
    ///     format!("Vertex({:?})", v.id()),
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", v.display(VDF::PointTuple)),
    ///     "Vertex([0, 2])",
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", v.display(VDF::AsPoint)),
    ///     "[0, 2]",
    /// );
    /// ```
    #[inline(always)]
    pub fn display(&self, format: VertexDisplayFormat) -> DebugDisplay<Self, VertexDisplayFormat> {
        DebugDisplay {
            entity: self,
            format,
        }
    }
}

impl<P> Clone for Vertex<P> {
    #[inline(always)]
    fn clone(&self) -> Vertex<P> {
        Vertex {
            point: Arc::clone(&self.point),
        }
    }
}

impl<P> PartialEq for Vertex<P> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool { self.id() == other.id() }
}

impl<P> Eq for Vertex<P> {}

impl<P> Hash for Vertex<P> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) { std::ptr::hash(Arc::as_ptr(&self.point), state); }
}

impl<'a, P: Debug> Debug for DebugDisplay<'a, Vertex<P>, VertexDisplayFormat> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.format {
            VertexDisplayFormat::Full => f
                .debug_struct("Vertex")
                .field("id", &Arc::as_ptr(&self.entity.point))
                .field("entity", &MutexFmt(&self.entity.point))
                .finish(),
            VertexDisplayFormat::IDTuple => {
                f.debug_tuple("Vertex").field(&self.entity.id()).finish()
            }
            VertexDisplayFormat::PointTuple => f
                .debug_tuple("Vertex")
                .field(&MutexFmt(&self.entity.point))
                .finish(),
            VertexDisplayFormat::AsPoint => {
                f.write_fmt(format_args!("{:?}", &MutexFmt(&self.entity.point)))
            }
        }
    }
}
