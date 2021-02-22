use crate::bspsurface::{CPColumnIter, CPRowIter};
use crate::*;

impl<V> NURBSSurface<V> {
    /// constructor
    #[inline(always)]
    pub const fn new(bspsurface: BSplineSurface<V>) -> Self { NURBSSurface(bspsurface) }

    /// Returns the nurbs surface before rationalized
    #[inline(always)]
    pub fn non_rationalized(&self) -> &BSplineSurface<V> { &self.0 }
    /// Returns the nurbs surface before rationalized
    #[inline(always)]
    pub fn non_rationalized_mut(&mut self) -> &mut BSplineSurface<V> { &mut self.0 }

    /// Returns the nurbs surface before rationalized
    #[inline(always)]
    pub fn into_non_rationalized(self) -> BSplineSurface<V> { self.0 }

    /// Returns the reference of the knot vectors
    #[inline(always)]
    pub fn knot_vecs(&self) -> &(KnotVec, KnotVec) { &self.0.knot_vecs }

    /// Returns the u knot vector.
    #[inline(always)]
    pub fn uknot_vec(&self) -> &KnotVec { &self.0.knot_vecs.0 }
    /// Returns the v knot vector.
    #[inline(always)]
    pub fn vknot_vec(&self) -> &KnotVec { &self.0.knot_vecs.1 }

    /// Returns the `idx`th u knot.
    #[inline(always)]
    pub fn uknot(&self, idx: usize) -> f64 { self.0.knot_vecs.0[idx] }
    /// returns the `idx`th v knot.
    #[inline(always)]
    pub fn vknot(&self, idx: usize) -> f64 { self.0.knot_vecs.1[idx] }

    /// Returns the reference of the vector of the control points
    #[inline(always)]
    pub fn control_points(&self) -> &Vec<Vec<V>> { &self.0.control_points }

    /// Returns the reference of the control point corresponding to the index `(idx0, idx1)`.
    #[inline(always)]
    pub fn control_point(&self, idx0: usize, idx1: usize) -> &V {
        &self.0.control_points[idx0][idx1]
    }

    /// Apply the given transformation to all control points.
    #[inline(always)]
    pub fn transform_control_points<F: FnMut(&mut V)>(&mut self, f: F) {
        self.0.transform_control_points(f)
    }

    /// Returns the iterator over the control points in the `column_idx`th row.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::bezier_knot(1);
    /// let vknot_vec = KnotVec::bezier_knot(2);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let mut iter = bspsurface.ctrl_pts_row_iter(1);
    /// assert_eq!(iter.next(), Some(&Vector3::new(1.0, 0.0, 1.0)));
    /// assert_eq!(iter.next(), Some(&Vector3::new(1.0, 1.0, 1.0)));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline(always)]
    pub fn ctrl_pts_row_iter(&self, column_idx: usize) -> CPRowIter<'_, V> {
        self.0.ctrl_pts_row_iter(column_idx)
    }

    /// Returns the iterator over the control points in the `row_idx`th row.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::bezier_knot(1);
    /// let vknot_vec = KnotVec::bezier_knot(2);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let mut iter = bspsurface.ctrl_pts_column_iter(1);
    /// assert_eq!(iter.next(), Some(&Vector3::new(0.0, 1.0, 0.0)));
    /// assert_eq!(iter.next(), Some(&Vector3::new(1.0, 1.0, 1.0)));
    /// assert_eq!(iter.next(), Some(&Vector3::new(2.0, 1.0, 2.0)));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline(always)]
    pub fn ctrl_pts_column_iter(&self, row_idx: usize) -> CPColumnIter<'_, V> {
        self.0.control_points[row_idx].iter()
    }

    /// Returns the mutable reference of the control point corresponding to index `(idx0, idx1)`.
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx0: usize, idx1: usize) -> &mut V {
        &mut self.0.control_points[idx0][idx1]
    }

    /// Returns the iterator on all control points
    #[inline(always)]
    pub fn control_points_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.0.control_points.iter_mut().flatten()
    }
    /// Returns the degrees of B-spline surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let vknot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// assert_eq!(bspsurface.udegree(), 1);
    /// ```
    #[inline(always)]
    pub fn udegree(&self) -> usize { self.0.udegree() }

    /// Returns the degrees of B-spline surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let vknot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// assert_eq!(bspsurface.vdegree(), 2);
    /// ```
    #[inline(always)]
    pub fn vdegree(&self) -> usize { self.0.vdegree() }

    /// Returns the degrees of B-spline surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let vknot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// assert_eq!(bspsurface.degrees(), (1, 2));
    /// ```
    #[inline(always)]
    pub fn degrees(&self) -> (usize, usize) { (self.udegree(), self.vdegree()) }
    /// Returns whether the knot vectors are clamped or not.
    #[inline(always)]
    pub fn is_clamped(&self) -> bool { self.0.is_clamped() }
    /// Swaps two parameters.
    pub fn swap_axes(&mut self) -> &mut Self
    where V: Clone {
        self.0.swap_axes();
        self
    }
    /// The range of the parameter of the surface.
    #[inline(always)]
    pub fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { self.0.parameter_range() }
    /// Creates the curve whose control points are the `idx`th column control points of `self`.
    #[inline(always)]
    pub fn column_curve(&self, row_idx: usize) -> NURBSCurve<V>
    where V: Clone {
        NURBSCurve(self.0.column_curve(row_idx))
    }
    /// Creates the column sectional curve.
    #[inline(always)]
    pub fn row_curve(&self, column_idx: usize) -> NURBSCurve<V>
    where V: Clone {
        NURBSCurve(self.0.row_curve(column_idx))
    }
}

impl<V: Homogeneous<f64>> NURBSSurface<V> {
    /// Substitutes to a NURBS surface.
    #[inline(always)]
    pub fn subs(&self, u: f64, v: f64) -> V::Point { self.0.subs(u, v).to_point() }
    /// Substitutes derived NURBS surface by the first parameter `u`.
    #[inline(always)]
    pub fn uder(&self, u: f64, v: f64) -> <V::Point as EuclideanSpace>::Diff {
        let pt = self.0.subs(u, v);
        let ud = self.0.uder(u, v);
        pt.rat_der(ud)
    }
    /// Substitutes derived NURBS surface by the first parameter `v`.
    #[inline(always)]
    pub fn vder(&self, u: f64, v: f64) -> <V::Point as EuclideanSpace>::Diff {
        let pt = self.0.subs(u, v);
        let vd = self.0.vder(u, v);
        pt.rat_der(vd)
    }
    /// Returns the closure of substitution.
    #[inline(always)]
    pub fn get_closure(&self) -> impl Fn(f64, f64) -> V::Point + '_ { move |u, v| self.subs(u, v) }
}

impl<V: Homogeneous<f64>> NURBSSurface<V>
where V::Point: Tolerance
{
    /// Returns whether constant curve or not, i.e. all control points are same or not.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::bezier_knot(1);
    /// let vknot_vec = KnotVec::bezier_knot(2);
    /// let pt = Vector3::new(1.0, 2.0, 1.0);
    /// // allows differences upto scalars
    /// let ctrl_pts = vec![
    ///     vec![pt.clone(), pt.clone() * 2.0, pt.clone() * 3.0],
    ///     vec![pt.clone() * 0.5, pt.clone() * 0.25, pt.clone() * 0.125],
    /// ];
    /// let mut surface = NURBSSurface::new(BSplineSurface::new((uknot_vec, vknot_vec), ctrl_pts));
    /// assert!(surface.is_const());
    ///
    /// *surface.control_point_mut(1, 2) = Vector3::new(2.0, 3.0, 1.0);
    /// assert!(!surface.is_const());
    /// ```
    #[inline(always)]
    pub fn is_const(&self) -> bool {
        let pt = self.0.control_points[0][0].to_point();
        for vec in self.0.control_points.iter().flat_map(|pts| pts.iter()) {
            if !vec.to_point().near(&pt) {
                return false;
            }
        }
        true
    }
    /// Determines whether `self` and `other` is near as the B-spline rational surfaces or not.  
    ///
    /// Divides each knot domain into the number of degree equal parts,
    /// and check `|self(u, v) - other(u, v)| < TOLERANCE` for each end points `(u, v)`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(3), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.5, -1.0, 2.0), Vector3::new(1.0, 0.0, 1.0)],
    ///     vec![Vector3::new(0.0, 1.0, 1.0), Vector3::new(0.5, 1.0, 1.0), Vector3::new(1.0, 1.0, 1.0)],
    ///     vec![Vector3::new(0.0, 2.0, 1.0), Vector3::new(0.5, 2.0, 3.0), Vector3::new(1.0, 2.0, 1.0)],
    ///     vec![Vector3::new(0.0, 3.0, 1.0), Vector3::new(0.5, 3.5, 2.0), Vector3::new(1.0, 3.0, 1.0)],
    /// ];
    /// let surface0 = NURBSSurface::new(BSplineSurface::new(knot_vecs, ctrl_pts));
    /// let mut surface1 = surface0.clone();
    /// assert!(surface0.near_as_surface(&surface1));
    ///
    /// *surface1.control_point_mut(1, 1) = Vector3::new(0.5, 1.0, 0.9);
    /// assert!(!surface0.near_as_surface(&surface1));
    /// ```
    #[inline(always)]
    pub fn near_as_surface(&self, other: &Self) -> bool {
        self.0
            .sub_near_as_surface(&other.0, 2, move |x, y| x.to_point().near(&y.to_point()))
    }
    /// Determines whether `self` and `other` is near in square order as the B-spline rational
    /// surfaces or not.  
    ///
    /// Divides each knot domain into the number of degree equal parts,
    /// and check `|self(u, v) - other(u, v)| < TOLERANCE` for each end points `(u, v)`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let eps = TOLERANCE;
    /// let knot_vecs = (KnotVec::bezier_knot(3), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.5, -1.0, 2.0), Vector3::new(1.0, 0.0, 1.0)],
    ///     vec![Vector3::new(0.0, 1.0, 1.0), Vector3::new(0.5, 1.0, 1.0), Vector3::new(1.0, 1.0, 1.0)],
    ///     vec![Vector3::new(0.0, 2.0, 1.0), Vector3::new(0.5, 2.0, 3.0), Vector3::new(1.0, 2.0, 1.0)],
    ///     vec![Vector3::new(0.0, 3.0, 1.0), Vector3::new(0.5, 3.5, 2.0), Vector3::new(1.0, 3.0, 1.0)],
    /// ];
    /// let surface0 = NURBSSurface::new(BSplineSurface::new(knot_vecs, ctrl_pts));
    /// let mut surface1 = surface0.clone();
    /// assert!(surface0.near_as_surface(&surface1));
    ///
    /// *surface1.control_point_mut(1, 1) = Vector3::new(0.5, 1.0, 1.0 - eps);
    /// assert!(surface0.near_as_surface(&surface1));
    /// assert!(!surface0.near2_as_surface(&surface1));
    /// ```
    #[inline(always)]
    pub fn near2_as_surface(&self, other: &Self) -> bool {
        self.0
            .sub_near_as_surface(&other.0, 2, move |x, y| x.to_point().near2(&y.to_point()))
    }
}

impl<V: Homogeneous<f64> + Tolerance> NURBSSurface<V> {
    /// Adds a knot `x` of the first parameter `u`, and do not change `self` as a surface.  
    #[inline(always)]
    pub fn add_uknot(&mut self, x: f64) -> &mut Self {
        self.0.add_uknot(x);
        self
    }
    /// Adds a knot `x` of the first parameter `u`, and do not change `self` as a surface.  
    #[inline(always)]
    pub fn add_vknot(&mut self, x: f64) -> &mut Self {
        self.0.add_vknot(x);
        self
    }
    /// Removes the uknot corresponding to the indice `idx`, and do not change `self` as a curve.  
    /// If the knot cannot be removed, returns
    /// [`Error::CannotRemoveKnot`](./errors/enum.Error.html#variant.CannotRemoveKnot).
    #[inline(always)]
    pub fn try_remove_uknot(&mut self, idx: usize) -> Result<&mut Self> {
        match self.0.try_remove_uknot(idx) {
            Ok(_) => Ok(self),
            Err(error) => Err(error),
        }
    }
    /// Removes the uknot corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    #[inline(always)]
    pub fn remove_uknot(&mut self, idx: usize) -> &mut Self {
        self.0.remove_uknot(idx);
        self
    }
    /// Removes the uknot corresponding to the indice `idx`, and do not change `self` as a curve.  
    /// If the knot cannot be removed, returns
    /// [`Error::CannotRemoveKnot`](./errors/enum.Error.html#variant.CannotRemoveKnot).
    #[inline(always)]
    pub fn try_remove_vknot(&mut self, idx: usize) -> Result<&mut Self> {
        match self.0.try_remove_vknot(idx) {
            Ok(_) => Ok(self),
            Err(error) => Err(error),
        }
    }
    /// Removes the uknot corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    #[inline(always)]
    pub fn remove_vknot(&mut self, idx: usize) -> &mut Self {
        self.0.remove_vknot(idx);
        self
    }
    /// Elevates the udegree.
    #[inline(always)]
    pub fn elevate_udegree(&mut self) -> &mut Self {
        self.0.elevate_udegree();
        self
    }
    /// Elevates the vdegree.
    #[inline(always)]
    pub fn elevate_vdegree(&mut self) -> &mut Self {
        self.0.elevate_vdegree();
        self
    }
    /// Aligns the udegree with the same degrees.
    #[inline(always)]
    pub fn syncro_uvdegrees(&mut self) -> &mut Self {
        self.0.syncro_uvdegrees();
        self
    }
    /// Makes the uknot vector and the vknot vector the same knot vector.
    #[inline(always)]
    pub fn syncro_uvknots(&mut self) -> &mut Self {
        self.0.syncro_uvknots();
        self
    }

    /// Cuts the surface into two surfaces at the parameter `u`
    #[inline(always)]
    pub fn ucut(&mut self, u: f64) -> Self { Self::new(self.0.ucut(u)) }

    /// Cuts the surface into two surfaces at the parameter `v`
    #[inline(always)]
    pub fn vcut(&mut self, v: f64) -> Self { Self::new(self.0.vcut(v)) }

    /// Normalizes the knot vectors
    #[inline(always)]
    pub fn knot_normalize(&mut self) -> &mut Self {
        self.0.knot_normalize();
        self
    }
    /// Translates the knot vectors.
    #[inline(always)]
    pub fn knot_translate(&mut self, x: f64, y: f64) -> &mut Self {
        self.0.knot_translate(x, y);
        self
    }

    /// Removes knots in order from the back
    #[inline(always)]
    pub fn optimize(&mut self) -> &mut Self {
        self.0.optimize();
        self
    }

    /// Get the boundary by four splitted curves.
    /// # Example
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(3), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.5, -1.0, 2.0), Vector3::new(1.0, 0.0, 1.0)],
    ///     vec![Vector3::new(0.0, 1.0, 2.0), Vector3::new(0.5, 1.0, 3.0), Vector3::new(1.0, 1.0, 2.0)],
    ///     vec![Vector3::new(0.0, 2.0, 2.0), Vector3::new(0.5, 2.0, 3.0), Vector3::new(1.0, 2.0, 2.0)],
    ///     vec![Vector3::new(0.0, 3.0, 1.0), Vector3::new(0.5, 3.5, 2.0), Vector3::new(1.0, 3.0, 1.0)],
    /// ];
    /// let bspsurface = NURBSSurface::new(BSplineSurface::new(knot_vecs, ctrl_pts));
    /// let curves = bspsurface.splitted_boundary();
    /// assert_eq!(
    ///     curves[0].control_points(),
    ///     &vec![
    ///         Vector3::new(0.0, 0.0, 1.0),
    ///         Vector3::new(0.0, 1.0, 2.0),
    ///         Vector3::new(0.0, 2.0, 2.0),
    ///         Vector3::new(0.0, 3.0, 1.0),
    ///     ],
    /// );
    /// assert_eq!(
    ///     curves[1].control_points(),
    ///     &vec![
    ///         Vector3::new(0.0, 3.0, 1.0),
    ///         Vector3::new(0.5, 3.5, 2.0),
    ///         Vector3::new(1.0, 3.0, 1.0),
    ///     ],
    /// );
    /// assert_eq!(
    ///     curves[2].control_points(),
    ///     &vec![
    ///         Vector3::new(1.0, 3.0, 1.0),
    ///         Vector3::new(1.0, 2.0, 2.0),
    ///         Vector3::new(1.0, 1.0, 2.0),
    ///         Vector3::new(1.0, 0.0, 1.0),
    ///     ],
    /// );
    /// assert_eq!(
    ///     curves[3].control_points(),
    ///     &vec![
    ///         Vector3::new(1.0, 0.0, 1.0),
    ///         Vector3::new(0.5, -1.0, 2.0),
    ///         Vector3::new(0.0, 0.0, 1.0),
    ///     ],
    /// );
    /// ```
    #[inline(always)]
    pub fn splitted_boundary(&self) -> [NURBSCurve<V>; 4] {
        std::convert::TryFrom::try_from(
            self.0
                .splitted_boundary()
                .to_vec()
                .into_iter()
                .map(|curve| NURBSCurve::new(curve))
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    /// Extracts the boundary of surface
    #[inline(always)]
    pub fn boundary(&self) -> NURBSCurve<V> { NURBSCurve::new(self.0.boundary()) }

    //--------------------------------------------------------------//
    //------------------------- WIP --------------------------------//
    //--------------------------------------------------------------//
}

impl<V: Homogeneous<f64>> NURBSSurface<V>
where <V::Point as EuclideanSpace>::Diff: InnerSpace
{
    /// Searches the parameter `(u, v)` which minimize `|self(u, v) - point|` by Newton's method
    /// with initial guess `(u0, v0)`. If the repeated trial does not converge, then returns `None`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(3), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, -2.0, 2.0), Vector3::new(1.0, 0.0, 1.0)],
    ///     vec![Vector3::new(0.0, 2.0, 2.0), Vector3::new(2.0, 4.0, 4.0), Vector3::new(2.0, 2.0, 2.0)],
    ///     vec![Vector3::new(0.0, 4.0, 2.0), Vector3::new(2.0, 8.0, 4.0), Vector3::new(2.0, 4.0, 2.0)],
    ///     vec![Vector3::new(0.0, 3.0, 1.0), Vector3::new(1.0, 7.0, 2.0), Vector3::new(1.0, 3.0, 1.0)],
    /// ];
    /// let surface = NURBSSurface::new(BSplineSurface::new(knot_vecs, ctrl_pts));
    /// let pt = surface.subs(0.3, 0.7);
    /// let (u, v) = surface.search_nearest_parameter(pt, (0.5, 0.5)).unwrap();
    /// assert!(u.near2(&0.3) && v.near2(&0.7));
    /// ```
    /// # Remarks
    /// It may converge to a local solution depending on the hint.
    /// cf. [`BSplineCurve::search_rational_nearest_parameter`](struct.BSplineCurve.html#method.search_rational_nearest_parameter)
    pub fn search_nearest_parameter(
        &self,
        pt: V::Point,
        (u0, v0): (f64, f64),
    ) -> Option<(f64, f64)> {
        self.0.search_rational_nearest_parameter(pt, (u0, v0))
    }
}

impl<V> NURBSSurface<V>
where
    V: Homogeneous<f64>,
    V::Point:
        MetricSpace<Metric = f64> + std::ops::Index<usize, Output = f64> + Bounded<f64> + Copy,
{
    /// Returns the bounding box including all control points.
    #[inline(always)]
    pub fn roughly_bounding_box(&self) -> BoundingBox<V::Point> {
        self.0
            .control_points
            .iter()
            .flatten()
            .map(|pt| pt.to_point())
            .collect()
    }
}

impl NURBSSurface<Vector3> {
    /// Serach the parameter `(u, v)` such that `self.subs(u, v).rational_projection()` is near `pt`.
    /// If cannot find, then return `None`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(2, 2);
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.1, 0.0, 0.5), Vector3::new(0.5, 0.0, 0.3), Vector3::new(1.0, 0.0, 1.0)],
    ///     vec![Vector3::new(0.0, 0.1, 0.1), Vector3::new(0.2, 0.2, 0.1), Vector3::new(0.4, 0.3, 0.4), Vector3::new(1.0, 0.3, 0.7)],
    ///     vec![Vector3::new(0.0, 0.5, 0.4), Vector3::new(0.3, 0.6, 0.5), Vector3::new(0.6, 0.4, 1.0), Vector3::new(1.0, 0.5, 0.4)],
    ///     vec![Vector3::new(0.0, 1.0, 1.0), Vector3::new(0.1, 1.0, 1.0), Vector3::new(0.5, 1.0, 0.5), Vector3::new(1.0, 1.0, 0.3)],
    /// ];
    /// let bspsurface = BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts);
    /// let surface = NURBSSurface::new(bspsurface);
    ///
    /// let pt = surface.subs(0.3, 0.7);
    /// let (u, v) = surface.search_parameter(pt, (0.5, 0.5)).unwrap();
    /// Point2::assert_near(&surface.subs(u, v), &pt);
    /// ```
    #[inline(always)]
    pub fn search_parameter(&self, pt: Point2, hint: (f64, f64)) -> Option<(f64, f64)> {
        bspsurface::sub_search_parameter2d(self, pt, hint.into(), 0).map(|v| v.into())
    }
}

impl<V: Homogeneous<f64>> ParameterDivision2D for NURBSSurface<V>
where V::Point: MetricSpace<Metric = f64>
{
    #[inline(always)]
    fn parameter_division(&self, tol: f64) -> (Vec<f64>, Vec<f64>) {
        self.0
            .create_space_division(tol, |v0, v1| v0.to_point().distance2(v1.to_point()))
    }
}

impl Surface for NURBSSurface<Vector3> {
    type Point = Point2;
    type Vector = Vector2;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point { self.subs(u, v) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector { self.uder(u, v) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Self::Vector { self.vder(u, v) }
    /// zero identity
    #[inline(always)]
    fn normal(&self, _: f64, _: f64) -> Self::Vector { Vector2::zero() }
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { self.parameter_range() }
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut surface = self.clone();
        surface.swap_axes();
        surface
    }
}

impl IncludeCurve<NURBSCurve<Vector3>> for NURBSSurface<Vector3> {
    #[inline(always)]
    fn include(&self, curve: &NURBSCurve<Vector3>) -> bool {
        let pt = curve.subs(curve.knot_vec()[0]);
        let mut hint = self.presearch(pt);
        hint = match self.search_parameter(pt, hint) {
            Some(got) => got,
            None => return false,
        };
        let uknot_vec = self.uknot_vec();
        let vknot_vec = self.vknot_vec();
        let degree = curve.degree() * 6;
        let (knots, _) = curve.knot_vec().to_single_multi();
        for i in 1..knots.len() {
            for j in 1..=degree {
                let p = j as f64 / degree as f64;
                let t = knots[i - 1] * (1.0 - p) + knots[i] * p;
                let pt = curve.subs(t);
                hint = match self.search_parameter(pt, hint) {
                    Some(got) => got,
                    None => return false,
                };
                if !self.subs(hint.0, hint.1).near(&pt) {
                    return false;
                } else if hint.0 < uknot_vec[0] - TOLERANCE
                    || hint.0 - uknot_vec[0] > uknot_vec.range_length() + TOLERANCE
                {
                    return false;
                } else if hint.1 < vknot_vec[0] - TOLERANCE
                    || hint.1 - vknot_vec[0] > vknot_vec.range_length() + TOLERANCE
                {
                    return false;
                }
            }
        }
        true
    }
}

impl Surface for NURBSSurface<Vector4> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point { self.subs(u, v) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector { self.uder(u, v) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Self::Vector { self.vder(u, v) }
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Self::Vector {
        let pt = self.0.subs(u, v);
        let ud = self.0.uder(u, v);
        let vd = self.0.vder(u, v);
        pt.rat_der(ud).cross(pt.rat_der(vd)).normalize()
    }
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { self.parameter_range() }
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut surface = self.clone();
        surface.swap_axes();
        surface
    }
}

impl IncludeCurve<NURBSCurve<Vector4>> for NURBSSurface<Vector4> {
    #[inline(always)]
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let pt = curve.front();
        let mut hint = self.presearch(pt);
        hint = match self.search_parameter(pt, hint) {
            Some(got) => got,
            None => return false,
        };
        let uknot_vec = self.uknot_vec();
        let vknot_vec = self.vknot_vec();
        let degree = curve.degree() * 6;
        let (knots, _) = curve.knot_vec().to_single_multi();
        for i in 1..knots.len() {
            for j in 1..=degree {
                let p = j as f64 / degree as f64;
                let t = knots[i - 1] * (1.0 - p) + knots[i] * p;
                let pt = curve.subs(t);
                hint = match self.search_parameter(pt, hint) {
                    Some(got) => got,
                    None => return false,
                };
                if !self.subs(hint.0, hint.1).near(&pt) {
                    return false;
                } else if hint.0 < uknot_vec[0] - TOLERANCE
                    || hint.0 - uknot_vec[0] > uknot_vec.range_length() + TOLERANCE
                {
                    return false;
                } else if hint.1 < vknot_vec[0] - TOLERANCE
                    || hint.1 - vknot_vec[0] > vknot_vec.range_length() + TOLERANCE
                {
                    return false;
                }
            }
        }
        true
    }
}

impl<V: Homogeneous<f64>> BSplineSurface<V>
where <V::Point as EuclideanSpace>::Diff: InnerSpace
{
    fn search_rational_nearest_parameter(
        &self,
        pt: V::Point,
        (u0, v0): (f64, f64),
    ) -> Option<(f64, f64)> {
        let uder = self.uderivation();
        let vder = self.vderivation();
        let uuder = uder.uderivation();
        let uvder = uder.vderivation();
        let vvder = vder.vderivation();
        self.optimized_srnp(&uder, &vder, &uuder, &uvder, &vvder, pt, (u0, v0))
    }

    fn optimized_srnp(
        &self,
        uder: &Self,
        vder: &Self,
        uuder: &Self,
        uvder: &Self,
        vvder: &Self,
        pt: V::Point,
        (u0, v0): (f64, f64),
    ) -> Option<(f64, f64)> {
        self.sub_srnp(uder, vder, uuder, uvder, vvder, pt, (u0, v0), 0)
    }

    fn sub_srnp(
        &self,
        uder: &Self,
        vder: &Self,
        uuder: &Self,
        uvder: &Self,
        vvder: &Self,
        pt: V::Point,
        (u0, v0): (f64, f64),
        count: usize,
    ) -> Option<(f64, f64)> {
        let s = self.subs(u0, v0);
        let rs = s.to_point();
        let ud = uder.subs(u0, v0);
        let rud = s.rat_der(ud);
        let vd = vder.subs(u0, v0);
        let rvd = s.rat_der(vd);
        let uud = uuder.subs(u0, v0);
        let ruud = s.rat_der2(ud, uud);
        let uvd = uvder.subs(u0, v0);
        let ruvd = s.rat_cross_der(ud, vd, uvd);
        let vvd = vvder.subs(u0, v0);
        let rvvd = s.rat_der2(vd, vvd);
        let f_u = rud.dot(rs - pt);
        let f_v = rvd.dot(rs - pt);
        let a = ruud.dot(rs - pt) + rud.dot(rud);
        let c = ruvd.dot(rs - pt) + rud.dot(rvd);
        let b = rvvd.dot(rs - pt) + rvd.dot(rvd);
        let det = a * b - c * c;
        if det.so_small() {
            return Some((u0, v0));
        }
        let u = u0 - (b * f_u - c * f_v) / det;
        let v = v0 - (-c * f_u + a * f_v) / det;
        if u.near(&u0) && v.near(&v0) {
            Some((u, v))
        } else if count == 100 {
            None
        } else {
            self.sub_srnp(uder, vder, uuder, uvder, vvder, pt, (u, v), count + 1)
        }
    }
}

impl<V: Homogeneous<f64>> NURBSSurface<V>
where V::Point: MetricSpace<Metric = f64>
{
    fn presearch(&self, point: V::Point) -> (f64, f64) {
        const N: usize = 50;
        let mut res = (0.0, 0.0);
        let mut min = std::f64::INFINITY;
        for i in 0..=N {
            for j in 0..=N {
                let p = i as f64 / N as f64;
                let q = j as f64 / N as f64;
                let u = self.uknot_vec()[0] + p * self.uknot_vec().range_length();
                let v = self.vknot_vec()[0] + q * self.vknot_vec().range_length();
                let dist = self.subs(u, v).distance2(point);
                if dist < min {
                    min = dist;
                    res = (u, v);
                }
            }
        }
        res
    }
}

impl NURBSSurface<Vector4> {
    /// Serach the parameter `(u, v)` such that `self.subs(u, v).rational_projection()` is near `pt`.
    /// If cannot find, then return `None`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(2, 2);
    /// let ctrl_pts = vec![
    ///     vec![Vector4::new(0.0, 0.0, 0.0, 1.0), Vector4::new(0.1, 0.0, 0.5, 0.4), Vector4::new(1.0, 0.0, 0.6, 2.0), Vector4::new(0.4, 0.0, 0.4, 0.4)],
    ///     vec![Vector4::new(0.0, 0.2, 0.2, 2.0), Vector4::new(0.24, 0.24, 0.1, 1.2), Vector4::new(2.4, 1.8, 2.4, 0.6), Vector4::new(1.4, 0.42, 0.98, 1.4)],
    ///     vec![Vector4::new(0.0, 1.5, 1.2, 3.0), Vector4::new(1.02, 2.04, 1.7, 3.4), Vector4::new(0.42, 0.28, 0.7, 0.7), Vector4::new(0.6, 0.3, 0.0, 0.6)],
    ///     vec![Vector4::new(0.0, 1.0, 1.0, 1.0), Vector4::new(0.2, 2.0, 2.0, 2.0), Vector4::new(0.85, 1.7, 0.85, 1.7), Vector4::new(1.0, 1.0, 0.3, 1.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts);
    /// let surface = NURBSSurface::new(bspsurface);
    ///
    /// let pt = surface.subs(0.3, 0.7);
    /// let (u, v) = surface.search_parameter(pt, (0.5, 0.5)).unwrap();
    /// Point3::assert_near(&surface.subs(u, v), &pt);
    /// ```
    pub fn search_parameter(&self, pt: Point3, hint: (f64, f64)) -> Option<(f64, f64)> {
        let normal = self.normal(hint.0, hint.1);
        let flag = normal[0].abs() > normal[1].abs();
        let tmp = if flag { 0 } else { 1 };
        let flag = normal[tmp].abs() > normal[2].abs();
        let max = if flag { tmp } else { 2 };
        let idx0 = (max + 1) % 3;
        let idx1 = (max + 2) % 3;
        let knot_vecs = self.knot_vecs().clone();
        let closure = move |pt: &Vector4| Vector3::new(pt[idx0], pt[idx1], pt[3]);
        let closure = move |vec: &Vec<Vector4>| vec.iter().map(closure).collect::<Vec<_>>();
        let control_points: Vec<Vec<_>> = self.control_points().iter().map(closure).collect();
        let newsurface = NURBSSurface::new(BSplineSurface::new(knot_vecs, control_points));
        let newpt = Point2::new(pt[idx0], pt[idx1]);
        newsurface
            .search_parameter(newpt, hint)
            .filter(|(u, v)| self.subs(*u, *v).near(&pt))
    }
}

#[test]
fn test_include2d() {
    use std::iter::FromIterator;
    let knot_vec = KnotVec::uniform_knot(2, 3);
    let ctrl_pts = vec![
        vec![
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.05, 0.0, 0.5),
            Vector3::new(0.15, 0.0, 0.3),
            Vector3::new(1.0, 0.0, 1.0),
        ],
        vec![
            Vector3::new(0.0, 0.01, 0.1),
            Vector3::new(0.02, 0.02, 0.1),
            Vector3::new(0.16, 0.12, 0.4),
            Vector3::new(0.7, 0.21, 0.7),
        ],
        vec![
            Vector3::new(0.0, 0.02, 0.4),
            Vector3::new(0.15, 0.3, 0.5),
            Vector3::new(0.6, 0.4, 1.0),
            Vector3::new(0.4, 0.2, 0.4),
        ],
        vec![
            Vector3::new(0.0, 1.0, 1.0),
            Vector3::new(0.1, 1.0, 1.0),
            Vector3::new(0.25, 0.5, 0.5),
            Vector3::new(0.3, 0.3, 0.3),
        ],
    ];
    let surface = BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts);
    let bnd_box = BoundingBox::from_iter(&[Vector2::new(0.2, 0.3), Vector2::new(0.8, 0.6)]);
    let mut curve = surface.sectional_curve(bnd_box);
    curve.control_points_mut().for_each(|pt| *pt *= 3.0);
    assert!(!surface.include(&curve));
    let surface = NURBSSurface::new(surface);
    let curve = NURBSCurve::new(curve);
    assert!(surface.include(&curve));
}

#[test]
fn test_include3d() {
    let knot_vec = KnotVec::bezier_knot(2);
    let ctrl_pts = vec![
        vec![
            Vector4::new(-1.0, -1.0, 2.0, 1.0),
            Vector4::new(-1.0, 0.0, 0.0, 1.0),
            Vector4::new(-1.0, 1.0, 2.0, 1.0),
        ],
        vec![
            Vector4::new(0.0, -1.0, 0.0, 1.0),
            Vector4::new(0.0, 0.0, -2.0, 1.0),
            Vector4::new(0.0, 1.0, 0.0, 1.0),
        ],
        vec![
            Vector4::new(1.0, -1.0, 2.0, 1.0),
            Vector4::new(1.0, 0.0, 0.0, 1.0),
            Vector4::new(1.0, 1.0, 2.0, 1.0),
        ],
    ];
    let surface = NURBSSurface::new(BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts));

    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0,
    ]);
    let ctrl_pts = vec![
        // the vector of the indices of control points
        Vector4::new(0.0, -2.0, 2.0, 2.0),
        Vector4::new(1.0, -1.0, 1.0, 1.0),
        Vector4::new(1.0, 0.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        Vector4::new(0.0, 2.0, 2.0, 2.0),
        Vector4::new(-1.0, 1.0, 1.0, 1.0),
        Vector4::new(-1.0, 0.0, 1.0, 1.0),
        Vector4::new(-1.0, -1.0, 1.0, 1.0),
        Vector4::new(0.0, -2.0, 2.0, 2.0),
    ];
    let mut curve = NURBSCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    assert!(surface.include(&curve));
    *curve.control_point_mut(1) += Vector4::new(0.0, 0.0, 0.00001, 0.0);
    assert!(!surface.include(&curve));
}
