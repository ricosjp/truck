use crate::errors::Error;
use crate::*;
use thiserror::Error;

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

    /// Returns a clone of the edge without inversion.
    /// # Examples
    /// ```
    /// use truck_topology::{Vertex, Edge};
    /// let v = Vertex::news(&[(), ()]);
    /// let edge0 = Edge::new(&v[0], &v[1], ());
    /// let edge1 = edge0.inverse();
    /// let edge2 = edge1.absolute_clone();
    /// assert_eq!(edge0, edge2);
    /// assert_ne!(edge1, edge2);
    /// assert!(edge1.is_same(&edge2));
    /// ```
    #[inline(always)]
    pub fn absolute_clone(&self) -> Self {
        Self {
            vertices: self.vertices.clone(),
            curve: Arc::clone(&self.curve),
            orientation: true,
        }
    }

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
    pub fn is_same(&self, other: &Edge<P, C>) -> bool { self.id() == other.id() }

    /// Returns the clone of the curve.
    /// # Remarks
    /// This method returns absolute curve i.e. does not consider the orientation of curve.
    /// If you want to get a curve compatible with edge's orientation, use `Edge::oriented_curve`.
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[0, 1]);
    /// let mut edge = Edge::new(&v[0], &v[1], (0, 1));
    /// edge.invert();
    ///
    /// // absolute curve
    /// assert_eq!(edge.get_curve(), (0, 1));
    /// // oriented curve
    /// assert_eq!(edge.oriented_curve(), (1, 0));
    /// ```
    #[inline(always)]
    pub fn get_curve(&self) -> C
    where C: Clone {
        self.curve.lock().unwrap().clone()
    }

    /// Set the curve.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v = Vertex::news(&[(), ()]);
    /// let edge0 = Edge::new(&v[0], &v[1], 0);
    /// let edge1 = edge0.clone();
    ///
    /// // Two edges have the same content.
    /// assert_eq!(edge0.get_curve(), 0);
    /// assert_eq!(edge1.get_curve(), 0);
    ///
    /// // set the content
    /// edge0.set_curve(1);
    ///
    /// // The contents of two edges are synchronized.
    /// assert_eq!(edge0.get_curve(), 1);
    /// assert_eq!(edge1.get_curve(), 1);
    /// ```
    #[inline(always)]
    pub fn set_curve(&self, curve: C) { *self.curve.lock().unwrap() = curve; }

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
    /// Returns the cloned curve in edge.
    /// If edge is inverted, then the returned curve is also inverted.
    #[inline(always)]
    pub fn oriented_curve(&self) -> C
    where C: Clone + Invertible {
        match self.orientation {
            true => self.curve.lock().unwrap().clone(),
            false => self.curve.lock().unwrap().inverse(),
        }
    }

    /// Returns a new edge whose curve is mapped by `curve_mapping` and
    /// whose end points are mapped by `point_mapping`.
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    #[inline(always)]
    pub fn try_mapped<Q, D>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Option<Q>,
        mut curve_mapping: impl FnMut(&C) -> Option<D>,
    ) -> Option<Edge<Q, D>> {
        let v0 = self.absolute_front().try_mapped(&mut point_mapping)?;
        let v1 = self.absolute_back().try_mapped(&mut point_mapping)?;
        let curve = curve_mapping(&*self.curve.lock().unwrap())?;
        let mut edge = Edge::debug_new(&v0, &v1, curve);
        if !self.orientation() {
            edge.invert();
        }
        Some(edge)
    }

    /// Returns a new edge whose curve is mapped by `curve_mapping` and
    /// whose end points are mapped by `point_mapping`.
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// let v0 = Vertex::new(0);
    /// let v1 = Vertex::new(1);
    /// let edge0 = Edge::new(&v0, &v1, 2);
    /// let edge1 = edge0.mapped(
    ///     &move |i: &usize| *i as f64 + 0.5,
    ///     &move |j: &usize| *j as f64 + 0.5,
    /// );
    ///
    /// assert_eq!(edge1.front().get_point(), 0.5);
    /// assert_eq!(edge1.back().get_point(), 1.5);
    /// assert_eq!(edge1.get_curve(), 2.5);
    /// ```
    /// # Remarks
    /// Accessing geometry elements directly in the closure will result in a deadlock.
    /// So, this method does not appear to the document.
    #[doc(hidden)]
    #[inline(always)]
    pub fn mapped<Q, D>(
        &self,
        mut point_mapping: impl FnMut(&P) -> Q,
        mut curve_mapping: impl FnMut(&C) -> D,
    ) -> Edge<Q, D> {
        let v0 = self.absolute_front().mapped(&mut point_mapping);
        let v1 = self.absolute_back().mapped(&mut point_mapping);
        let curve = curve_mapping(&*self.curve.lock().unwrap());
        let mut edge = Edge::debug_new(&v0, &v1, curve);
        if edge.orientation() != self.orientation() {
            edge.invert();
        }
        edge
    }

    /// Returns the consistence of the geometry of end vertices
    /// and the geometry of edge.
    #[inline(always)]
    pub fn is_geometric_consistent(&self) -> bool
    where
        P: Tolerance,
        C: ParametricCurve<Point = P>, {
        let curve = self.curve.lock().unwrap();
        let geom_front = curve.front();
        let geom_back = curve.back();
        let top_front = self.absolute_front().point.lock().unwrap();
        let top_back = self.absolute_back().point.lock().unwrap();
        geom_front.near(&*top_front) && geom_back.near(&*top_back)
    }

    /// Cuts the edge at `vertex`.
    /// # Failure
    /// Returns `None` if:
    /// - cannot find the parameter `t` such that `edge.get_curve().subs(t) == vertex.get_point()`, or
    /// - the found parameter is not in the parameter range without end points.
    pub fn cut(&self, vertex: &Vertex<P>) -> Option<(Self, Self)>
    where
        P: Clone,
        C: Cut<Point = P> + SearchParameter<Point = P, Parameter = f64>, {
        let mut curve0 = self.get_curve();
        let t = curve0.search_parameter(vertex.get_point(), None, SEARCH_PARAMETER_TRIALS)?;
        let (t0, t1) = curve0.parameter_range();
        if t < t0 + TOLERANCE || t1 - TOLERANCE < t {
            return None;
        }
        let curve1 = curve0.cut(t);
        let edge0 = Edge {
            vertices: (self.absolute_front().clone(), vertex.clone()),
            orientation: self.orientation,
            curve: Arc::new(Mutex::new(curve0)),
        };
        let edge1 = Edge {
            vertices: (vertex.clone(), self.absolute_back().clone()),
            orientation: self.orientation,
            curve: Arc::new(Mutex::new(curve1)),
        };
        if self.orientation {
            Some((edge0, edge1))
        } else {
            Some((edge1, edge0))
        }
    }

    /// Cuts the edge at `vertex` with parameter `t`.
    /// # Failure
    /// Returns `None` if `!edge.get_curve().subs(t).near(&vertex.get_point())`.
    pub fn cut_with_parameter(&self, vertex: &Vertex<P>, t: f64) -> Option<(Self, Self)>
    where
        P: Clone + Tolerance,
        C: Cut<Point = P>, {
        let mut curve0 = self.get_curve();
        if !curve0.subs(t).near(&vertex.get_point()) {
            return None;
        }
        let (t0, t1) = curve0.parameter_range();
        if t < t0 + TOLERANCE || t1 - TOLERANCE < t {
            return None;
        }
        let curve1 = curve0.cut(t);
        let edge0 = Edge {
            vertices: (self.absolute_front().clone(), vertex.clone()),
            orientation: self.orientation,
            curve: Arc::new(Mutex::new(curve0)),
        };
        let edge1 = Edge {
            vertices: (vertex.clone(), self.absolute_back().clone()),
            orientation: self.orientation,
            curve: Arc::new(Mutex::new(curve1)),
        };
        if self.orientation {
            Some((edge0, edge1))
        } else {
            Some((edge1, edge0))
        }
    }

    /// Concats two edges.
    pub fn concat(&self, rhs: &Self) -> std::result::Result<Self, ConcatError<P>>
    where
        P: Debug,
        C: Concat<C, Point = P, Output = C> + Invertible + ParameterTransform, {
        if self.back() != rhs.front() {
            return Err(ConcatError::DisconnectedVertex(
                self.back().clone(),
                rhs.front().clone(),
            ));
        }
        if self.front() == rhs.back() {
            return Err(ConcatError::SameVertex(self.front().clone()));
        }
        let curve0 = self.oriented_curve();
        let mut curve1 = rhs.oriented_curve();
        let t0 = curve0.parameter_range().1;
        let t1 = curve1.parameter_range().0;
        curve1.parameter_transform(1.0, t0 - t1);
        let curve = curve0.try_concat(&curve1)?;
        Ok(Edge::debug_new(self.front(), rhs.back(), curve))
    }

    /// Create display struct for debugging the edge.
    /// 
    /// # Examples
    /// ```
    /// use truck_topology::*;
    /// use EdgeDisplayFormat as EDF;
    /// 
    /// let vertex_format = VertexDisplayFormat::AsPoint;
    /// let edge = Edge::new(&Vertex::new(0), &Vertex::new(1), 2);
    /// 
    /// assert_eq!(
    ///     format!("{:?}", edge.display(EDF::Full { vertex_format })),
    ///     format!("Edge {{ id: {:?}, vertices: (0, 1), entity: 2 }}", edge.id()),
    /// );
    /// assert_eq!(
    ///     format!("{:?}", edge.display(EDF::VerticesTupleAndID { vertex_format })),
    ///     format!("Edge {{ id: {:?}, vertices: (0, 1) }}", edge.id()),
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", edge.display(EDF::VerticesTupleAndCurve { vertex_format })),
    ///     "Edge { vertices: (0, 1), entity: 2 }",
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", edge.display(EDF::VerticesTupleStruct { vertex_format })),
    ///     "Edge(0, 1)",
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", edge.display(EDF::VerticesTuple { vertex_format })),
    ///     "(0, 1)",
    /// );
    /// assert_eq!(
    ///     &format!("{:?}", edge.display(EDF::AsCurve)),
    ///     "2",
    /// );
    /// ```
    #[inline(always)]
    pub fn display(&self, format: EdgeDisplayFormat) -> EdgeDisplay<P, C> {
        EdgeDisplay {
            edge: self,
            format,
        }
    }
}

/// Error for concat
#[derive(Clone, Debug, Error)]
pub enum ConcatError<P: Debug> {
    /// Failed to concat edges since the end point of the first curve is different from the start point of the second curve.
    #[error("The end point {0:?} of the first curve is different from the start point {1:?} of the second curve.")]
    DisconnectedVertex(Vertex<P>, Vertex<P>),
    #[error("The end vertices are the same vertex {0:?}.")]
    SameVertex(Vertex<P>),
    /// From geometric error.
    #[error("{0}")]
    FromGeometry(truck_geotrait::ConcatError<P>),
}

impl<P: Debug> From<truck_geotrait::ConcatError<P>> for ConcatError<P> {
    fn from(err: truck_geotrait::ConcatError<P>) -> ConcatError<P> {
        ConcatError::FromGeometry(err)
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

/// Display struct for debugging the edge
#[derive(Clone, Copy)]
pub struct EdgeDisplay<'a, P, C> {
    edge: &'a Edge<P, C>,
    format: EdgeDisplayFormat,
}

impl<'a, P: Debug, C: Debug> Debug for EdgeDisplay<'a, P, C> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.format {
            EdgeDisplayFormat::Full { vertex_format } => f
                .debug_struct("Edge")
                .field("id", &Arc::as_ptr(&self.edge.curve))
                .field(
                    "vertices",
                    &(
                        self.edge.front().display(vertex_format),
                        self.edge.back().display(vertex_format),
                    ),
                )
                .field("entity", &MutexFmt(&self.edge.curve))
                .finish(),
            EdgeDisplayFormat::VerticesTupleAndID { vertex_format } => f
                .debug_struct("Edge")
                .field("id", &Arc::as_ptr(&self.edge.curve))
                .field(
                    "vertices",
                    &(
                        self.edge.front().display(vertex_format),
                        self.edge.back().display(vertex_format),
                    ),
                )
                .finish(),
            EdgeDisplayFormat::VerticesTupleAndCurve { vertex_format } => f
                .debug_struct("Edge")
                .field(
                    "vertices",
                    &(
                        self.edge.front().display(vertex_format),
                        self.edge.back().display(vertex_format),
                    ),
                )
                .field("entity", &MutexFmt(&self.edge.curve))
                .finish(),
            EdgeDisplayFormat::VerticesTupleStruct { vertex_format } => f
                .debug_tuple("Edge")
                .field(&self.edge.front().display(vertex_format))
                .field(&self.edge.back().display(vertex_format))
                .finish(),
            EdgeDisplayFormat::VerticesTuple { vertex_format } => f.write_fmt(format_args!(
                "({:?}, {:?})",
                self.edge.front().display(vertex_format),
                self.edge.back().display(vertex_format),
            )),
            EdgeDisplayFormat::AsCurve => {
                f.write_fmt(format_args!("{:?}", &MutexFmt(&self.edge.curve)))
            }
        }
    }
}
