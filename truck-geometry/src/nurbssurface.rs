use crate::bspsurface::{CPColumnIter, CPRowIter};
use crate::*;

impl<V> NURBSSurface<V> {
    /// constructor
    #[inline(always)]
    pub fn new(bspsurface: BSplineSurface<V>) -> Self { NURBSSurface(bspsurface) }

    /// Returns the nurbs surface before rationalized
    #[inline(always)]
    pub fn non_rationalized(&self) -> &BSplineSurface<V> { &self.0 }

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
}

impl<V: Homogeneous<f64>> ParameterDivision2D for NURBSSurface<V>
where V::Point: MetricSpace<Metric = f64> {
    #[inline(always)]
    fn parameter_division(&self, tol: f64) -> (Vec<f64>, Vec<f64>) {
        self.0.create_space_division(tol, |v0, v1| {
            v0.to_point().distance2(v1.to_point())
        })
    }
}

impl Surface for NURBSSurface<Vector4> {
    type Point = Point3;
    type Vector = Vector3;
    type Curve = NURBSCurve<Vector4>;
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
    #[inline(always)]
    fn include(&self, curve: &Self::Curve) -> bool { self.0.rational_include(&curve.0) }
}
