use crate::*;

impl<V> NURBSCurve<V> {
    /// constructor
    #[inline(always)]
    pub fn new(curve: BSplineCurve<V>) -> Self { NURBSCurve(curve) }

    /// Returns the BSpline curve before rationalized.
    #[inline(always)]
    pub fn non_rationalized(&self) -> &BSplineCurve<V> { &self.0 }

    /// Returns the BSpline curve before rationalized.
    #[inline(always)]
    pub fn into_non_rationalized(self) -> BSplineCurve<V> { self.0 }
    /// Returns the reference of the knot vector
    #[inline(always)]
    pub fn knot_vec(&self) -> &KnotVec { &self.0.knot_vec }

    /// Returns the `idx`th knot
    #[inline(always)]
    pub fn knot(&self, idx: usize) -> f64 { self.0.knot_vec[idx] }

    /// Returns the reference of the control points.
    #[inline(always)]
    pub fn control_points(&self) -> &Vec<V> { &self.0.control_points }

    /// Returns the reference of the control point corresponding to the index `idx`.
    #[inline(always)]
    pub fn control_point(&self, idx: usize) -> &V { &self.0.control_points[idx] }

    /// Returns the mutable reference of the control point corresponding to index `idx`.
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx: usize) -> &mut V { &mut self.0.control_points[idx] }
    /// Returns the iterator on all control points
    #[inline(always)]
    pub fn control_points_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.0.control_points.iter_mut()
    }

    /// Apply the given transformation to all control points.
    #[inline(always)]
    pub fn transform_control_points<F: FnMut(&mut V)>(&mut self, f: F) {
        self.0.control_points.iter_mut().for_each(f)
    }

    /// Returns the degree of B-spline curve
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// assert_eq!(bspcurve.degree(), 2);
    /// ```
    #[inline(always)]
    pub fn degree(&self) -> usize { self.0.knot_vec.len() - self.0.control_points.len() - 1 }
    /// Inverts a curve
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(2, 2);
    /// let ctrl_pts = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0), Vector2::new(4.0, 5.0)];
    /// let bspcurve0 = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let mut bspcurve1 = bspcurve0.clone();
    /// bspcurve1.invert();
    ///
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let t = (i as f64) / (N as f64);
    ///     Vector2::assert_near2(&bspcurve0.subs(t), &bspcurve1.subs(1.0 - t));
    /// }
    /// ```
    #[inline(always)]
    pub fn invert(&mut self) -> &mut Self {
        self.0.knot_vec.invert();
        self.0.control_points.reverse();
        self
    }

    /// Returns whether the knot vector is clamped or not.
    #[inline(always)]
    pub fn is_clamped(&self) -> bool { self.0.knot_vec.is_clamped(self.0.degree()) }

    /// Normalizes the knot vector  
    #[inline(always)]
    pub fn knot_normalize(&mut self) -> &mut Self {
        self.0.knot_vec.try_normalize().unwrap();
        self
    }

    /// Translates the knot vector
    #[inline(always)]
    pub fn knot_translate(&mut self, x: f64) -> &mut Self {
        self.0.knot_vec.translate(x);
        self
    }
}

impl<V: Homogeneous<f64>> NURBSCurve<V> {
    /// Returns the closure of substitution.
    /// # Examples
    /// The following test code is the same test with the one of `BSplineCurve::subs()`.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// const N: usize = 100; // sample size
    /// let get_t = |i: usize| -1.0 + 2.0 * (i as f64) / (N as f64);
    /// let res: Vec<_> = (0..=N).map(get_t).map(bspcurve.get_closure()).collect();
    /// let ans: Vec<_> = (0..=N).map(get_t).map(|t| Vector2::new(t, t * t)).collect();
    /// res.iter().zip(&ans).for_each(|(v0, v1)| Vector2::assert_near2(v0, v1));
    /// ```
    #[inline(always)]
    pub fn get_closure(&self) -> impl Fn(f64) -> V::Point + '_ { move |t| self.subs(t) }
}

impl<V: Homogeneous<f64> + Tolerance> NURBSCurve<V>
where V::Point: Tolerance
{
    /// Returns whether all control points are the same or not.
    /// If the knot vector is clamped, it means whether the curve is constant or not.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let pt = Vector2::new(1.0, 2.0);
    /// let mut ctrl_pts = vec![pt.clone(), pt.clone(), pt.clone()];
    /// let const_bspcurve = BSplineCurve::new(knot_vec.clone(), ctrl_pts.clone());
    /// assert!(const_bspcurve.is_const());
    ///
    /// ctrl_pts.push(Vector2::new(2.0, 3.0));
    /// let bspcurve = BSplineCurve::new(knot_vec.clone(), ctrl_pts.clone());
    /// assert!(!bspcurve.is_const());
    /// ```
    /// # Remarks
    /// If the knot vector is not clamped and the BSpline basis function is not partition of unity,
    /// then perhaps returns true even if the curve is not constant.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(1, 5);
    /// let ctrl_pts = vec![Vector2::new(1.0, 2.0), Vector2::new(1.0, 2.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// // bspcurve is not constant.
    /// assert_eq!(bspcurve.subs(0.0), Vector2::new(0.0, 0.0));
    /// assert_ne!(bspcurve.subs(0.5), Vector2::new(0.0, 0.0));
    ///
    /// // bspcurve.is_const() is true
    /// assert!(bspcurve.is_const());
    /// ```
    pub fn is_const(&self) -> bool {
        for vec in &self.0.control_points {
            if !vec.to_point().near(&self.0.control_points[0].to_point()) {
                return false;
            }
        }
        true
    }

    /// Adds a knot `x`, and do not change `self` as a curve.  
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    ///
    /// // add 4 knots
    /// bspcurve.add_knot(0.5).add_knot(0.5).add_knot(0.25).add_knot(0.75);
    /// assert_eq!(bspcurve.knot_vec().len(), org_curve.knot_vec().len() + 4);
    /// // bspcurve does not change as a curve
    /// assert!(bspcurve.near2_as_curve(&org_curve));
    /// ```
    /// # Remarks
    /// If the added knot `x` is out of the range of the knot vector, then the knot vector will extended.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// assert_eq!(bspcurve.knot_vec().range_length(), 1.0);
    /// assert_eq!(bspcurve.end_points(), (Vector2::new(-1.0, 1.0), Vector2::new(1.0, 1.0)));
    ///
    /// // add knots out of the range of the knot vectors.
    /// bspcurve.add_knot(-1.0).add_knot(2.0);
    /// assert_eq!(bspcurve.knot_vec().range_length(), 3.0);
    /// assert_eq!(bspcurve.end_points(), (Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0)));
    /// ```
    pub fn add_knot(&mut self, x: f64) -> &mut Self {
        self.0.add_knot(x);
        self
    }

    /// Removes a knot corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    ///
    /// // add knots and remove them.
    /// bspcurve.add_knot(0.5).add_knot(0.5).add_knot(0.25).add_knot(0.75);
    /// bspcurve.remove_knot(3).remove_knot(3).remove_knot(3).remove_knot(3);
    /// assert!(bspcurve.near2_as_curve(&org_curve));
    /// assert_eq!(bspcurve.knot_vec().len(), org_curve.knot_vec().len())
    /// ```
    pub fn remove_knot(&mut self, idx: usize) -> &mut Self {
        let _ = self.try_remove_knot(idx);
        self
    }

    /// Removes a knot corresponding to the indice `idx`, and do not change `self` as a curve.  
    /// If the knot cannot be removed, returns
    /// [`Error::CannotRemoveKnot`](./errors/enum.Error.html#variant.CannotRemoveKnot).
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    /// bspcurve.add_knot(0.5).add_knot(0.5).add_knot(0.25).add_knot(0.75);
    /// assert!(bspcurve.try_remove_knot(3).is_ok());
    /// assert_eq!(bspcurve.try_remove_knot(2), Err(Error::CannotRemoveKnot(2)));
    /// ```
    pub fn try_remove_knot(&mut self, idx: usize) -> Result<&mut Self> {
        self.0.try_remove_knot(idx)?;
        Ok(self)
    }

    /// elevate 1 degree.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(1);
    /// let ctrl_pts = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// bspcurve.elevate_degree();
    /// assert_eq!(bspcurve.degree(), 2);
    /// assert_eq!(bspcurve.knot_vec(), &KnotVec::bezier_knot(2));
    /// assert_eq!(bspcurve.control_point(1), &Vector2::new(0.5, 0.5));
    /// ```
    pub fn elevate_degree(&mut self) -> &mut Self {
        self.0.elevate_degree();
        self
    }

    /// Makes the B-spline curve clamped
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    /// let ctrl_pts = vec![Vector2::new(0.0, 1.0), Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// assert!(!bspcurve.is_clamped());
    /// bspcurve.clamp();
    /// assert!(bspcurve.is_clamped());
    /// assert_eq!(bspcurve.knot_vec().len(), 10);
    /// ```
    #[inline(always)]
    pub fn clamp(&mut self) -> &mut Self {
        self.0.clamp();
        self
    }

    /// Repeats `Self::try_remove_knot()` from the back knot in turn until the knot cannot be removed.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    ///
    /// // add 4 new knots
    /// bspcurve.add_knot(0.5).add_knot(0.5).add_knot(0.25).add_knot(0.75);
    /// assert_eq!(bspcurve.knot_vec().len(), KnotVec::bezier_knot(2).len() + 4);
    ///
    /// // By the optimization, added knots are removed.
    /// bspcurve.optimize();
    /// assert_eq!(bspcurve.knot_vec(), &KnotVec::bezier_knot(2));
    /// assert!(bspcurve.near2_as_curve(&org_curve));
    /// ```
    pub fn optimize(&mut self) -> &mut Self {
        self.0.optimize();
        self
    }

    /// Makes two splines having the same degrees.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec0 = KnotVec::bezier_knot(1);
    /// let ctrl_pts0 = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0)];
    /// let mut bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// let knot_vec1 = KnotVec::bezier_knot(2);
    /// let ctrl_pts1 = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0)];
    /// let mut bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    /// assert_ne!(bspcurve0.degree(), bspcurve1.degree());
    ///
    /// let org_curve0 = bspcurve0.clone();
    /// let org_curve1 = bspcurve1.clone();
    /// bspcurve0.syncro_degree(&mut bspcurve1);
    /// assert_eq!(bspcurve0.degree(), bspcurve1.degree());
    /// assert!(bspcurve0.near2_as_curve(&org_curve0));
    /// assert!(bspcurve1.near2_as_curve(&org_curve1));
    /// ```
    pub fn syncro_degree(&mut self, other: &mut Self) {
        let (degree0, degree1) = (self.degree(), other.degree());
        for _ in degree0..degree1 {
            self.elevate_degree();
        }
        for _ in degree1..degree0 {
            other.elevate_degree();
        }
    }

    /// Makes two splines having the same normalized knot vectors.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec0 = KnotVec::from(vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0]);
    /// let ctrl_pts0 = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0), Vector2::new(2.0, 2.0), Vector2::new(3.0, 3.0)];
    /// let mut bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// let mut org_curve0 = bspcurve0.clone();
    /// let knot_vec1 = KnotVec::from(vec![0.0, 0.0, 1.0, 3.0, 4.0, 4.0]);
    /// let ctrl_pts1 = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0), Vector2::new(2.0, 2.0), Vector2::new(3.0, 3.0)];
    /// let mut bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    /// let mut org_curve1 = bspcurve1.clone();
    ///
    /// bspcurve0.syncro_knots(&mut bspcurve1);
    ///
    /// // The knot vectors are made the same.
    /// assert_eq!(bspcurve0.knot_vec(), bspcurve1.knot_vec());
    /// assert_eq!(
    ///     bspcurve0.knot_vec().as_slice(),
    ///     &[0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0]
    /// );
    /// // The degrees are not changed.
    /// assert_eq!(bspcurve0.degree(), org_curve0.degree());
    /// assert_eq!(bspcurve1.degree(), org_curve1.degree());
    /// // The knot vector is normalized, however, the shape of curve is not changed.
    /// assert!(bspcurve0.near2_as_curve(org_curve0.knot_normalize()));
    /// assert!(bspcurve1.near2_as_curve(org_curve1.knot_normalize()));
    /// ```
    pub fn syncro_knots(&mut self, other: &mut Self) { self.0.syncro_knots(&mut other.0) }

    /// Cuts the curve to two curves at the parameter `t`
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec = KnotVec::uniform_knot(2, 3);
    /// let ctrl_pts = vec![
    ///     Vector2::new(0.0, 0.0),
    ///     Vector2::new(1.0, 0.0),
    ///     Vector2::new(2.0, 2.0),
    ///     Vector2::new(4.0, 3.0),
    ///     Vector2::new(5.0, 6.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// let mut part0 = bspcurve.clone();
    /// let part1 = part0.cut(0.56);
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     let t = 0.56 * (i as f64) / (N as f64);
    ///     Vector2::assert_near2(&bspcurve.subs(t), &part0.subs(t));
    /// }
    /// for i in 0..=N {
    ///     let t = 0.56 + 0.44 * (i as f64) / (N as f64);
    ///     Vector2::assert_near2(&bspcurve.subs(t), &part1.subs(t));
    /// }
    /// ```
    pub fn cut(&mut self, t: f64) -> Self { NURBSCurve(self.0.cut(t)) }

    /// Concats two B-spline curves.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(2, 3);
    /// let ctrl_pts = vec![
    ///     Vector2::new(0.0, 0.0),
    ///     Vector2::new(1.0, 0.0),
    ///     Vector2::new(2.0, 2.0),
    ///     Vector2::new(4.0, 3.0),
    ///     Vector2::new(5.0, 6.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// let mut part0 = bspcurve.clone();
    /// let mut part1 = part0.cut(0.56);
    /// part0.try_concat(&mut part1).unwrap();
    /// assert!(bspcurve.near2_as_curve(&part0));
    /// ```
    /// # Failure
    /// If the back of the knot vector of `self` does not coincides with the front of the one of `other`,
    /// returns [`Error::DifferentBackFront`](/errors/enum.Error.html#variant.DifferentBackFront).
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    ///
    /// let knot_vec0 = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let ctrl_pts0 = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// let knot_vec1 = KnotVec::from(vec![2.0, 2.0, 3.0, 3.0]);
    /// let ctrl_pts1 = vec![Vector2::new(1.0, 1.0), Vector2::new(2.0, 2.0)];
    /// let mut bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    ///
    /// assert_eq!(bspcurve0.try_concat(&mut bspcurve1), Err(Error::DifferentBackFront(1.0, 2.0)));
    /// ```
    /// # Remarks
    /// Unlike `Vec::append()`, this method does not change `other` as a curve.  
    /// However, side effects, such as degree synchronization, or knot vector clamped, do occur.
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec0 = KnotVec::bezier_knot(2);
    /// let ctrl_pts0 = vec![Vector2::new(0.0, 0.0), Vector2::new(0.0, 1.0), Vector2::new(2.0, 2.0)];
    /// let mut bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// let knot_vec1 = KnotVec::bezier_knot(1);
    /// let ctrl_pts1 = vec![Vector2::new(2.0, 2.0), Vector2::new(3.0, 3.0)];
    /// let mut bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    /// bspcurve1.knot_translate(1.0);
    /// let org_curve1 = bspcurve1.clone();
    ///
    /// bspcurve0.try_concat(&mut bspcurve1).unwrap();
    ///
    /// // do not change bspcurve as a curve
    /// assert!(bspcurve1.near2_as_curve(&org_curve1));
    /// // The degree is changed.
    /// assert_ne!(bspcurve1.degree(), org_curve1.degree());
    /// ```
    pub fn try_concat(&mut self, other: &mut Self) -> Result<&mut Self> {
        self.0.try_concat(&mut other.0)?;
        Ok(self)
    }

    /// Concats two B-spline curves.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector2::new(0.0, 0.0),
    ///     Vector2::new(1.0, 0.0),
    ///     Vector2::new(2.0, 2.0),
    ///     Vector2::new(4.0, 3.0),
    ///     Vector2::new(5.0, 6.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// let mut part0 = bspcurve.clone();
    /// let mut part1 = part0.cut(1.8);
    /// part0.concat(&mut part1);
    /// assert!(bspcurve.near2_as_curve(&part0));
    /// ```
    /// # Panics
    /// Panic occurs if the back of the knot vector of `self` does not coincides
    /// with the front of the one of `other`
    /// ```should_panic
    /// use truck_geometry::*;
    ///
    /// let knot_vec0 = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let ctrl_pts0 = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// let knot_vec1 = KnotVec::from(vec![2.0, 2.0, 3.0, 3.0]);
    /// let ctrl_pts1 = vec![Vector2::new(1.0, 1.0), Vector2::new(2.0, 2.0)];
    /// let mut bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    /// bspcurve0.concat(&mut bspcurve1);
    /// ```
    /// # Remarks
    /// Unlike `Vec::append()`, this method does not change `other` as a curve.  
    /// However, side effects, such as degree synchronization, or knot vector clamped, do occur.
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec0 = KnotVec::bezier_knot(2);
    /// let ctrl_pts0 = vec![Vector2::new(0.0, 0.0), Vector2::new(0.0, 1.0), Vector2::new(2.0, 2.0)];
    /// let mut bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    /// let knot_vec1 = KnotVec::bezier_knot(1);
    /// let ctrl_pts1 = vec![Vector2::new(2.0, 2.0), Vector2::new(3.0, 3.0)];
    /// let mut bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    /// bspcurve1.knot_translate(1.0);
    /// let org_curve1 = bspcurve1.clone();
    ///
    /// bspcurve0.concat(&mut bspcurve1);
    ///
    /// // do not change bspcurve as a curve
    /// assert!(bspcurve1.near2_as_curve(&org_curve1));
    /// // The degree is changed.
    /// assert_ne!(bspcurve1.degree(), org_curve1.degree());
    /// ```
    #[inline(always)]
    pub fn concat(&mut self, other: &mut Self) -> &mut Self {
        self.try_concat(other)
            .unwrap_or_else(|error| panic!("{}", error))
    }
    /// Makes the rational curve locally injective.
    /// # Example
    /// ```
    /// use truck_geometry::*;
    /// const N : usize = 100; // sample size for test
    ///
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 3.0, 4.0, 4.0, 4.0]
    /// );
    /// let control_points = vec![
    ///     Vector4::new(1.0, 0.0, 0.0, 1.0),
    ///     Vector4::new(0.0, 1.0, 0.0, 1.0),
    ///     Vector4::new(0.0, 2.0, 0.0, 2.0),
    ///     Vector4::new(0.0, 3.0, 0.0, 3.0),
    ///     Vector4::new(0.0, 0.0, 3.0, 3.0),
    /// ];
    ///
    /// let mut curve = NURBSCurve::new(BSplineCurve::new(knot_vec, control_points));
    /// let mut flag = false;
    /// for i in 0..N {
    ///     let t = 4.0 * (i as f64) / (N as f64);
    ///     let pt0 = curve.subs(t);
    ///     let pt1 = curve.subs(t + 1.0 / (N as f64));
    ///     flag = flag || pt0.near(&pt1);
    /// }
    /// // There exists t such that bspcurve(t) == bspcurve(t + 0.01).
    /// assert!(flag);
    ///
    /// curve.make_locally_injective().knot_normalize();
    /// let mut flag = false;
    /// for i in 0..N {
    ///     let t = 1.0 * (i as f64) / (N as f64);
    ///     let pt0 = bspcurve.subs(t);
    ///     let pt1 = bspcurve.subs(t + 1.0 / (N as f64));
    ///     flag = flag || pt0.near(&pt1);
    /// }
    /// // There does not exist t such that bspcurve(t) == bspcurve(t + 0.01).
    /// assert!(!flag);
    /// ```
    /// # Remarks
    /// If `self` is a constant curve, then does nothing.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0]);
    /// let ctrl_pts = vec![
    ///     Vector3::new(1.0, 1.0, 1.0),
    ///     Vector3::new(2.0, 2.0, 2.0),
    ///     Vector3::new(3.0, 3.0, 3.0),
    ///     Vector3::new(4.0, 4.0, 4.0),
    /// ];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    /// bspcurve.make_rational_locally_injective();
    /// assert_eq!(bspcurve, org_curve);
    /// ```
    pub fn make_locally_injective(&mut self) -> &mut Self {
        let mut iter = self.0.bezier_decomposition().into_iter();
        while let Some(bezier) = iter.next().map(|curve| NURBSCurve(curve)) {
            if !bezier.is_const() {
                *self = bezier;
                break;
            }
        }
        let mut x = 0.0;
        for mut bezier in iter.map(|curve| NURBSCurve(curve)) {
            if bezier.is_const() {
                x += bezier.0.knot_vec.range_length();
            } else {
                let s0 = self.0.control_points.last().unwrap().weight();
                let s1 = bezier.0.control_points[0].weight();
                bezier
                    .0
                    .control_points
                    .iter_mut()
                    .for_each(move |vec| *vec = *vec * (s0 / s1));
                self.concat(bezier.knot_translate(-x));
            }
        }
        self
    }
    /// Determine whether `self` and `other` is near as the B-spline curves or not.  
    ///
    /// Divides each knot interval into the number of degree equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE` for each end points `t`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 4.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector2::new(1.0, 1.0),
    ///     Vector2::new(3.0, 2.0),
    ///     Vector2::new(2.0, 3.0),
    ///     Vector2::new(4.0, 5.0),
    ///     Vector2::new(5.0, 4.0),
    ///     Vector2::new(1.0, 1.0),
    /// ];
    /// let bspcurve0 = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let mut bspcurve1 = bspcurve0.clone();
    /// assert!(bspcurve0.near_as_curve(&bspcurve1));
    /// *bspcurve1.control_point_mut(1) += Vector2::new(0.01, 0.0002);
    /// assert!(!bspcurve0.near_as_curve(&bspcurve1));
    /// ```
    #[inline(always)]
    pub fn near_as_curve(&self, other: &Self) -> bool {
        self.0
            .sub_near_as_curve(&other.0, 2, move |x, y| x.to_point().near(&y.to_point()))
    }

    /// Determines `self` and `other` is near in square order as the B-spline curves or not.  
    ///
    /// Divide each knot interval into the number of degree equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE`for each end points `t`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let eps = TOLERANCE;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 4.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector2::new(1.0, 1.0),
    ///     Vector2::new(3.0, 2.0),
    ///     Vector2::new(2.0, 3.0),
    ///     Vector2::new(4.0, 5.0),
    ///     Vector2::new(5.0, 4.0),
    ///     Vector2::new(1.0, 1.0),
    /// ];
    /// let bspcurve0 = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let mut bspcurve1 = bspcurve0.clone();
    /// assert!(bspcurve0.near_as_curve(&bspcurve1));
    /// *bspcurve1.control_point_mut(1) += Vector2::new(eps, 0.0);
    /// assert!(!bspcurve0.near2_as_curve(&bspcurve1));
    /// ```
    #[inline(always)]
    pub fn near2_as_curve(&self, other: &Self) -> bool {
        self.0
            .sub_near_as_curve(&other.0, 2, move |x, y| x.to_point().near2(&y.to_point()))
    }
}

impl<V: Homogeneous<f64>> NURBSCurve<V>
where V::Point: MetricSpace<Metric = f64>
{
    /// Creates the curve division
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(2, 3);
    /// let ctrl_pts = vec![
    ///     Vector4::new(0.0, 0.0, 0.0, 1.0),
    ///     Vector4::new(2.0, 0.0, 0.0, 2.0),
    ///     Vector4::new(0.0, 3.0, 0.0, 3.0),
    ///     Vector4::new(0.0, 0.0, 2.0, 2.0),
    ///     Vector4::new(1.0, 1.0, 1.0, 1.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let tol = 0.01;
    /// let div = bspcurve.rational_parameter_division(tol);
    /// let knot_vec = bspcurve.knot_vec();
    /// assert_eq!(knot_vec[0], div[0]);
    /// assert_eq!(knot_vec.range_length(), div.last().unwrap() - div[0]);
    /// for i in 1..div.len() {
    ///     let pt0 = bspcurve.subs(div[i - 1]).rational_projection();
    ///     let pt1 = bspcurve.subs(div[i]).rational_projection();
    ///     let value_middle = (pt0 + pt1) / 2.0;
    ///     let param_middle = bspcurve.subs((div[i - 1] + div[i]) / 2.0).rational_projection();
    ///     println!("{}", value_middle.distance(param_middle));
    ///     assert!(value_middle.distance(param_middle) < tol);
    /// }
    /// ```
    #[inline(always)]
    pub fn parameter_division(&self, tol: f64) -> Vec<f64> {
        self.0
            .create_division(tol, move |v0, v1| v0.to_point().distance2(v1.to_point()))
    }
}

impl<V: Homogeneous<f64>> NURBSCurve<V>
where <V::Point as EuclideanSpace>::Diff: InnerSpace {
    /// Searches the parameter `t` which minimize |self(t) - point| in the projective space
    /// by Newton's method with initial guess `hint`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// // Defines the half unit circle in x > 0 as a rational curve `bspcurve`
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector3::new(0.0, -1.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 1.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// // search rational nearest parameter
    /// let pt = Vector2::new(1.0, 2.0);
    /// let hint = 0.6;
    /// let t = bspcurve.search_rational_nearest_parameter(pt, hint).unwrap();
    ///
    /// // check the answer
    /// let res = bspcurve.subs(t).rational_projection();
    /// let ans = pt / pt.magnitude();
    /// Vector2::assert_near2(&res, &ans);
    /// ```
    /// # Remarks
    /// It may converge to a local solution depending on the hint.
    /// ```
    /// use truck_geometry::*;
    ///
    /// // Same curve and point as above example
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector3::new(0.0, -1.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 1.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let pt = Vector2::new(1.0, 2.0);
    ///
    /// // another hint
    /// let hint = 0.5;
    ///
    /// // Newton's method is vibration divergent.
    /// assert!(bspcurve.search_rational_nearest_parameter(pt, hint).is_none());
    /// ```
    #[inline(always)]
    pub fn search_nearest_parameter(&self, point: V::Point, hint: f64) -> Option<f64> {
        self.0.search_rational_nearest_parameter(point, hint)
    }
}

impl<V: Homogeneous<f64>> Curve for NURBSCurve<V> {
    type Point = V::Point;
    type Vector = <V::Point as EuclideanSpace>::Diff;
    fn subs(&self, t: f64) -> Self::Point { self.0.subs(t).to_point() }
    fn der(&self, t: f64) -> Self::Vector {
        let pt = self.0.subs(t);
        let der = self.0.der(t);
        pt.rat_der(der)
    }
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) {
        (
            self.0.knot_vec[0],
            self.0.knot_vec[self.0.knot_vec.len() - 1],
        )
    }
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut curve = self.0.clone();
        curve.invert();
        NURBSCurve(curve)
    }
}

impl<V: Homogeneous<f64>> BSplineCurve<V>
where <V::Point as EuclideanSpace>::Diff: InnerSpace {
    fn search_rational_nearest_parameter(
        &self,
        point: V::Point,
        hint: f64,
    ) -> Option<f64>
    {
        let derived = self.derivation();
        let derived2 = derived.derivation();
        self.sub_srnp(&derived, &derived2, point, hint, 0)
    }

    fn sub_srnp(
        &self,
        derived: &BSplineCurve<V>,
        derived2: &BSplineCurve<V>,
        point: V::Point,
        hint: f64,
        counter: usize,
    ) -> Option<f64>
    {
        let pt = self.subs(hint);
        let der = derived.subs(hint);
        let der2 = derived2.subs(hint);
        let der2 = pt.rat_der2(der, der2);
        let der = pt.rat_der(der);
        let pt = pt.to_point() - point;
        let f = der.dot(pt);
        let fprime = der2.dot(pt) + der.magnitude2();
        let t = hint - f / fprime;
        if t.near2(&hint) {
            Some(t)
        } else if counter == 100 {
            None
        } else {
            self.sub_srnp(derived, derived2, point, t, counter + 1)
        }
    }
}

#[allow(dead_code)]
fn hoge() {
    let knot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    let control_points = vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)];
    let curve = NURBSCurve::new(BSplineCurve::new(knot_vec, control_points));
    let _pt = curve.subs(0.5);
}
