use crate::errors::Error;
use crate::traits::inv_or_zero;
use crate::*;
use std::convert::TryInto;
use std::ops::*;

impl<V: ExVectorSpace> BSplineCurve<V>
where V::Rationalized: cgmath::AbsDiffEq<Epsilon = f64>
{
    /// constructor.
    /// # Arguments
    /// * `knot_vec` - the knot vector
    /// * `control_points` - the vector of the control points
    /// # Panics
    /// Panics occurs if:
    /// * There are no control points.
    /// * The number of knots is more than the one of control points.
    /// * The range of the knot vector is zero.
    pub fn new(knot_vec: KnotVec, control_points: Vec<V>) -> BSplineCurve<V> {
        BSplineCurve::try_new(knot_vec, control_points).unwrap_or_else(|e| panic!("{}", e))
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vec` - the knot vector
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// * If there are no control points, returns [`Error::EmptyControlPoints`].
    /// * If the number of knots is more than the one of control points, returns [`Error::TooShortKnotVector`].
    /// * If the range of the knot vector is zero, returns [`Error::ZeroRange`].
    ///
    /// [`Error::EmptyControlPoints`]: errors/enum.Error.html#variant.EmptyControlPoints
    /// [`Error::TooShortKnotVector`]: errors/enum.Error.html#variant.TooShortKnotVector
    /// [`Error::ZeroRange`]: errors/enum.Error.html#variant.ZeroRange
    pub fn try_new(knot_vec: KnotVec, control_points: Vec<V>) -> Result<BSplineCurve<V>> {
        if control_points.is_empty() {
            Err(Error::EmptyControlPoints)
        } else if knot_vec.len() <= control_points.len() {
            Err(Error::TooShortKnotVector(
                knot_vec.len(),
                control_points.len(),
            ))
        } else if knot_vec.range_length().so_small() {
            Err(Error::ZeroRange)
        } else {
            Ok(BSplineCurve::new_unchecked(knot_vec, control_points))
        }
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vec` - the knot vector
    /// * `control_points` - the vector of the control points
    /// # Remarks
    /// This method is prepared only for performance-critical development and is not recommended.  
    /// This method does NOT check the rules for constructing B-spline curve.  
    /// The programmer must guarantee these conditions before using this method.
    pub fn new_unchecked(knot_vec: KnotVec, control_points: Vec<V>) -> BSplineCurve<V> {
        BSplineCurve::<V> {
            knot_vec: knot_vec,
            control_points: control_points,
        }
    }

    /// Returns the reference of the knot vector
    #[inline(always)]
    pub fn knot_vec(&self) -> &KnotVec { &self.knot_vec }

    /// Returns the `idx`th knot
    #[inline(always)]
    pub fn knot(&self, idx: usize) -> f64 { self.knot_vec[idx] }

    /// Returns the reference of the control points.
    #[inline(always)]
    pub fn control_points(&self) -> &Vec<V> { &self.control_points }

    /// Returns the reference of the control point corresponding to the index `idx`.
    #[inline(always)]
    pub fn control_point(&self, idx: usize) -> &V { &self.control_points[idx] }

    /// Returns the mutable reference of the control point corresponding to index `idx`.
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx: usize) -> &mut V { &mut self.control_points[idx] }

    /// Apply the given transformation to all control points.
    #[inline(always)]
    pub fn transform_control_points<F: FnMut(&mut V)>(&mut self, f: F) {
        self.control_points.iter_mut().for_each(f)
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
    pub fn degree(&self) -> usize { self.knot_vec.len() - self.control_points.len() - 1 }

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
        for vec in &self.control_points {
            if !vec.near(&self.control_points[0]) {
                return false;
            }
        }
        true
    }

    /// Returns whether constant curve or not, i.e. all control points are same or not.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let pt = Vector3::new(1.0, 2.0, 1.0);
    /// // allows differences upto scalars
    /// let mut ctrl_pts = vec![pt.clone(), pt.clone() * 2.0, pt.clone() * 3.0];
    /// let const_bspcurve = BSplineCurve::new(knot_vec.clone(), ctrl_pts.clone());
    /// assert!(const_bspcurve.is_rational_const());
    ///
    /// ctrl_pts.push(Vector3::new(2.0, 3.0, 1.0));
    /// let bspcurve = BSplineCurve::new(knot_vec.clone(), ctrl_pts.clone());
    /// assert!(!bspcurve.is_rational_const());
    /// ```
    pub fn is_rational_const(&self) -> bool {
        let pt = self.control_points[0].rational_projection();
        for vec in &self.control_points {
            if !vec.rational_projection().near(&pt) {
                return false;
            }
        }
        true
    }
    /// Returns the bounding box including all control points.
    #[inline(always)]
    pub fn roughly_bounding_box(&self) -> BoundingBox<V> { self.control_points.iter().collect() }

    /// substitution to B-spline curve.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// // bspcurve coincides with (t, t * t) in the range [-1.0..1.0].
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let t = -1.0 + 2.0 * (i as f64) / (N as f64);
    ///     Vector2::assert_near2(&bspcurve.subs(t), &Vector2::new(t, t * t));
    /// }
    /// ```
    #[inline(always)]
    pub fn subs(&self, t: f64) -> V {
        let basis = self
            .knot_vec
            .try_bspline_basis_functions(self.degree(), t)
            .unwrap();
        let iter = self.control_points.iter().zip(basis.iter());
        let mut sum = V::zero();
        iter.for_each(|(vec, basis)| sum += *vec * *basis);
        sum
    }

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
    pub fn get_closure(&self) -> impl Fn(f64) -> V + '_ { move |t| self.subs(t) }

    /// Returns the end points of a curve.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// assert_eq!(bspcurve.end_points(), (Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0)));
    /// ```
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// // Since the knot vector is too long to the number of control points,
    /// assert_eq!(bspcurve.end_points(), (Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0)));
    /// ```
    #[inline(always)]
    pub fn end_points(&self) -> (V, V) {
        let t0 = self.knot_vec[0];
        let t1 = self.knot_vec[self.knot_vec.len() - 1];
        (self.subs(t0), self.subs(t1))
    }

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
        self.knot_vec.invert();
        self.control_points.reverse();
        self
    }

    /// Returns whether the knot vector is clamped or not.
    #[inline(always)]
    pub fn is_clamped(&self) -> bool { self.knot_vec.is_clamped(self.degree()) }

    /// Normalizes the knot vector  
    #[inline(always)]
    pub fn knot_normalize(&mut self) -> &mut Self {
        self.knot_vec.try_normalize().unwrap();
        self
    }

    /// Translates the knot vector
    #[inline(always)]
    pub fn knot_translate(&mut self, x: f64) -> &mut Self {
        self.knot_vec.translate(x);
        self
    }

    #[inline(always)]
    fn delta_control_points(&self, i: usize) -> V {
        if i == 0 {
            self.control_point(i).clone()
        } else if i == self.control_points.len() {
            self.control_points[i - 1] * (-1.0)
        } else {
            self.control_points[i] - self.control_points[i - 1]
        }
    }

    /// Returns the derived B-spline curve.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let derived = bspcurve.derivation();
    ///
    /// // `bpscurve = (t, t^2), derived = (1, 2t)`
    /// const N : usize = 100; // sample size
    /// for i in 0..=N {
    ///     let t = 1.0 / (N as f64) * (i as f64);
    ///     Vector2::assert_near2(&derived.subs(t), &Vector2::new(1.0, 2.0 * t));
    /// }
    /// ```
    pub fn derivation(&self) -> BSplineCurve<V> {
        let n = self.control_points.len();
        let k = self.degree();
        let knot_vec = self.knot_vec.clone();
        let mut new_points = Vec::with_capacity(n + 1);
        if k > 0 {
            for i in 0..=n {
                let delta = knot_vec[i + k] - knot_vec[i];
                let coef = (k as f64) * inv_or_zero(delta);
                new_points.push(self.delta_control_points(i) * coef);
            }
        } else {
            new_points = vec![V::zero(); n];
        }
        BSplineCurve::new_unchecked(knot_vec, new_points)
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
        if x < self.knot_vec[0] {
            self.knot_vec.add_knot(x);
            self.control_points.insert(0, V::zero());
            return self;
        }

        let k = self.degree();
        let n = self.control_points.len();

        let idx = self.knot_vec.add_knot(x);
        let start = if idx > k { idx - k } else { 0 };
        let end = if idx > n {
            self.control_points.push(V::zero());
            n + 1
        } else {
            self.control_points
                .insert(idx - 1, self.control_point(idx - 1).clone());
            idx
        };
        for i in start..end {
            let i0 = end + start - i - 1;
            let delta = self.knot_vec[i0 + k + 1] - self.knot_vec[i0];
            let a = (self.knot_vec[idx] - self.knot_vec[i0]) * inv_or_zero(delta);
            let p = self.delta_control_points(i0) * (1.0 - a);
            self.control_points[i0] -= p;
        }
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
    pub fn try_remove_knot(&mut self, idx: usize) -> Result<&mut BSplineCurve<V>> {
        let k = self.degree();
        let n = self.control_points.len();
        let knot_vec = &self.knot_vec;

        if idx < k + 1 || idx >= n {
            return Err(Error::CannotRemoveKnot(idx));
        }

        let mut new_points = Vec::with_capacity(k + 1);
        new_points.push(self.control_point(idx - k - 1).clone());
        for i in (idx - k)..idx {
            let delta = knot_vec[i + k + 1] - knot_vec[i];
            let a = inv_or_zero(delta) * (knot_vec[idx] - knot_vec[i]);
            if a.so_small() {
                break;
            } else {
                let p = self.control_points[i] / a - *new_points.last().unwrap() * (1.0 - a) / a;
                new_points.push(p);
            }
        }

        if !new_points.last().unwrap().near(self.control_point(idx)) {
            return Err(Error::CannotRemoveKnot(idx));
        }

        for (i, vec) in new_points.into_iter().skip(1).enumerate() {
            self.control_points[idx - k + i] = vec;
        }

        self.control_points.remove(idx);
        self.knot_vec.remove(idx);
        Ok(self)
    }

    /// elevate 1 degree for bezier curve.
    fn elevate_degree_bezier(&mut self) -> &mut Self {
        let k = self.degree();
        self.knot_vec.add_knot(self.knot_vec[0]);
        self.knot_vec
            .add_knot(self.knot_vec[self.knot_vec.len() - 1]);
        self.control_points.push(V::zero());
        for i in 0..=(k + 1) {
            let i0 = k + 1 - i;
            let a = (i0 as f64) / ((k + 1) as f64);
            let p = self.delta_control_points(i0) * a;
            self.control_points[i0] -= p;
        }
        self
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
        let mut result = CurveCollector::Singleton;
        for mut bezier in self.bezier_decomposition() {
            result.concat(bezier.elevate_degree_bezier());
        }
        *self = result.try_into().unwrap();
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
        let degree = self.degree();

        let s = self.knot_vec.multiplicity(0);
        for _ in s..=degree {
            self.add_knot(self.knot_vec[0]);
        }

        let n = self.knot_vec.len();
        let s = self.knot_vec.multiplicity(n - 1);
        for _ in s..=degree {
            self.add_knot(self.knot_vec[n - 1]);
        }
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
        loop {
            let n = self.knot_vec.len();
            let mut flag = true;
            for i in 1..=n {
                flag = flag && self.try_remove_knot(n - i).is_err();
            }
            if flag {
                break;
            }
        }
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
    pub fn syncro_knots(&mut self, other: &mut BSplineCurve<V>) {
        self.knot_normalize();
        other.knot_normalize();

        let mut i = 0;
        let mut j = 0;
        while !self.knot(i).near2(&1.0) || !other.knot(j).near2(&1.0) {
            if self.knot(i) - other.knot(j) > TOLERANCE {
                self.add_knot(other.knot(j));
            } else if other.knot(j) - self.knot(i) > TOLERANCE {
                other.add_knot(self.knot(i));
            }
            i += 1;
            j += 1;
        }

        if self.knot_vec.len() < other.knot_vec.len() {
            for _ in 0..(other.knot_vec.len() - self.knot_vec.len()) {
                self.add_knot(1.0);
            }
        } else if other.knot_vec.len() < self.knot_vec.len() {
            for _ in 0..(self.knot_vec.len() - other.knot_vec.len()) {
                other.add_knot(1.0);
            }
        }
    }

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
    pub fn cut(&mut self, mut t: f64) -> BSplineCurve<V> {
        let degree = self.degree();

        let idx = match self.knot_vec.floor(t) {
            Some(idx) => idx,
            None => {
                let bspline = self.clone();
                let knot_vec = KnotVec::from(vec![t, self.knot_vec[0]]);
                let ctrl_pts = vec![V::zero()];
                *self = BSplineCurve::new(knot_vec, ctrl_pts);
                return bspline;
            }
        };
        let s = if t.near(&self.knot_vec[idx]) {
            t = self.knot_vec[idx];
            self.knot_vec.multiplicity(idx)
        } else {
            0
        };

        for _ in s..=degree {
            self.add_knot(t);
        }

        let k = self.knot_vec.floor(t).unwrap();
        let m = self.knot_vec.len();
        let n = self.control_points.len();
        let knot_vec0 = self.knot_vec.sub_vec(0..=k);
        let knot_vec1 = self.knot_vec.sub_vec((k - degree)..m);
        let control_points0 = Vec::from(&self.control_points[0..(k - degree)]);
        let control_points1 = Vec::from(&self.control_points[(k - degree)..n]);
        *self = BSplineCurve::new_unchecked(knot_vec0, control_points0);
        BSplineCurve::new_unchecked(knot_vec1, control_points1)
    }

    /// Separates `self` into Bezier curves by each knots.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec = KnotVec::uniform_knot(2, 2);
    /// let ctrl_pts = vec![Vector2::new(0.0, 1.0), Vector2::new(1.0, 2.0), Vector2::new(2.0, 3.0), Vector2::new(3.0, 4.0)];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let beziers = bspcurve.bezier_decomposition();
    ///
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     let t = 0.5 * (i as f64) / (N as f64);
    ///     Vector2::assert_near2(&bspcurve.subs(t), &beziers[0].subs(t));
    /// }
    /// for i in 0..=N {
    ///     let t = 0.5 + 0.5 * (i as f64) / (N as f64);
    ///     Vector2::assert_near2(&bspcurve.subs(t), &beziers[1].subs(t));
    /// }
    /// ```
    pub fn bezier_decomposition(&self) -> Vec<BSplineCurve<V>> {
        let mut bspline = self.clone();
        bspline.clamp();
        let (knots, _) = self.knot_vec.to_single_multi();
        let n = knots.len();

        let mut result = Vec::new();
        for i in 2..n {
            result.push(bspline.cut(knots[n - i]));
        }
        result.push(bspline);
        result.reverse();
        result
    }

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
    pub fn try_concat(&mut self, other: &mut BSplineCurve<V>) -> Result<&mut Self> {
        self.syncro_degree(other);
        self.clamp();
        other.clamp();
        self.knot_vec.try_concat(&other.knot_vec, self.degree())?;
        for point in &other.control_points {
            self.control_points.push(point.clone());
        }
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
        self.try_concat(other).unwrap_or_else(|error| panic!("{}", error))
    }

    /// Makes the curve locally injective.
    /// # Example
    /// ```
    /// use truck_geometry::*;
    /// const N : usize = 100; // sample size for test
    ///
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 3.0, 4.0, 4.0, 4.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector3::new(1.0, 0.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 0.0, 1.0),
    /// ];
    ///
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let mut flag = false;
    /// for i in 0..=N {
    ///     let t = 4.0 * (i as f64) / (N as f64);
    ///     flag = flag || bspcurve.subs(t).near(&bspcurve.subs(t + 1.0 / (N as f64)));
    /// }
    /// // There exists t such that bspcurve(t) == bspcurve(t + 0.01).
    /// assert!(flag);
    ///
    /// bspcurve.make_locally_injective().knot_normalize();
    /// let mut flag = false;
    /// for i in 0..=N {
    ///     let t = 1.0 * (i as f64) / (N as f64);
    ///     flag = flag || bspcurve.subs(t).near(&bspcurve.subs(t + 1.0 / (N as f64)));
    /// }
    /// // There does not exist t such that bspcurve(t) == bspcurve(t + 0.01).
    /// assert!(!flag);
    /// ```
    /// # Remarks
    /// If `self` is a constant curve, then does nothing.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0]);
    /// let ctrl_pts = vec![Vector2::new(1.0, 1.0); 4];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    /// bspcurve.make_locally_injective();
    /// assert_eq!(bspcurve, org_curve);
    /// ```
    pub fn make_locally_injective(&mut self) -> &mut Self {
        let mut iter = self.bezier_decomposition().into_iter();
        while let Some(bezier) = iter.next() {
            if !bezier.is_const() {
                *self = bezier;
                break;
            }
        }
        let mut x = 0.0;
        for mut bezier in iter {
            if bezier.is_const() {
                x += bezier.knot_vec.range_length();
            } else {
                self.concat(bezier.knot_translate(-x));
            }
        }
        self
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
    /// let mut bspcurve = BSplineCurve::new(knot_vec, control_points);
    /// let mut flag = false;
    /// for i in 0..N {
    ///     let t = 4.0 * (i as f64) / (N as f64);
    ///     let pt0 = bspcurve.subs(t).rational_projection();
    ///     let pt1 = bspcurve.subs(t + 1.0 / (N as f64)).rational_projection();
    ///     flag = flag || pt0.near(&pt1);
    /// }
    /// // There exists t such that bspcurve(t) == bspcurve(t + 0.01) as a rational point.
    /// assert!(flag);
    ///
    /// bspcurve.make_rational_locally_injective().knot_normalize();
    /// let mut flag = false;
    /// for i in 0..N {
    ///     let t = 1.0 * (i as f64) / (N as f64);
    ///     let pt0 = bspcurve.subs(t).rational_projection();
    ///     let pt1 = bspcurve.subs(t + 1.0 / (N as f64)).rational_projection();
    ///     flag = flag || pt0.near(&pt1);
    /// }
    /// // There does not exist t such that bspcurve(t) == bspcurve(t + 0.01) as a rational point.
    /// assert!(!flag);
    ///
    /// // the last control points is not the same, however, has the same rational projection.
    /// let pt0 = bspcurve.end_points().1;
    /// let pt1 = Vector4::new(0.0, 0.0, 3.0, 3.0);
    /// assert_ne!(pt0, pt1);
    /// assert_eq!(pt0.rational_projection(), pt1.rational_projection());
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
    pub fn make_rational_locally_injective(&mut self) -> &mut Self {
        let mut iter = self.bezier_decomposition().into_iter();
        while let Some(bezier) = iter.next() {
            if !bezier.is_rational_const() {
                *self = bezier;
                break;
            }
        }
        let mut x = 0.0;
        for mut bezier in iter {
            if bezier.is_rational_const() {
                x += bezier.knot_vec.range_length();
            } else {
                let s0 = self.control_points.last().unwrap().last();
                let s1 = bezier.control_points[0].last();
                bezier
                    .control_points
                    .iter_mut()
                    .for_each(|vec| *vec *= s0 / s1);
                self.concat(bezier.knot_translate(-x));
            }
        }
        self
    }

    /// Searches the parameter `t` which minimize `|self(t) - point|` by Newton's method
    /// with initial guess `hint`. If the repeated trial does not converge, then returns `None`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector3::new(0.0, 0.0, 0.0),
    ///     Vector3::new(1.0, 0.0, 0.0),
    ///     Vector3::new(1.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 1.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let pt = bspcurve.subs(1.2);
    /// let t = bspcurve.search_nearest_parameter(pt, 0.8).unwrap();
    /// assert_eq!(t, 1.2);
    /// ```
    /// # Remarks
    /// It may converge to a local solution depending on the hint.
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector3::new(0.0, 0.0, 0.0),
    ///     Vector3::new(1.0, 0.0, 0.0),
    ///     Vector3::new(1.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 1.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let pt = Vector3::new(0.0, 0.5, 1.0);
    /// let t = bspcurve.search_nearest_parameter(pt, 0.8).unwrap();
    /// let pt0 = bspcurve.subs(t);
    /// let pt1 = bspcurve.subs(3.0);
    /// // the point corresponding the obtained parameter is not
    /// // the globally nearest point in the curve.
    /// assert!((pt0 - pt).magnitude() > (pt1 - pt).magnitude());
    /// ```
    pub fn search_nearest_parameter(&self, point: V, hint: f64) -> Option<f64> {
        let derived = self.derivation();
        let derived2 = derived.derivation();
        self.sub_snp(&derived, &derived2, point, hint, 0)
    }

    fn optimized_snp(
        &self,
        derived: &BSplineCurve<V>,
        derived2: &BSplineCurve<V>,
        point: V,
        hint: f64,
    ) -> Option<f64>
    {
        self.sub_snp(derived, derived2, point, hint, 0)
    }

    fn sub_snp(
        &self,
        derived: &BSplineCurve<V>,
        derived2: &BSplineCurve<V>,
        point: V,
        hint: f64,
        counter: usize,
    ) -> Option<f64>
    {
        let pt = self.subs(hint) - point;
        let der = derived.subs(hint);
        let der2 = derived2.subs(hint);
        let f = der.dot(pt);
        let fprime = der2.dot(pt) + der.magnitude2();
        let t = hint - f / fprime;
        println!("{} {} {}", f, fprime, t);
        if t.near(&hint) {
            Some(t)
        } else if counter == 100 {
            None
        } else {
            self.sub_snp(derived, derived2, point, t, counter + 1)
        }
    }

    /// Determines whether `self` is an arc of `curve` by repeating applying Newton method.
    ///
    /// The parameter `hint` is the init value, required that `curve.subs(hint)` is the front point of `self`.
    ///
    /// If `self` is an arc of `curve`, then returns `Some(t)` such that `curve.subs(t)` coincides with
    /// the back point of `self`. Otherwise, returns `None`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector3::new(0.0, 0.0, 0.0),
    ///     Vector3::new(1.0, 0.0, 0.0),
    ///     Vector3::new(1.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     Vector3::new(0.0, 1.0, 1.0),
    /// ];
    /// let bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    ///
    /// let mut part = bspcurve.clone().cut(0.6);
    /// part.cut(2.8);
    /// let t = part.is_arc_of(&bspcurve, 0.6).unwrap();
    /// f64::assert_near2(&t, &2.8);
    ///
    /// // hint is required the init value.
    /// assert!(part.is_arc_of(&bspcurve, 0.7).is_none());
    ///
    /// // normal failure
    /// *part.control_point_mut(2) += Vector3::new(1.0, 2.0, 3.0);
    /// assert!(part.is_arc_of(&bspcurve, 0.6).is_none());
    /// ```
    pub fn is_arc_of(&self, curve: &BSplineCurve<V>, mut hint: f64) -> Option<f64> {
        let degree = std::cmp::max(self.degree(), curve.degree()) * 3 + 1;
        let (knots, _) = self.knot_vec.to_single_multi();
        if !self.subs(knots[0]).near(&curve.subs(hint)) {
            return None;
        }

        let derived = curve.derivation();
        let derived2 = derived.derivation();
        for i in 1..knots.len() {
            let range = knots[i] - knots[i - 1];
            for j in 1..=degree {
                let t = knots[i - 1] + range * (j as f64) / (degree as f64);
                let pt = self.subs(t);
                let res = curve.optimized_snp(&derived, &derived2, pt, hint);
                let flag = res.map(|res| hint <= res && curve.subs(res).near(&pt));
                hint = match flag {
                    Some(true) => res.unwrap(),
                    _ => return None,
                };
            }
        }
        Some(hint)
    }

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
    /// let pt = Vector3::new(1.0, 2.0, 1.0);
    /// let hint = 0.6;
    /// let t = bspcurve.search_rational_nearest_parameter(pt, hint).unwrap();
    ///
    /// // check the answer
    /// let res: Vector2 = bspcurve.subs(t).rational_projection().into();
    /// let ans: Vector2 = pt.rational_projection().into();
    /// Vector2::assert_near2(&(&ans / ans.magnitude()), &res);
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
    /// let pt = Vector3::new(1.0, 2.0, 1.0);
    ///
    /// // another hint
    /// let hint = 0.5;
    ///
    /// // Newton's method is vibration divergent.
    /// assert!(bspcurve.search_rational_nearest_parameter(pt, hint).is_none());
    /// ```
    pub fn search_rational_nearest_parameter(&self, point: V, hint: f64) -> Option<f64> {
        let derived = self.derivation();
        let derived2 = derived.derivation();
        self.sub_srnp(&derived, &derived2, point, hint, 0)
    }

    fn optimized_srnp(
        &self,
        derived: &BSplineCurve<V>,
        derived2: &BSplineCurve<V>,
        point: V,
        hint: f64,
    ) -> Option<f64>
    {
        self.sub_srnp(&derived, &derived2, point, hint, 0)
    }

    fn sub_srnp(
        &self,
        derived: &BSplineCurve<V>,
        derived2: &BSplineCurve<V>,
        point: V,
        hint: f64,
        counter: usize,
    ) -> Option<f64>
    {
        let pt = self.subs(hint);
        let der = derived.subs(hint);
        let der2 = derived2.subs(hint);
        let der2 = pt.rational_derivation2(der, der2);
        let der = pt.rational_derivation(der);
        let pt = pt.rational_projection() - point.rational_projection();
        let f = der.dot(pt);
        let fprime = der2.dot(pt) + der.magnitude2();
        let t = hint - f / fprime;
        println!("{}", t);
        if t.near2(&hint) {
            Some(t)
        } else if counter == 100 {
            None
        } else {
            self.sub_srnp(derived, derived2, point, t, counter + 1)
        }
    }
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// # let file = std::fs::File::open("tests/data/examples.tgb").unwrap();
    /// # let geomdata = truck_io::tgb::read(file).unwrap();
    /// # let mut bspline = geomdata.curves[2].clone();
    ///
    /// // let bspline = BSplineCurve<V>::new(...).unwrap();
    /// let (knots, _) = bspline.knot_vec().to_single_multi();
    /// assert_eq!(&knots, &[0.0, 0.5, 1.0]);
    ///
    /// let mut part = bspline.clone();
    /// let mut part = part.cut(0.2);
    /// part.cut(0.8);
    /// assert!(part.is_rational_arc_of(&mut bspline, 0.2).is_some());
    /// *part.control_point_mut(1) += Vector4::new(1.0, 2.0, 3.0, 4.0);
    /// assert!(part.is_rational_arc_of(&mut bspline, 0.2).is_none());
    /// ```
    pub fn is_rational_arc_of(&self, curve: &mut BSplineCurve<V>, mut hint: f64) -> Option<f64> {
        let degree = std::cmp::max(self.degree(), curve.degree()) * 3 + 1;
        let (knots, _) = self.knot_vec.to_single_multi();
        let pt0 = self.subs(knots[0]).rational_projection();
        let pt1 = curve.subs(hint).rational_projection();
        if !pt0.near(&pt1) {
            return None;
        }

        let derived = curve.derivation();
        let derived2 = derived.derivation();
        for i in 1..knots.len() {
            let range = knots[i] - knots[i - 1];
            for j in 1..=degree {
                let t = knots[i - 1] + range * (j as f64) / (degree as f64);
                let pt = self.subs(t);
                let res = curve.optimized_srnp(&derived, &derived2, pt, hint);
                let flag = res.map(|res| {
                    let pt0 = curve.subs(res).rational_projection();
                    let pt1 = pt.rational_projection();
                    hint <= res && pt0.near(&pt1)
                });
                hint = match flag {
                    Some(true) => res.unwrap(),
                    _ => return None,
                };
            }
        }
        Some(hint)
    }

    fn sub_near_as_curve<F: Fn(&V, &V) -> bool>(
        &self,
        other: &BSplineCurve<V>,
        div_coef: usize,
        ord: F,
    ) -> bool
    {
        if !self.knot_vec.same_range(&other.knot_vec) {
            return false;
        }

        let division = std::cmp::max(self.degree(), other.degree()) * div_coef;
        for i in 0..(self.knot_vec.len() - 1) {
            let delta = self.knot_vec[i + 1] - self.knot_vec[i];
            if delta.so_small() {
                continue;
            }

            for j in 0..division {
                let t = self.knot_vec[i] + delta * (j as f64) / (division as f64);
                if !ord(&self.subs(t), &other.subs(t)) {
                    return false;
                }
            }
        }
        true
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
    pub fn near_as_curve(&self, other: &BSplineCurve<V>) -> bool {
        self.sub_near_as_curve(other, 1, |x, y| x.near(y))
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
    pub fn near2_as_curve(&self, other: &BSplineCurve<V>) -> bool {
        self.sub_near_as_curve(other, 1, |x, y| x.near2(y))
    }

    /// determine `self` and `other` is near order as the NURBS curve in 3D space.  
    /// Divide each knot interval into the number of degree + 1 equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE`for each end points `t`.
    #[inline(always)]
    pub fn near_as_rational_curve(&self, other: &BSplineCurve<V>) -> bool {
        self.sub_near_as_curve(other, 2, |x, y| {
            x.rational_projection().near(&y.rational_projection())
        })
    }

    /// determine `self` and `other` is near in square order as the NURBS curves in 3D space.  
    /// Divide each knot interval into the number of degree + 1 equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE`for each end points `t`.
    #[inline(always)]
    pub fn near2_as_rational_curve(&self, other: &BSplineCurve<V>) -> bool {
        self.sub_near_as_curve(other, 2, |x, y| {
            x.rational_projection().near2(&y.rational_projection())
        })
    }
}

macro_rules! impl_mat_multi {
    ($vector: ty, $matrix: ty) => {
        impl Mul<BSplineCurve<$vector>> for $matrix {
            type Output = BSplineCurve<$vector>;
            fn mul(self, mut spline: BSplineCurve<$vector>) -> Self::Output {
                spline
                    .control_points
                    .iter_mut()
                    .for_each(|vec| *vec = self * *vec);
                spline
            }
        }
        impl Mul<&BSplineCurve<$vector>> for $matrix {
            type Output = BSplineCurve<$vector>;
            fn mul(self, spline: &BSplineCurve<$vector>) -> Self::Output { self * spline.clone() }
        }
    };
}

macro_rules! impl_scalar_multi {
    ($vector: ty, $scalar: ty) => {
        impl_mat_multi!($vector, $scalar);
        impl Mul<$scalar> for &BSplineCurve<$vector> {
            type Output = BSplineCurve<$vector>;
            fn mul(self, scalar: $scalar) -> Self::Output { scalar * self }
        }
        impl Mul<$scalar> for BSplineCurve<$vector> {
            type Output = BSplineCurve<$vector>;
            fn mul(self, scalar: $scalar) -> Self::Output { scalar * self }
        }
    };
}

impl_mat_multi!(Vector2, Matrix2);
impl_scalar_multi!(Vector2, f64);
impl_mat_multi!(Vector3, Matrix3);
impl_scalar_multi!(Vector3, f64);
impl_mat_multi!(Vector4, Matrix4);
impl_scalar_multi!(Vector4, f64);

impl<V: ExVectorSpace> CurveCollector<V>
where V::Rationalized: cgmath::AbsDiffEq<Epsilon = f64> {
    /// Concats two B-spline curves.
    #[inline(always)]
    pub fn try_concat(&mut self, curve: &mut BSplineCurve<V>) -> Result<&mut Self> {
        match self {
            CurveCollector::Singleton => {
                *self = CurveCollector::Curve(curve.clone());
            }
            CurveCollector::Curve(ref mut curve0) => {
                curve0.try_concat(curve)?;
            }
        }
        Ok(self)
    }
    
    /// Concats two B-spline curves.
    #[inline(always)]
    pub fn concat(&mut self, curve: &mut BSplineCurve<V>) -> &mut Self {
        self.try_concat(curve).unwrap_or_else(|error| panic!("{}", error))
    }
}

impl<V> std::convert::TryFrom<CurveCollector<V>> for BSplineCurve<V> {
    type Error = Error;
    #[inline(always)]
    fn try_from(collector: CurveCollector<V>) -> Result<Self> {
        match collector {
            CurveCollector::Singleton => Err(Error::EmptyCurveCollector),
            CurveCollector::Curve(curve) => Ok(curve),
        }
    }
}

#[test]
fn test_near_as_curve() {
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        Vector4::new(1.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 1.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 0.0, 0.0),
        Vector4::new(1.0, 0.0, 1.0, 0.0),
        Vector4::new(1.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 0.0),
    ];
    let bspline0 = BSplineCurve::new(knot_vec, control_points.clone());
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.5, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        control_points[0].clone(),
        control_points[1].clone(),
        control_points[2].clone(),
        &control_points[3] * (5.0 / 6.0) + &control_points[2] * (1.0 / 6.0),
        &control_points[4] * 0.5 + &control_points[3] * 0.5,
        &control_points[5] * (1.0 / 6.0) + &control_points[4] * (5.0 / 6.0),
        control_points[5].clone(),
        control_points[6].clone(),
        control_points[7].clone(),
    ];
    let bspline1 = BSplineCurve::new(knot_vec, control_points.clone());
    let knot_vec = KnotVec::from(vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0,
    ]);
    let control_points = vec![
        Vector4::new(1.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 1.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.01, 0.0, 0.0),
        Vector4::new(1.0, 0.0, 1.0, 0.0),
        Vector4::new(1.0, 0.0, 0.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 0.0),
    ];
    let bspline2 = BSplineCurve::new(knot_vec, control_points.clone());
    assert!(bspline0.near_as_curve(&bspline1));
    assert!(!bspline0.near_as_curve(&bspline2));
}
