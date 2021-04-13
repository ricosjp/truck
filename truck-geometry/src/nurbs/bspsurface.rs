use crate::errors::Error;
use crate::*;
use std::convert::TryInto;
use std::ops::*;

impl<V> BSplineSurface<V> {
    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Panics
    /// There are 3 rules for construct B-spline curve.
    /// * The number of knots is more than the one of control points.
    /// * There exist at least two different knots.
    /// * There are at least one control point.
    #[inline(always)]
    pub fn new(knot_vecs: (KnotVec, KnotVec), control_points: Vec<Vec<V>>) -> BSplineSurface<V> {
        BSplineSurface::try_new(knot_vecs, control_points).unwrap_or_else(|e| panic!("{}", e))
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// There are 3 rules for construct B-spline curve.
    /// * The number of knots is more than the one of control points.
    /// * There exist at least two different knots.
    /// * There are at least one control point.
    #[inline(always)]
    pub fn try_new(
        knot_vecs: (KnotVec, KnotVec),
        control_points: Vec<Vec<V>>,
    ) -> Result<BSplineSurface<V>> {
        if control_points.is_empty() {
            Err(Error::EmptyControlPoints)
        } else if control_points[0].is_empty() {
            Err(Error::EmptyControlPoints)
        } else if knot_vecs.0.len() <= control_points.len() {
            Err(Error::TooShortKnotVector(
                knot_vecs.0.len(),
                control_points.len(),
            ))
        } else if knot_vecs.1.len() <= control_points[0].len() {
            Err(Error::TooShortKnotVector(
                knot_vecs.1.len(),
                control_points[0].len(),
            ))
        } else if knot_vecs.0.range_length().so_small() || knot_vecs.1.range_length().so_small() {
            Err(Error::ZeroRange)
        } else {
            let len = control_points[0].len();
            if control_points
                .iter()
                .fold(false, |flag, vec| flag || vec.len() != len)
            {
                Err(Error::IrregularControlPoints)
            } else {
                Ok(BSplineSurface::new_unchecked(knot_vecs, control_points))
            }
        }
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// This method is prepared only for performance-critical development and is not recommended.  
    /// This method does NOT check the 3 rules for constructing B-spline surface.  
    /// The programmer must guarantee these conditions before using this method.  
    #[inline(always)]
    pub const fn new_unchecked(
        knot_vecs: (KnotVec, KnotVec),
        control_points: Vec<Vec<V>>,
    ) -> BSplineSurface<V> {
        BSplineSurface {
            knot_vecs,
            control_points,
        }
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// This method checks the 3 rules for constructing B-spline surface in the debug mode.  
    /// The programmer must guarantee these conditions before using this method.  
    #[inline(always)]
    pub fn debug_new(
        knot_vecs: (KnotVec, KnotVec),
        control_points: Vec<Vec<V>>,
    ) -> BSplineSurface<V> {
        match cfg!(debug_assertions) {
            true => Self::new(knot_vecs, control_points),
            false => Self::new_unchecked(knot_vecs, control_points),
        }
    }
    /// Returns the reference of the knot vectors
    #[inline(always)]
    pub fn knot_vecs(&self) -> &(KnotVec, KnotVec) { &self.knot_vecs }

    /// Returns the u knot vector.
    #[inline(always)]
    pub fn uknot_vec(&self) -> &KnotVec { &self.knot_vecs.0 }
    /// Returns the v knot vector.
    #[inline(always)]
    pub fn vknot_vec(&self) -> &KnotVec { &self.knot_vecs.1 }

    /// Returns the `idx`th u knot.
    #[inline(always)]
    pub fn uknot(&self, idx: usize) -> f64 { self.knot_vecs.0[idx] }
    /// returns the `idx`th v knot.
    #[inline(always)]
    pub fn vknot(&self, idx: usize) -> f64 { self.knot_vecs.1[idx] }

    /// Returns the reference of the vector of the control points
    #[inline(always)]
    pub fn control_points(&self) -> &Vec<Vec<V>> { &self.control_points }

    /// Returns the reference of the control point corresponding to the index `(idx0, idx1)`.
    #[inline(always)]
    pub fn control_point(&self, idx0: usize, idx1: usize) -> &V { &self.control_points[idx0][idx1] }
    /// Apply the given transformation to all control points.
    #[inline(always)]
    pub fn transform_control_points<F: FnMut(&mut V)>(&mut self, f: F) {
        self.control_points
            .iter_mut()
            .flat_map(|vec| vec)
            .for_each(f)
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
        CPRowIter {
            iter: self.control_points.iter(),
            idx: column_idx,
        }
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
        self.control_points[row_idx].iter()
    }

    /// Returns the mutable reference of the control point corresponding to index `(idx0, idx1)`.
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx0: usize, idx1: usize) -> &mut V {
        &mut self.control_points[idx0][idx1]
    }

    /// Returns the iterator on all control points
    #[inline(always)]
    pub fn control_points_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.control_points.iter_mut().flatten()
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
    pub fn udegree(&self) -> usize { self.knot_vecs.0.len() - self.control_points.len() - 1 }

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
    pub fn vdegree(&self) -> usize { self.knot_vecs.1.len() - self.control_points[0].len() - 1 }

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
    pub fn is_clamped(&self) -> bool {
        self.knot_vecs.0.is_clamped(self.udegree()) && self.knot_vecs.1.is_clamped(self.vdegree())
    }

    /// Swaps two parameters.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs0 = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts0 = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let mut bspsurface0 = BSplineSurface::new(knot_vecs0, ctrl_pts0);
    ///
    /// let knot_vecs1 = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(1));
    /// let ctrl_pts1 = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.0, 1.0)],
    ///     vec![Vector2::new(0.5, -1.0), Vector2::new(0.5, 2.0)],
    ///     vec![Vector2::new(1.0, 0.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let mut bspsurface1 = BSplineSurface::new(knot_vecs1, ctrl_pts1);
    /// assert_eq!(bspsurface0.swap_axes(), &bspsurface1);
    /// ```
    pub fn swap_axes(&mut self) -> &mut Self
    where V: Clone {
        let knot_vec = self.knot_vecs.0.clone();
        self.knot_vecs.0 = self.knot_vecs.1.clone();
        self.knot_vecs.1 = knot_vec;

        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        let mut new_points = vec![Vec::with_capacity(n0); n1];
        for pts in &self.control_points {
            for (vec0, pt) in new_points.iter_mut().zip(pts) {
                vec0.push(pt.clone());
            }
        }
        self.control_points = new_points;
        self
    }

    /// The range of the parameter of the surface.
    #[inline(always)]
    pub fn parameter_range(&self) -> ((f64, f64), (f64, f64)) {
        (
            (
                self.knot_vecs.0[0],
                self.knot_vecs.0[self.knot_vecs.0.len() - 1],
            ),
            (
                self.knot_vecs.1[0],
                self.knot_vecs.1[self.knot_vecs.1.len() - 1],
            ),
        )
    }
    /// Creates the curve whose control points are the `idx`th column control points of `self`.
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
    /// let bspcurve = bspsurface.column_curve(1);
    ///
    /// assert_eq!(bspcurve.knot_vec(), &KnotVec::bezier_knot(2));
    /// assert_eq!(
    ///     bspcurve.control_points(),
    ///     &vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// );
    /// ```
    pub fn column_curve(&self, row_idx: usize) -> BSplineCurve<V>
    where V: Clone {
        let knot_vec = self.vknot_vec().clone();
        let ctrl_pts = self.control_points[row_idx].clone();
        BSplineCurve::new_unchecked(knot_vec, ctrl_pts)
    }
    /// Creates the column sectional curve.
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
    /// let bspcurve = bspsurface.row_curve(1);
    ///
    /// assert_eq!(bspcurve.knot_vec(), &KnotVec::bezier_knot(1));
    /// assert_eq!(
    ///     bspcurve.control_points(),
    ///     &vec![Vector3::new(1.0, 0.0, 1.0), Vector3::new(1.0, 1.0, 1.0)],
    /// );
    /// ```
    pub fn row_curve(&self, column_idx: usize) -> BSplineCurve<V>
    where V: Clone {
        let knot_vec = self.uknot_vec().clone();
        let ctrl_pts: Vec<_> = self
            .ctrl_pts_row_iter(column_idx)
            .map(|pt| pt.clone())
            .collect();
        BSplineCurve::new_unchecked(knot_vec, ctrl_pts)
    }
}

impl<V: VectorSpace<Scalar = f64>> BSplineSurface<V> {
    /// Substitutes to a B-spline surface.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    ///
    /// // bspsurface: (v, 2v(1 - v)(2u - 1) + u)
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         assert_near2!(
    ///             bspsurface.subs(u, v),
    ///             Vector2::new(v, 2.0 * v * (1.0 - v) * (2.0 * u - 1.0) + u),
    ///         );
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn subs(&self, u: f64, v: f64) -> V {
        let (degree0, degree1) = self.degrees();
        let BSplineSurface {
            knot_vecs: (ref uknot_vec, ref vknot_vec),
            ref control_points,
        } = self;
        let basis0 = uknot_vec.bspline_basis_functions(degree0, u);
        let basis1 = vknot_vec.bspline_basis_functions(degree1, v);
        let closure = move |sum: V, (vec, b0): (&Vec<V>, f64)| {
            let closure = move |sum: V, (pt, b1): (&V, &f64)| sum + *pt * (b0 * b1);
            vec.iter().zip(&basis1).fold(sum, closure)
        };
        control_points.iter().zip(basis0).fold(V::zero(), closure)
    }
    /// Substitutes derived B-spline surface by the first parameter `u`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    ///
    /// // bspsurface: (v, 2v(1 - v)(2u - 1) + u), uderivation: (0.0, 4v(1 - v) + 1)
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         assert_near2!(
    ///             bspsurface.uder(u, v),
    ///             Vector2::new(0.0, 4.0 * v * (1.0 - v) + 1.0),
    ///         );
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn uder(&self, u: f64, v: f64) -> V {
        let (degree0, degree1) = self.degrees();
        let BSplineSurface {
            knot_vecs: (ref uknot_vec, ref vknot_vec),
            ref control_points,
        } = self;
        let basis0 = uknot_vec.bspline_basis_functions(degree0 - 1, u);
        let basis1 = vknot_vec.bspline_basis_functions(degree1, v);
        let closure = move |sum: V, (i, vec): (usize, &Vec<V>)| {
            let coef0 = inv_or_zero(uknot_vec[i + degree0] - uknot_vec[i]);
            let coef1 = inv_or_zero(uknot_vec[i + degree0 + 1] - uknot_vec[i + 1]);
            let closure = |sum: V, (pt, b1): (&V, &f64)| {
                sum + *pt * (basis0[i] * coef0 - basis0[i + 1] * coef1) * *b1
            };
            vec.iter().zip(&basis1).fold(sum, closure)
        };
        control_points.iter().enumerate().fold(V::zero(), closure) * degree0 as f64
    }
    /// Substitutes derived B-spline surface by the first parameter `v`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    ///
    /// // bspsurface: (v, 2v(1 - v)(2u - 1) + u), vderivation: (1, -2(2u - 1)(2v - 1))
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         assert_near2!(
    ///             bspsurface.vder(u, v),
    ///             Vector2::new(1.0, -2.0 * (2.0 * u - 1.0) * (2.0 * v - 1.0)),
    ///         );
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn vder(&self, u: f64, v: f64) -> V {
        let (degree0, degree1) = self.degrees();
        let BSplineSurface {
            knot_vecs: (ref uknot_vec, ref vknot_vec),
            ref control_points,
        } = self;
        let basis0 = uknot_vec.bspline_basis_functions(degree0, u);
        let basis1 = vknot_vec.bspline_basis_functions(degree1 - 1, v);
        let closure = |sum: V, (vec, b0): (&Vec<V>, f64)| {
            let closure = |sum: V, (i, pt): (usize, &V)| {
                let coef0 = inv_or_zero(vknot_vec[i + degree1] - vknot_vec[i]);
                let coef1 = inv_or_zero(vknot_vec[i + degree1 + 1] - vknot_vec[i + 1]);
                sum + *pt * (basis1[i] * coef0 - basis1[i + 1] * coef1) * b0
            };
            vec.iter().enumerate().fold(sum, closure)
        };
        control_points.iter().zip(basis0).fold(V::zero(), closure) * degree1 as f64
    }

    /// Substitutes 2nd-ord derived B-spline surface by the first parameter `u`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 0.5), Vector2::new(0.5, 1.0), Vector2::new(1.0, 0.5)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    ///
    /// // bspsurface: (v, 2 u^2 v^2 - 2 u^2 v - 6 u v^2 + 6uv + 2v^2 + u - 2v)
    /// // uuder: (0, 4v(v - 1))
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         assert_near2!(
    ///             bspsurface.uuder(u, v),
    ///             Vector2::new(0.0, 4.0 * v * (v - 1.0)),
    ///         );
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn uuder(&self, u: f64, v: f64) -> V {
        let (degree0, degree1) = self.degrees();
        if degree0 < 2 {
            return V::zero();
        }
        let BSplineSurface {
            knot_vecs: (ref uknot_vec, ref vknot_vec),
            ref control_points,
        } = self;
        let basis0 = uknot_vec.bspline_basis_functions(degree0 - 2, u);
        let basis1 = vknot_vec.bspline_basis_functions(degree1, v);
        let closure = move |sum: V, (i, vec): (usize, &Vec<V>)| {
            let a = inv_or_zero(uknot_vec[i + degree0] - uknot_vec[i]);
            let b = inv_or_zero(uknot_vec[i + degree0 + 1] - uknot_vec[i + 1]);
            let c = inv_or_zero(uknot_vec[i + degree0 - 1] - uknot_vec[i]);
            let d = inv_or_zero(uknot_vec[i + degree0] - uknot_vec[i + 1]);
            let e = inv_or_zero(uknot_vec[i + degree0 + 1] - uknot_vec[i + 2]);
            let closure = |sum: V, (pt, b1): (&V, &f64)| {
                sum + *pt
                    * (basis0[i] * a * c - basis0[i + 1] * (a + b) * d + basis0[i + 2] * b * e)
                    * *b1
            };
            vec.iter().zip(&basis1).fold(sum, closure)
        };
        control_points.iter().enumerate().fold(V::zero(), closure) * degree0 as f64
    }

    /// Substitutes 2nd-ord derived B-spline surface by the second parameter `v`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 0.5), Vector2::new(0.5, 1.0), Vector2::new(1.0, 0.5)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    ///
    /// // bspsurface: (v, 2 u^2 v^2 - 2 u^2 v - 6 u v^2 + 6uv + 2v^2 + u - 2v)
    /// // vvder: (0, 4(u^2 - 3u + 1))
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         assert_near2!(
    ///             bspsurface.vvder(u, v),
    ///             Vector2::new(0.0, 4.0 * (u * u - 3.0 * u + 1.0)),
    ///         );
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn vvder(&self, u: f64, v: f64) -> V {
        let (degree0, degree1) = self.degrees();
        if degree1 < 2 {
            return V::zero();
        }
        let BSplineSurface {
            knot_vecs: (ref uknot_vec, ref vknot_vec),
            ref control_points,
        } = self;
        let basis0 = uknot_vec.bspline_basis_functions(degree0, u);
        let basis1 = vknot_vec.bspline_basis_functions(degree1 - 2, v);
        let closure = |sum: V, (vec, b0): (&Vec<V>, f64)| {
            let closure = |sum: V, (i, pt): (usize, &V)| {
                let a = inv_or_zero(vknot_vec[i + degree1] - vknot_vec[i]);
                let b = inv_or_zero(vknot_vec[i + degree1 + 1] - vknot_vec[i + 1]);
                let c = inv_or_zero(vknot_vec[i + degree1 - 1] - vknot_vec[i]);
                let d = inv_or_zero(vknot_vec[i + degree1] - vknot_vec[i + 1]);
                let e = inv_or_zero(vknot_vec[i + degree1 + 1] - vknot_vec[i + 2]);
                sum + *pt
                    * (basis1[i] * a * c - basis1[i + 1] * (a + b) * d + basis1[i + 2] * b * e)
                    * b0
            };
            vec.iter().enumerate().fold(sum, closure)
        };
        control_points.iter().zip(basis0).fold(V::zero(), closure) * degree1 as f64
    }

    /// Substitutes 2nd-ord derived B-spline surface by the both parameters `u, v`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 0.5), Vector2::new(0.5, 1.0), Vector2::new(1.0, 0.5)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    ///
    /// // bspsurface: (v, 2 u^2 v^2 - 2 u^2 v - 6 u v^2 + 6uv + 2v^2 + u - 2v)
    /// // uvder: (0, 8uv - 4u - 12v + 6)
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         assert_near2!(
    ///             bspsurface.uvder(u, v),
    ///             Vector2::new(0.0, 8.0 * u * v - 4.0 * u - 12.0 * v + 6.0),
    ///         );
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn uvder(&self, u: f64, v: f64) -> V {
        let (degree0, degree1) = self.degrees();
        let BSplineSurface {
            knot_vecs: (ref uknot_vec, ref vknot_vec),
            ref control_points,
        } = self;
        let basis0 = uknot_vec.bspline_basis_functions(degree0 - 1, u);
        let basis1 = vknot_vec.bspline_basis_functions(degree1 - 1, v);
        let closure = |sum: V, (i, vec): (usize, &Vec<V>)| {
            let coef0 = inv_or_zero(uknot_vec[i + degree0] - uknot_vec[i]);
            let coef1 = inv_or_zero(uknot_vec[i + degree0 + 1] - uknot_vec[i + 1]);
            let b0 = basis0[i] * coef0 - basis0[i + 1] * coef1;
            let closure = |sum: V, (j, pt): (usize, &V)| {
                let coef0 = inv_or_zero(vknot_vec[j + degree1] - vknot_vec[j]);
                let coef1 = inv_or_zero(vknot_vec[j + degree1 + 1] - vknot_vec[j + 1]);
                sum + *pt * (basis1[j] * coef0 - basis1[j + 1] * coef1) * b0
            };
            vec.iter().enumerate().fold(sum, closure)
        };
        control_points.iter().enumerate().fold(V::zero(), closure) * degree0 as f64 * degree1 as f64
    }

    /// Returns the closure of substitution.
    #[inline(always)]
    pub fn get_closure(&self) -> impl Fn(f64, f64) -> V + '_ { move |u, v| self.subs(u, v) }

    #[inline(always)]
    fn udelta_control_points(&self, i: usize, j: usize) -> V {
        if i == 0 {
            self.control_point(i, j).clone()
        } else if i == self.control_points.len() {
            self.control_points[i - 1][j] * (-1.0)
        } else {
            self.control_points[i][j] - self.control_points[i - 1][j]
        }
    }

    #[inline(always)]
    fn vdelta_control_points(&self, i: usize, j: usize) -> V {
        if j == 0 {
            self.control_point(i, j).clone()
        } else if j == self.control_points[0].len() {
            self.control_points[i][j - 1] * (-1.0)
        } else {
            self.control_points[i][j] - self.control_points[i][j - 1]
        }
    }

    /// Calculate derived B-spline surface by the first parameter `u`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let uderivation = bspsurface.uderivation();
    ///
    /// // bspsurface: (v, 2v(1 - v)(2u - 1) + u), uderivation: (0.0, 4v(1 - v) + 1)
    /// const N: usize = 100; // sample size
    /// for i in 1..N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         assert_near2!(
    ///             uderivation.subs(u, v),
    ///             Vector2::new(0.0, 4.0 * v * (1.0 - v) + 1.0),
    ///         );
    ///     }
    /// }
    /// ```
    pub fn uderivation(&self) -> BSplineSurface<V> {
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        let (k, _) = self.degrees();
        let (uknot_vec, vknot_vec) = self.knot_vecs.clone();

        let new_points = if k > 0 {
            (0..=n0)
                .map(|i| {
                    let delta = uknot_vec[i + k] - uknot_vec[i];
                    let coef = (k as f64) * inv_or_zero(delta);
                    (0..n1)
                        .map(|j| self.udelta_control_points(i, j) * coef)
                        .collect()
                })
                .collect()
        } else {
            vec![vec![V::zero(); n1]; n0]
        };

        BSplineSurface::new_unchecked((uknot_vec, vknot_vec), new_points)
    }

    /// Calculate derived B-spline surface by the second parameter `v`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let vderivation = bspsurface.vderivation();
    ///
    /// // bspsurface: (v, 2v(1 - v)(2u - 1) + u), vderivation: (1, -2(2u - 1)(2v - 1))
    /// const N: usize = 100; // sample size
    /// for i in 0..=N {
    ///     let u = (i as f64) / (N as f64);
    ///     for j in 0..=N {
    ///         let v = (j as f64) / (N as f64);
    ///         assert_near2!(
    ///             vderivation.subs(u, v),
    ///             Vector2::new(1.0, -2.0 * (2.0 * u - 1.0) * (2.0 * v - 1.0)),
    ///         );
    ///     }
    /// }
    /// ```
    pub fn vderivation(&self) -> BSplineSurface<V> {
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        let (_, k) = self.degrees();

        let (uknot_vec, vknot_vec) = self.knot_vecs.clone();

        let new_points = if k > 0 {
            let mut new_points = vec![Vec::with_capacity(n1 + 1); n0];
            for j in 0..=n1 {
                let delta = vknot_vec[j + k] - vknot_vec[j];
                let coef = (k as f64) * inv_or_zero(delta);
                for (i, vec) in new_points.iter_mut().enumerate() {
                    vec.push(self.vdelta_control_points(i, j) * coef)
                }
            }
            new_points
        } else {
            vec![vec![V::zero(); n1]; n0]
        };

        BSplineSurface::new_unchecked((uknot_vec, vknot_vec), new_points)
    }

    fn is_far<F: Fn(V, V) -> f64>(
        &self,
        u0: f64,
        u1: f64,
        v0: f64,
        v1: f64,
        tol: f64,
        dist2: &F,
    ) -> bool {
        let (mut degree0, mut degree1) = self.degrees();
        let bspsurface = self.get_closure();
        degree0 *= 2;
        degree1 *= 2;
        let pt00 = bspsurface(u0, v0);
        let pt01 = bspsurface(u0, v1);
        let pt10 = bspsurface(u1, v0);
        let pt11 = bspsurface(u1, v1);
        for i in 0..=degree0 {
            for j in 0..=degree1 {
                let p = (i as f64) / (degree0 as f64);
                let q = (j as f64) / (degree1 as f64);
                let u = u0 * p + u1 * (1.0 - p);
                let v = v0 * q + v1 * (1.0 - q);
                let par_mid = bspsurface(u, v);
                let val_mid = pt00 * p * q
                    + pt01 * p * (1.0 - q)
                    + pt10 * (1.0 - p) * q
                    + pt11 * (1.0 - p) * (1.0 - q);
                let res = dist2(val_mid, par_mid);
                if res > tol * tol {
                    return true;
                }
            }
        }
        false
    }

    /// Creats the division of the parametric space.
    pub(super) fn create_space_division<F: Fn(V, V) -> f64>(
        &self,
        tol: f64,
        dist2: F,
    ) -> (Vec<f64>, Vec<f64>) {
        let (knot_vec0, knot_vec1) = self.knot_vecs();
        let u0 = knot_vec0[0];
        let u1 = knot_vec0[knot_vec0.len() - 1];
        let mut div0 = vec![u0, u1];
        let v0 = knot_vec1[0];
        let v1 = knot_vec1[knot_vec1.len() - 1];
        let mut div1 = vec![v0, v1];

        self.sub_create_space_division(tol, dist2, &mut div0, &mut div1);
        (div0, div1)
    }

    fn sub_create_space_division<F: Fn(V, V) -> f64>(
        &self,
        tol: f64,
        dist2: F,
        div0: &mut Vec<f64>,
        div1: &mut Vec<f64>,
    ) {
        let mut divide_flag0 = vec![false; div0.len() - 1];
        let mut divide_flag1 = vec![false; div1.len() - 1];

        for i in 1..div0.len() {
            for j in 1..div1.len() {
                let far = self.is_far(div0[i - 1], div0[i], div1[j - 1], div1[j], tol, &dist2);
                divide_flag0[i - 1] = divide_flag0[i - 1] || far;
                divide_flag1[j - 1] = divide_flag1[j - 1] || far;
            }
        }

        let mut new_div0 = vec![div0[0]];
        for i in 1..div0.len() {
            if divide_flag0[i - 1] {
                new_div0.push((div0[i - 1] + div0[i]) / 2.0);
            }
            new_div0.push(div0[i]);
        }

        let mut new_div1 = vec![div1[0]];
        for i in 1..div1.len() {
            if divide_flag1[i - 1] {
                new_div1.push((div1[i - 1] + div1[i]) / 2.0);
            }
            new_div1.push(div1[i]);
        }

        if div0.len() != new_div0.len() || div1.len() != new_div1.len() {
            *div0 = new_div0;
            *div1 = new_div1;
            self.sub_create_space_division(tol, dist2, div0, div1);
        }
    }
    pub(super) fn sub_near_as_surface<F: Fn(&V, &V) -> bool>(
        &self,
        other: &BSplineSurface<V>,
        div_coef: usize,
        ord: F,
    ) -> bool {
        if !self.knot_vecs.0.same_range(&other.knot_vecs.0) {
            return false;
        }
        if !self.knot_vecs.1.same_range(&other.knot_vecs.1) {
            return false;
        }

        let (self_degree0, self_degree1) = self.degrees();
        let (other_degree0, other_degree1) = other.degrees();
        let division0 = std::cmp::max(self_degree0, other_degree0) * div_coef;
        let division1 = std::cmp::max(self_degree1, other_degree1) * div_coef;

        for i0 in 1..self.knot_vecs.0.len() {
            let delta0 = self.knot_vecs.0[i0] - self.knot_vecs.0[i0 - 1];
            if delta0.so_small() {
                continue;
            }
            for j0 in 0..division0 {
                let u = self.knot_vecs.0[i0 - 1] + delta0 * (j0 as f64) / (division0 as f64);
                for i1 in 1..self.knot_vecs.1.len() {
                    let delta1 = self.knot_vecs.1[i1] - self.knot_vecs.1[i1 - 1];
                    if delta1.so_small() {
                        continue;
                    }
                    for j1 in 0..division1 {
                        let v =
                            self.knot_vecs.1[i1 - 1] + delta1 * (j1 as f64) / (division1 as f64);
                        if !ord(&self.subs(u, v), &other.subs(u, v)) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
}

impl<V: Tolerance> BSplineSurface<V> {
    /// Returns whether all control points are same or not.
    /// If the knot vector is clamped, it means whether the curve is constant or not.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::bezier_knot(1);
    /// let vknot_vec = KnotVec::bezier_knot(2);
    /// let pt = Vector2::new(1.0, 2.0);
    /// let ctrl_pts = vec![
    ///     vec![pt.clone(), pt.clone(), pt.clone()],
    ///     vec![pt.clone(), pt.clone(), pt.clone()],
    /// ];
    /// let mut bspsurface = BSplineSurface::new((uknot_vec, vknot_vec), ctrl_pts);
    /// assert!(bspsurface.is_const());
    ///
    /// *bspsurface.control_point_mut(1, 2) = Vector2::new(2.0, 3.0);
    /// assert!(!bspsurface.is_const());
    /// ```
    /// # Remarks
    /// If the knot vector is not clamped and the BSpline basis function is not partition of unity,
    /// then perhaps returns true even if the surface is not constant.
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::uniform_knot(1, 5);
    /// let vknot_vec = KnotVec::uniform_knot(1, 5);
    /// let pt = Vector2::new(1.0, 2.0);
    /// let ctrl_pts = vec![
    ///     vec![pt.clone(), pt.clone(), pt.clone()],
    ///     vec![pt.clone(), pt.clone(), pt.clone()],
    /// ];
    /// let mut bspsurface = BSplineSurface::new((uknot_vec, vknot_vec), ctrl_pts);
    ///
    /// // bspsurface is not constant.
    /// assert_eq!(bspsurface.subs(0.0, 0.0), Vector2::new(0.0, 0.0));
    /// assert_ne!(bspsurface.subs(0.5, 0.5), Vector2::new(0.0, 0.0));
    ///
    /// // bspsurface.is_const() is true.
    /// assert!(bspsurface.is_const());
    /// ```
    #[inline(always)]
    pub fn is_const(&self) -> bool {
        for vec in self.control_points.iter().flat_map(|pts| pts.iter()) {
            if !vec.near(&self.control_points[0][0]) {
                return false;
            }
        }
        true
    }
}

impl<V: VectorSpace<Scalar = f64> + Tolerance> BSplineSurface<V> {
    /// Adds a knot `x` of the first parameter `u`, and do not change `self` as a surface.  
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    /// bspsurface.add_uknot(0.0).add_uknot(0.3).add_uknot(0.5).add_uknot(1.0);
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// assert_eq!(bspsurface.uknot_vec().len(), org_surface.uknot_vec().len() + 4);
    /// ```
    pub fn add_uknot(&mut self, x: f64) -> &mut Self {
        let k = self.udegree();
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();
        let uknot_vec = &mut self.knot_vecs.0;
        let control_points = &mut self.control_points;
        if x < uknot_vec[0] {
            uknot_vec.add_knot(x);
            control_points.insert(0, vec![V::zero(); n1]);
            return self;
        }

        let idx = uknot_vec.add_knot(x);
        let start = if idx > k { idx - k } else { 0 };
        let end = if idx > n0 {
            control_points.push(vec![V::zero(); n1]);
            n0 + 1
        } else {
            control_points.insert(idx - 1, control_points[idx - 1].clone());
            idx
        };
        for i in start..end {
            let i0 = end + start - i - 1;
            let delta = self.uknot(i0 + k + 1) - self.uknot(i0);
            let a = inv_or_zero(delta) * (self.uknot(idx) - self.uknot(i0));
            for j in 0..n1 {
                let p = self.udelta_control_points(i0, j) * (1.0 - a);
                self.control_points[i0][j] = self.control_points[i0][j] - p;
            }
        }
        self
    }

    /// add a knot `x` for the second parameter, and do not change `self` as a surface.  
    /// Return `false` if cannot add the knot, i.e.
    /// * the index of `x` will be lower than the degree, or
    /// * the index of `x` will be higher than the number of control points.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    /// bspsurface.add_vknot(0.0).add_vknot(0.3).add_vknot(0.5).add_vknot(1.0);
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// assert_eq!(bspsurface.vknot_vec().len(), org_surface.vknot_vec().len() + 4);
    /// ```
    pub fn add_vknot(&mut self, x: f64) -> &mut Self {
        if x < self.knot_vecs.1[0] {
            self.knot_vecs.1.add_knot(x);
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.insert(0, V::zero()));
            return self;
        }

        let k = self.vdegree();
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();

        let idx = self.knot_vecs.1.add_knot(x);
        let start = if idx > k { idx - k } else { 0 };
        let end = if idx > n1 {
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.push(V::zero()));
            n1 + 1
        } else {
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.insert(idx - 1, vec[idx - 1].clone()));
            idx
        };
        for j in start..end {
            let j0 = end + start - j - 1;
            let delta = self.knot_vecs.1[j0 + k + 1] - self.knot_vecs.1[j0];
            let a = inv_or_zero(delta) * (self.knot_vecs.1[idx] - self.knot_vecs.1[j0]);
            for i in 0..n0 {
                let p = self.vdelta_control_points(i, j0) * (1.0 - a);
                self.control_points[i][j0] = self.control_points[i][j0] - p;
            }
        }
        self
    }

    /// Removes the uknot corresponding to the indice `idx`, and do not change `self` as a curve.  
    /// If the knot cannot be removed, returns
    /// [`Error::CannotRemoveKnot`](./errors/enum.Error.html#variant.CannotRemoveKnot).
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    ///     vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 2.0)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// bspsurface.add_uknot(0.3).add_uknot(0.5);
    ///
    /// assert!(bspsurface.try_remove_uknot(3).is_ok());
    /// assert_eq!(bspsurface.try_remove_uknot(2), Err(Error::CannotRemoveKnot(2)));
    ///
    /// assert_eq!(bspsurface.uknot_vec().len(), org_surface.uknot_vec().len() + 1);
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// ```
    pub fn try_remove_uknot(&mut self, idx: usize) -> Result<&mut Self> {
        let k = self.udegree();
        let knot_vec = self.uknot_vec();
        let n = self.control_points.len();

        if idx < k + 1 || idx >= n {
            return Err(Error::CannotRemoveKnot(idx));
        }

        let mut new_points = Vec::with_capacity(k + 1);
        let first_vec = self
            .ctrl_pts_column_iter(idx - k - 1)
            .map(|pt| pt.clone())
            .collect::<Vec<_>>();
        new_points.push(first_vec);
        for i in (idx - k)..idx {
            let delta = knot_vec[i + k + 1] - knot_vec[i];
            let a = inv_or_zero(delta) * (knot_vec[idx] - knot_vec[i]);
            if a.so_small() {
                break;
            } else {
                let vec = self
                    .ctrl_pts_column_iter(i)
                    .zip(new_points.last().unwrap())
                    .map(|(pt0, pt1)| *pt0 / a - *pt1 * ((1.0 - a) / a))
                    .collect();
                new_points.push(vec);
            }
        }

        for (pt0, pt1) in self
            .ctrl_pts_column_iter(idx)
            .zip(new_points.last().unwrap())
        {
            if !pt0.near(pt1) {
                return Err(Error::CannotRemoveKnot(idx));
            }
        }

        for (i, vec) in new_points.into_iter().skip(1).enumerate() {
            self.control_points[idx - k + i] = vec;
        }

        self.control_points.remove(idx);
        self.knot_vecs.0.remove(idx);
        Ok(self)
    }

    /// Removes the uknot corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    ///     vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 2.0)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// bspsurface.add_uknot(0.3).add_uknot(0.5);
    /// bspsurface.remove_uknot(3).remove_uknot(3);
    ///
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// assert_eq!(bspsurface.uknot_vec().len(), org_surface.uknot_vec().len())
    /// ```
    #[inline(always)]
    pub fn remove_uknot(&mut self, idx: usize) -> &mut Self {
        let _ = self.try_remove_uknot(idx);
        self
    }

    /// Removes a vknot corresponding to the indice `idx`, and do not change `self` as a curve.  
    /// If the knot cannot be removed, returns
    /// [`Error::CannotRemoveKnot`](./errors/enum.Error.html#variant.CannotRemoveKnot).
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    ///     vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 2.0)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// bspsurface.add_vknot(0.3).add_vknot(0.5);
    /// assert!(bspsurface.try_remove_vknot(3).is_ok());
    /// assert_eq!(bspsurface.try_remove_vknot(2), Err(Error::CannotRemoveKnot(2)));
    ///
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// assert_eq!(bspsurface.vknot_vec().len(), org_surface.vknot_vec().len() + 1);
    /// ```
    pub fn try_remove_vknot(&mut self, idx: usize) -> Result<&mut Self> {
        let (_, k) = self.degrees();
        let knot_vec = self.vknot_vec();
        let n = self.control_points[0].len();

        if idx < k + 1 || idx >= n {
            return Err(Error::CannotRemoveKnot(idx));
        }

        let mut new_points = Vec::with_capacity(k + 1);
        let first_vec = self
            .ctrl_pts_row_iter(idx - k - 1)
            .map(|pt| pt.clone())
            .collect::<Vec<_>>();
        new_points.push(first_vec);
        for i in (idx - k)..idx {
            let delta = knot_vec[i + k + 1] - knot_vec[i];
            let a = inv_or_zero(delta) * (knot_vec[idx] - knot_vec[i]);
            if a.so_small() {
                break;
            } else {
                let vec = self
                    .ctrl_pts_row_iter(i)
                    .zip(new_points.last().unwrap())
                    .map(|(pt0, pt1)| *pt0 / a - *pt1 * ((1.0 - a) / a))
                    .collect();
                new_points.push(vec);
            }
        }

        for (pt0, pt1) in self.ctrl_pts_row_iter(idx).zip(new_points.last().unwrap()) {
            if !pt0.near(pt1) {
                return Err(Error::CannotRemoveKnot(idx));
            }
        }

        for (i, vec) in new_points.into_iter().skip(1).enumerate() {
            for (j, pt) in vec.into_iter().enumerate() {
                self.control_points[j][idx - k + i] = pt;
            }
        }

        for vec in &mut self.control_points {
            vec.remove(idx);
        }
        self.knot_vecs.1.remove(idx);
        Ok(self)
    }

    /// Removes a uknot corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    ///     vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 2.0)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// bspsurface.add_vknot(0.3).add_vknot(0.5);
    /// bspsurface.remove_vknot(3).remove_vknot(3);
    ///
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// assert_eq!(bspsurface.vknot_vec().len(), org_surface.vknot_vec().len())
    /// ```
    #[inline(always)]
    pub fn remove_vknot(&mut self, idx: usize) -> &mut Self {
        let _ = self.try_remove_vknot(idx);
        self
    }

    /// Elevates the vdegree.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    ///     vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 2.0)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// bspsurface.elevate_vdegree();
    ///
    /// assert_eq!(bspsurface.udegree(), org_surface.udegree());
    /// assert_eq!(bspsurface.vdegree(), org_surface.vdegree() + 1);
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// ```
    pub fn elevate_vdegree(&mut self) -> &mut Self {
        let mut new_knot_vec = KnotVec::new();
        for (i, vec) in self.control_points.iter_mut().enumerate() {
            let knot_vec = self.knot_vecs.1.clone();
            let ctrl_pts = vec.clone();
            let mut curve = BSplineCurve::new(knot_vec, ctrl_pts);
            curve.elevate_degree();
            if i == 0 {
                new_knot_vec = curve.knot_vec().clone();
            }
            *vec = curve.control_points;
        }
        self.knot_vecs.1 = new_knot_vec;
        self
    }

    /// Elevates the udegree.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 1.0)],
    ///     vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 2.0)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// bspsurface.elevate_udegree();
    ///
    /// assert_eq!(bspsurface.udegree(), org_surface.udegree() + 1);
    /// assert_eq!(bspsurface.vdegree(), org_surface.vdegree());
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// ```
    pub fn elevate_udegree(&mut self) -> &mut Self {
        self.swap_axes();
        self.elevate_vdegree();
        self.swap_axes();
        self
    }

    /// Aligns the udegree with the same degrees.
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
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// assert_ne!(bspsurface.udegree(), bspsurface.vdegree());
    /// bspsurface.syncro_uvdegrees();
    /// assert_eq!(bspsurface.udegree(), bspsurface.vdegree());
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// ```
    pub fn syncro_uvdegrees(&mut self) -> &mut Self {
        if self.udegree() > self.vdegree() {
            for _ in 0..(self.udegree() - self.vdegree()) {
                self.elevate_vdegree();
            }
        }
        if self.vdegree() > self.udegree() {
            for _ in 0..(self.vdegree() - self.udegree()) {
                self.elevate_udegree();
            }
        }
        self
    }

    /// Makes the uknot vector and the vknot vector the same knot vector.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let uknot_vec = KnotVec::uniform_knot(1, 2);
    /// let vknot_vec = KnotVec::bezier_knot(2);
    /// let knot_vecs = (uknot_vec, vknot_vec);
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    ///     vec![Vector3::new(0.0, 2.0, 0.0), Vector3::new(1.0, 2.0, 1.0), Vector3::new(2.0, 2.0, 2.0)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let org_surface = bspsurface.clone();
    ///
    /// assert_ne!(bspsurface.uknot_vec(), bspsurface.vknot_vec());
    /// bspsurface.syncro_uvknots();
    /// assert_eq!(bspsurface.uknot_vec(), bspsurface.vknot_vec());
    /// assert!(bspsurface.near2_as_surface(&org_surface));
    /// ```
    pub fn syncro_uvknots(&mut self) -> &mut Self {
        self.knot_vecs.0.normalize();
        self.knot_vecs.1.normalize();
        let mut i = 0;
        let mut j = 0;
        while !self.uknot(i).near2(&1.0) || !self.vknot(j).near2(&1.0) {
            if self.uknot(i) - self.vknot(j) > TOLERANCE {
                self.add_uknot(self.vknot(j));
            } else if self.vknot(j) - self.uknot(i) > TOLERANCE {
                self.add_vknot(self.uknot(i));
            }
            i += 1;
            j += 1;
        }

        let ulen = self.uknot_vec().len();
        let vlen = self.vknot_vec().len();
        if ulen > vlen {
            for _ in 0..ulen - vlen {
                self.add_vknot(1.0);
            }
        } else if ulen < vlen {
            for _ in 0..vlen - ulen {
                self.add_uknot(1.0);
            }
        }

        self
    }

    /// Cuts the surface into two surfaces at the parameter `u`
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec0 = KnotVec::uniform_knot(2, 2);
    /// let knot_vec1 = KnotVec::uniform_knot(2, 2);
    /// let ctrl_pts0 = vec![
    ///     Vector2::new(0.0, 0.0), Vector2::new(0.5, 0.0), Vector2::new(2.0, 0.0), Vector2::new(2.5, 0.0),
    /// ];
    /// let ctrl_pts1 = vec![
    ///     Vector2::new(0.0, 1.0), Vector2::new(0.5, 1.0), Vector2::new(2.0, 1.0), Vector2::new(2.5, 1.0),
    /// ];
    /// let ctrl_pts2 = vec![
    ///     Vector2::new(0.0, 1.5), Vector2::new(0.5, 1.5), Vector2::new(2.0, 1.5), Vector2::new(2.5, 1.5),
    /// ];
    /// let ctrl_pts3 = vec![
    ///     Vector2::new(0.0, 2.5), Vector2::new(0.5, 2.5), Vector2::new(2.0, 2.5), Vector2::new(2.5, 2.5),
    /// ];
    /// let ctrl_pts = vec![ctrl_pts0, ctrl_pts1, ctrl_pts2, ctrl_pts3];
    /// let bspsurface = BSplineSurface::new((knot_vec0, knot_vec1), ctrl_pts);
    ///
    /// let mut part0 = bspsurface.clone();
    /// let part1 = part0.ucut(0.68);
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     for j in 0..=N {
    ///         let u = 0.68 * (i as f64) / (N as f64);
    ///         let v = 1.0 * (j as f64) / (N as f64);
    ///         assert_near2!(bspsurface.subs(u, v), part0.subs(u, v));
    ///     }
    /// }
    /// for i in 0..=N {
    ///     for j in 0..=N {
    ///         let u = 0.68 + 0.32 * (i as f64) / (N as f64);
    ///         let v = 1.0 * (j as f64) / (N as f64);
    ///         assert_near2!(bspsurface.subs(u, v), part1.subs(u, v));
    ///     }
    /// }
    /// ```
    pub fn ucut(&mut self, mut u: f64) -> BSplineSurface<V> {
        let degree = self.udegree();

        let idx = match self.uknot_vec().floor(u) {
            Some(idx) => idx,
            None => {
                let bspline = self.clone();
                let uknot_vec = KnotVec::from(vec![u, self.vknot_vec()[0]]);
                let vknot_vec = self.vknot_vec().clone();
                let ctrl_pts = vec![vec![V::zero(); vknot_vec.len()]];
                *self = BSplineSurface::new((uknot_vec, vknot_vec), ctrl_pts);
                return bspline;
            }
        };
        let s = if u.near(&self.uknot_vec()[idx]) {
            u = self.uknot_vec()[idx];
            self.uknot_vec().multiplicity(idx)
        } else {
            0
        };

        for _ in s..=degree {
            self.add_uknot(u);
        }

        let vknot_vec = self.vknot_vec().clone();
        let k = self.uknot_vec().floor(u).unwrap();
        let m = self.uknot_vec().len();
        let n = self.control_points.len();
        let knot_vec0 = self.uknot_vec().sub_vec(0..=k);
        let knot_vec1 = self.uknot_vec().sub_vec((k - degree)..m);
        let control_points0 = Vec::from(&self.control_points[0..(k - degree)]);
        let control_points1 = Vec::from(&self.control_points[(k - degree)..n]);
        *self = BSplineSurface::new((knot_vec0, vknot_vec.clone()), control_points0);
        BSplineSurface::new((knot_vec1, vknot_vec), control_points1)
    }
    /// Cuts the curve to two curves at the parameter `t`
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec0 = KnotVec::uniform_knot(2, 2);
    /// let knot_vec1 = KnotVec::uniform_knot(2, 2);
    /// let ctrl_pts0 = vec![
    ///     Vector2::new(0.0, 0.0), Vector2::new(0.5, 0.0), Vector2::new(2.0, 0.0), Vector2::new(2.5, 0.0),
    /// ];
    /// let ctrl_pts1 = vec![
    ///     Vector2::new(0.0, 1.0), Vector2::new(0.5, 1.0), Vector2::new(2.0, 1.0), Vector2::new(2.5, 1.0),
    /// ];
    /// let ctrl_pts2 = vec![
    ///     Vector2::new(0.0, 1.5), Vector2::new(0.5, 1.5), Vector2::new(2.0, 1.5), Vector2::new(2.5, 1.5),
    /// ];
    /// let ctrl_pts3 = vec![
    ///     Vector2::new(0.0, 2.5), Vector2::new(0.5, 2.5), Vector2::new(2.0, 2.5), Vector2::new(2.5, 2.5),
    /// ];
    /// let ctrl_pts = vec![ctrl_pts0, ctrl_pts1, ctrl_pts2, ctrl_pts3];
    /// let bspsurface = BSplineSurface::new((knot_vec0, knot_vec1), ctrl_pts);
    ///
    /// let mut part0 = bspsurface.clone();
    /// let part1 = part0.vcut(0.68);
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     for j in 0..=N {
    ///         let u = 1.0 * (i as f64) / (N as f64);
    ///         let v = 0.68 * (j as f64) / (N as f64);
    ///         assert_near2!(bspsurface.subs(u, v), part0.subs(u, v));
    ///     }
    /// }
    /// for i in 0..=N {
    ///     for j in 0..=N {
    ///         let u = 1.0 * (i as f64) / (N as f64);
    ///         let v = 0.68 + 0.32 * (j as f64) / (N as f64);
    ///         assert_near2!(bspsurface.subs(u, v), part1.subs(u, v));
    ///     }
    /// }
    /// ```
    pub fn vcut(&mut self, v: f64) -> BSplineSurface<V> {
        self.swap_axes();
        let mut res = self.ucut(v);
        self.swap_axes();
        res.swap_axes();
        res
    }

    /// Creates a sectional curve with normalized knot vector from the parameter `p` to the parameter `q`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::iter::FromIterator;
    ///
    /// // a parabola surface: x = 2u - 1, y = 2v - 1, z = x^2 + y^z
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(-1.0, -1.0, 2.0), Vector3::new(-1.0, 0.0, 0.0), Vector3::new(-1.0, 1.0, 2.0)],
    ///     vec![Vector3::new(0.0, -1.0, 0.0), Vector3::new(0.0, 0.0, -2.0), Vector3::new(0.0, 1.0, 0.0)],
    ///     vec![Vector3::new(1.0, -1.0, 2.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 2.0)],
    /// ];
    /// let mut bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    ///
    /// // add some knots for the test!
    /// bspsurface.add_uknot(0.26);
    /// # bspsurface.add_uknot(0.64);
    /// bspsurface.add_vknot(0.23);
    /// # bspsurface.add_vknot(0.82);
    ///
    /// let bnd_box = BoundingBox::from_iter(&[Vector2::new(0.2, 0.3), Vector2::new(0.8, 0.6)]);
    /// let curve = bspsurface.sectional_curve(bnd_box);
    /// const N: usize = 100;
    /// assert_near2!(curve.subs(0.0), bspsurface.subs(0.2, 0.3));
    /// assert_near2!(curve.subs(1.0), bspsurface.subs(0.8, 0.6));
    /// for i in 0..=N {
    ///     let t = i as f64 / N as f64;
    ///     let pt = curve.subs(t);
    ///     assert_near2!(pt[1], pt[0] * 0.5 - 0.1);
    ///     assert_near2!(pt[2], pt[0] * pt[0] + pt[1] * pt[1]);
    /// }
    /// ```
    pub fn sectional_curve(&self, bnd_box: BoundingBox<Vector2>) -> BSplineCurve<V> {
        let p = bnd_box.min();
        let q = bnd_box.max();
        let mut bspsurface = self.clone();
        if !p[0].near(&bspsurface.uknot(0)) {
            bspsurface = bspsurface.ucut(p[0]);
        }
        if !q[0].near(&bspsurface.uknot(bspsurface.uknot_vec().len() - 1)) {
            bspsurface.ucut(q[0]);
        }
        if !p[0].near(&bspsurface.vknot(0)) {
            bspsurface = bspsurface.vcut(p[1]);
        }
        if !q[0].near(&bspsurface.vknot(bspsurface.vknot_vec().len() - 1)) {
            bspsurface.vcut(q[1]);
        }
        bspsurface.syncro_uvdegrees();
        bspsurface.syncro_uvknots();
        let degree = bspsurface.udegree();
        let comb = combinatorial(degree);
        let comb2 = combinatorial(degree * 2);
        let (knots, _) = bspsurface.uknot_vec().to_single_multi();
        let mut cc = CurveCollector::<V>::Singleton;
        for p in 1..knots.len() {
            let mut backup = None;
            if p + 1 != knots.len() {
                backup = Some(bspsurface.ucut(knots[p]));
                bspsurface.vcut(knots[p]);
            }
            let mut knot_vec = KnotVec::bezier_knot(degree * 2);
            knot_vec.translate(p as f64 - 1.0);
            let ctrl_pts: Vec<_> = (0..=degree * 2)
                .map(|k| {
                    (0..=k).fold(V::zero(), |sum, i| {
                        if i <= degree && k - i <= degree {
                            let coef = (comb[i] * comb[k - i]) as f64 / comb2[k] as f64;
                            sum + bspsurface.control_points[i][k - i] * coef
                        } else {
                            sum
                        }
                    })
                })
                .collect();
            cc.concat(&mut BSplineCurve::new(knot_vec, ctrl_pts));
            if p + 1 != knots.len() {
                bspsurface = backup.unwrap().vcut(knots[p]);
            }
        }
        let mut curve: BSplineCurve<V> = cc.try_into().unwrap();
        curve.knot_normalize();
        curve
    }

    /// Creates a surface with normailized knot vectors connecting two curves.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec0 = KnotVec::bezier_knot(2);
    /// let ctrl_pts0 = vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)];
    /// let bspcurve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    ///
    /// let knot_vec1 = KnotVec::bezier_knot(2);
    /// let ctrl_pts1 = vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, 1.0), Vector2::new(1.0, 2.0)];
    /// let bspcurve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    ///
    /// let homotopy_surface = BSplineSurface::homotopy(bspcurve0, bspcurve1);
    /// assert_eq!(
    ///     homotopy_surface.control_points(),
    ///     &vec![
    ///         vec![Vector2::new(0.0, 0.0), Vector2::new(0.0, 2.0)],
    ///         vec![Vector2::new(0.5, -1.0), Vector2::new(0.5, 1.0)],
    ///         vec![Vector2::new(1.0, 0.0), Vector2::new(1.0, 2.0)],
    ///     ],
    /// );
    /// ```
    pub fn homotopy(
        mut bspcurve0: BSplineCurve<V>,
        mut bspcurve1: BSplineCurve<V>,
    ) -> BSplineSurface<V> {
        bspcurve0.syncro_degree(&mut bspcurve1);

        bspcurve0.optimize();
        bspcurve1.optimize();

        bspcurve0.syncro_knots(&mut bspcurve1);

        let uknot_vec = bspcurve0.knot_vec().clone();
        let vknot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
        let mut control_points = Vec::new();
        for i in 0..bspcurve0.control_points().len() {
            control_points.push(Vec::new());
            control_points[i].push(bspcurve0.control_point(i).clone());
            control_points[i].push(bspcurve1.control_point(i).clone());
        }
        BSplineSurface::new_unchecked((uknot_vec, vknot_vec), control_points)
    }

    /// Creats a surface by its boundary.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let curve0 = BSplineCurve::new(
    ///     KnotVec::bezier_knot(1),
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.0)],
    /// );
    /// let curve1 = BSplineCurve::new(
    ///     KnotVec::bezier_knot(2),
    ///     vec![Vector2::new(1.0, 0.0), Vector2::new(2.0, 0.5), Vector2::new(1.0, 1.0)],
    /// );
    /// let curve2 = BSplineCurve::new(
    ///     KnotVec::bezier_knot(1),
    ///     vec![Vector2::new(1.0, 1.0), Vector2::new(0.0, 1.0)],
    /// );
    /// let curve3 = BSplineCurve::new(
    ///     KnotVec::bezier_knot(2),
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(-1.0, 0.5), Vector2::new(0.0, 0.0)],
    /// );
    /// let surface = BSplineSurface::by_boundary(curve0, curve1, curve2, curve3);
    /// assert_eq!(
    ///     surface.control_points(),
    ///     &vec![
    ///         vec![Vector2::new(0.0, 0.0), Vector2::new(-1.0, 0.5), Vector2::new(0.0, 1.0)],
    ///         vec![Vector2::new(1.0, 0.0), Vector2::new(2.0, 0.5), Vector2::new(1.0, 1.0)],
    ///     ],
    /// );
    /// ```
    /// # Remarks
    /// If the end points of curves are not connected, `curve1` and `curve3` take precedence. i.e.
    /// `curve1` and `curve3` are contained in the boundary of the surface and `curve0` and
    /// `curve2` are not contained.
    /// ```
    /// use truck_geometry::*;
    /// let curve0 = BSplineCurve::new(
    ///     KnotVec::bezier_knot(1),
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.0)],
    /// );
    /// let curve1 = BSplineCurve::new(
    ///     KnotVec::bezier_knot(2),
    ///     vec![Vector2::new(2.0, 0.0), Vector2::new(3.0, 0.5), Vector2::new(2.0, 1.0)],
    /// );
    /// let curve2 = BSplineCurve::new(
    ///     KnotVec::bezier_knot(1),
    ///     vec![Vector2::new(1.0, 1.0), Vector2::new(0.0, 1.0)],
    /// );
    /// let curve3 = BSplineCurve::new(
    ///     KnotVec::bezier_knot(2),
    ///     vec![Vector2::new(-1.0, 1.0), Vector2::new(-2.0, 0.5), Vector2::new(-1.0, 0.0)],
    /// );
    /// let surface = BSplineSurface::by_boundary(
    ///     curve0.clone(),
    ///     curve1.clone(),
    ///     curve2.clone(),
    ///     curve3.clone()
    /// );
    /// assert_ne!(surface.subs(0.0, 0.0), curve0.subs(0.0));
    /// assert_eq!(surface.subs(0.0, 0.0), curve3.subs(1.0));
    /// ```
    pub fn by_boundary(
        mut curve0: BSplineCurve<V>,
        mut curve1: BSplineCurve<V>,
        mut curve2: BSplineCurve<V>,
        mut curve3: BSplineCurve<V>,
    ) -> BSplineSurface<V> {
        curve2.invert();
        curve3.invert();
        curve0.syncro_degree(&mut curve2);
        curve0.optimize();
        curve2.optimize();
        curve0.syncro_knots(&mut curve2);
        curve1.syncro_degree(&mut curve3);
        curve1.optimize();
        curve3.optimize();
        curve1.syncro_knots(&mut curve3);

        let knot_vecs = (curve0.knot_vec().clone(), curve3.knot_vec().clone());
        let mut control_points = Vec::new();
        control_points.push(curve3.control_points().clone());
        let n = curve0.control_points().len();
        let m = curve3.control_points().len();
        for i in 1..(n - 1) {
            let u = (i as f64) / (n as f64);
            let pt0 = curve0.control_points[i] * u + curve2.control_points[i] * (1.0 - u);
            let mut new_row = Vec::new();
            new_row.push(curve0.control_point(i).clone());
            for j in 1..(m - 1) {
                let v = (j as f64) / (m as f64);
                let pt1 = curve3.control_points[j] * v + curve1.control_points[j] * (1.0 - v);
                new_row.push((pt0 + pt1) / 2.0);
            }
            new_row.push(curve2.control_point(i).clone());
            control_points.push(new_row);
        }
        control_points.push(curve1.control_points().clone());
        BSplineSurface::new(knot_vecs, control_points)
    }

    /// Normalizes the knot vectors
    #[inline(always)]
    pub fn knot_normalize(&mut self) -> &mut Self {
        self.knot_vecs.0.normalize();
        self.knot_vecs.1.normalize();
        self
    }

    /// Translates the knot vectors.
    #[inline(always)]
    pub fn knot_translate(&mut self, x: f64, y: f64) -> &mut Self {
        self.knot_vecs.0.translate(x);
        self.knot_vecs.1.translate(y);
        self
    }

    /// Removes knots in order from the back
    pub fn optimize(&mut self) -> &mut Self {
        loop {
            let (n0, n1) = (self.knot_vecs.0.len(), self.knot_vecs.1.len());
            let mut flag = true;
            for i in 1..=n0 {
                flag = flag && self.try_remove_uknot(n0 - i).is_err();
            }
            for j in 1..=n1 {
                flag = flag && self.try_remove_vknot(n1 - j).is_err();
            }
            if flag {
                break;
            }
        }
        self
    }

    /// Get the boundary by four splitted curves.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(3), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 1.0), Vector2::new(1.0, 1.0)],
    ///     vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 2.0)],
    ///     vec![Vector2::new(0.0, 3.0), Vector2::new(0.5, 3.5), Vector2::new(1.0, 3.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let curves = bspsurface.splitted_boundary();
    /// assert_eq!(
    ///     curves[0].control_points(),
    ///     &vec![Vector2::new(0.0, 0.0), Vector2::new(0.0, 1.0), Vector2::new(0.0, 2.0), Vector2::new(0.0, 3.0)],
    /// );
    /// assert_eq!(
    ///     curves[1].control_points(),
    ///     &vec![Vector2::new(0.0, 3.0), Vector2::new(0.5, 3.5), Vector2::new(1.0, 3.0)],
    /// );
    /// assert_eq!(
    ///     curves[2].control_points(),
    ///     &vec![Vector2::new(1.0, 3.0), Vector2::new(1.0, 2.0), Vector2::new(1.0, 1.0), Vector2::new(1.0, 0.0)],
    /// );
    /// assert_eq!(
    ///     curves[3].control_points(),
    ///     &vec![Vector2::new(1.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(0.0, 0.0)],
    /// );
    /// ```
    pub fn splitted_boundary(&self) -> [BSplineCurve<V>; 4] {
        let (uknot_vec, vknot_vec) = self.knot_vecs.clone();
        let control_points0 = self.control_points.iter().map(|x| x[0].clone()).collect();
        let control_points1 = self.control_points.last().unwrap().clone();
        let control_points2 = self
            .control_points
            .iter()
            .map(|x| x.last().unwrap().clone())
            .collect();
        let control_points3 = self.control_points[0].clone();
        let curve0 = BSplineCurve::new_unchecked(uknot_vec.clone(), control_points0);
        let curve1 = BSplineCurve::new_unchecked(vknot_vec.clone(), control_points1);
        let mut curve2 = BSplineCurve::new_unchecked(uknot_vec, control_points2);
        curve2.invert();
        let mut curve3 = BSplineCurve::new_unchecked(vknot_vec, control_points3);
        curve3.invert();
        [curve0, curve1, curve2, curve3]
    }

    /// Extracts the boundary of surface
    pub fn boundary(&self) -> BSplineCurve<V> {
        let (uknot_vec, vknot_vec) = self.knot_vecs.clone();
        let (range0, range1) = (uknot_vec.range_length(), vknot_vec.range_length());
        let [mut bspline0, mut bspline1, mut bspline2, mut bspline3] = self.splitted_boundary();
        bspline0
            .concat(&mut bspline1.knot_translate(range0))
            .concat(&mut bspline2.invert().knot_translate(range0 + range1))
            .concat(&mut bspline3.invert().knot_translate(range0 * 2.0 + range1));
        bspline0
    }
    /// Determines whether `self` and `other` is near as the B-spline surfaces or not.  
    ///
    /// Divides each knot domain into the number of degree equal parts,
    /// and check `|self(u, v) - other(u, v)| < TOLERANCE` for each end points `(u, v)`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(3), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 1.0), Vector2::new(1.0, 1.0)],
    ///     vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 2.0)],
    ///     vec![Vector2::new(0.0, 3.0), Vector2::new(0.5, 3.5), Vector2::new(1.0, 3.0)],
    /// ];
    /// let bspsurface0 = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let mut bspsurface1 = bspsurface0.clone();
    /// assert!(bspsurface0.near_as_surface(&bspsurface1));
    ///
    /// *bspsurface1.control_point_mut(1, 1) = Vector2::new(0.4, 1.0);
    /// assert!(!bspsurface0.near_as_surface(&bspsurface1));
    /// ```
    #[inline(always)]
    pub fn near_as_surface(&self, other: &BSplineSurface<V>) -> bool {
        self.sub_near_as_surface(other, 1, |x, y| x.near(y))
    }
    /// Determines whether `self` and `other` is near in square order as the B-spline surfaces or not.  
    ///
    /// Divides each knot domain into the number of degree equal parts,
    /// and check `|self(u, v) - other(u, v)| < TOLERANCE` for each end points `(u, v)`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let eps = TOLERANCE;
    /// let knot_vecs = (KnotVec::bezier_knot(3), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 1.0), Vector2::new(1.0, 1.0)],
    ///     vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 2.0)],
    ///     vec![Vector2::new(0.0, 3.0), Vector2::new(0.5, 3.5), Vector2::new(1.0, 3.0)],
    /// ];
    /// let bspsurface0 = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let mut bspsurface1 = bspsurface0.clone();
    /// assert!(bspsurface0.near_as_surface(&bspsurface1));
    ///
    /// *bspsurface1.control_point_mut(1, 1) += Vector2::new(eps, eps / 2.0);
    /// assert!(bspsurface0.near_as_surface(&bspsurface1));
    /// assert!(!bspsurface0.near2_as_surface(&bspsurface1));
    /// ```
    #[inline(always)]
    pub fn near2_as_surface(&self, other: &BSplineSurface<V>) -> bool {
        self.sub_near_as_surface(other, 1, |x, y| x.near2(y))
    }
}

impl<V> BSplineSurface<V>
where
    Self: ParametricSurface<Vector = V>,
    <Self as ParametricSurface>::Point: EuclideanSpace<Scalar = f64, Diff = V>,
    V: InnerSpace<Scalar = f64> + Tolerance,
{
    /// Searches the parameter `(u, v)` which minimize `|self(u, v) - point|` by Newton's method
    /// with initial guess `(u0, v0)`. If the repeated trial does not converge, then returns `None`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vecs = (KnotVec::bezier_knot(3), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, -1.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 1.0), Vector2::new(1.0, 1.0)],
    ///     vec![Vector2::new(0.0, 2.0), Vector2::new(0.5, 2.0), Vector2::new(1.0, 2.0)],
    ///     vec![Vector2::new(0.0, 3.0), Vector2::new(0.5, 3.5), Vector2::new(1.0, 3.0)],
    /// ];
    /// let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    /// let pt = ParametricSurface::subs(&bspsurface, 0.3, 0.7);
    /// let (u, v) = bspsurface.search_nearest_parameter(pt, (0.5, 0.5), 100).unwrap();
    /// assert!(u.near2(&0.3) && v.near2(&0.7));
    /// ```
    /// # Remarks
    /// It may converge to a local solution depending on the hint.
    /// cf. [`BSplineCurve::search_nearest_parameter`](struct.BSplineCurve.html#method.search_nearest_parameter)
    pub fn search_nearest_parameter(
        &self,
        pt: <Self as ParametricSurface>::Point,
        (u0, v0): (f64, f64),
        trials: usize,
    ) -> Option<(f64, f64)> {
        surface_search_nearest_parameter(self, pt, (u0, v0), trials)
    }
}

impl<V> BSplineSurface<V>
where V: MetricSpace<Metric = f64> + Index<usize, Output = f64> + Bounded<f64> + Copy
{
    /// Returns the bounding box including all control points.
    #[inline(always)]
    pub fn roughly_bounding_box(&self) -> BoundingBox<V> {
        self.control_points.iter().flatten().collect()
    }
}

impl<V: InnerSpace<Scalar = f64>> ParameterDivision2D for BSplineSurface<V> {
    #[inline(always)]
    fn parameter_division(&self, tol: f64) -> (Vec<f64>, Vec<f64>) {
        self.create_space_division(tol, |v0, v1| v0.distance2(v1))
    }
}

impl ParametricSurface for BSplineSurface<Vector2> {
    type Point = Point2;
    type Vector = Vector2;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point2 { Point2::from_vec(self.subs(u, v)) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector2 { self.uder(u, v) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector2 { self.vder(u, v) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector2 { self.uuder(u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector2 { self.uvder(u, v) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector2 { self.vvder(u, v) }
    /// zero identity
    #[inline(always)]
    fn normal(&self, _: f64, _: f64) -> Vector2 { Vector2::zero() }
}

impl<'a> ParametricSurface for &'a BSplineSurface<Vector2> {
    type Point = Point2;
    type Vector = Vector2;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point2 { Point2::from_vec((*self).subs(u, v)) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector2 { (*self).uder(u, v) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector2 { (*self).vder(u, v) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector2 { (*self).uuder(u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector2 { (*self).uvder(u, v) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector2 { (*self).vvder(u, v) }
    /// zero identity
    #[inline(always)]
    fn normal(&self, _: f64, _: f64) -> Vector2 { Vector2::zero() }
}

impl ParametricSurface for BSplineSurface<Vector3> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 { Point3::from_vec(self.subs(u, v)) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 { self.uder(u, v) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 { self.vder(u, v) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector3 { self.uuder(u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector3 { self.uvder(u, v) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector3 { self.vvder(u, v) }
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        self.uder(u, v).cross(self.vder(u, v)).normalize()
    }
}

impl<'a> ParametricSurface for &'a BSplineSurface<Vector3> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 { Point3::from_vec((*self).subs(u, v)) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 { (*self).uder(u, v) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 { (*self).vder(u, v) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector3 { (*self).uuder(u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector3 { (*self).uvder(u, v) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector3 { (*self).vvder(u, v) }
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        self.uder(u, v).cross(self.vder(u, v)).normalize()
    }
}

impl<V> BoundedSurface for BSplineSurface<V>
where BSplineSurface<V>: ParametricSurface
{
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { self.parameter_range() }
}

impl<V: Clone> Invertible for BSplineSurface<V> {
    #[inline(always)]
    fn invert(&mut self) { self.swap_axes(); }
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut surface = self.clone();
        surface.swap_axes();
        surface
    }
}

impl BSplineSurface<Vector2> {
    /// Serach the parameter `(u, v)` such that `self.subs(u, v)` is near `pt`.
    /// If cannot find, then return `None`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(2, 3);
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.1, 0.0), Vector2::new(0.5, 0.0), Vector2::new(0.7, 0.0), Vector2::new(1.0, 0.0)],
    ///     vec![Vector2::new(0.0, 0.1), Vector2::new(0.2, 0.2), Vector2::new(0.4, 0.3), Vector2::new(0.6, 0.2), Vector2::new(1.0, 0.3)],
    ///     vec![Vector2::new(0.0, 0.5), Vector2::new(0.3, 0.6), Vector2::new(0.6, 0.4), Vector2::new(0.9, 0.6), Vector2::new(1.0, 0.5)],
    ///     vec![Vector2::new(0.0, 0.7), Vector2::new(0.2, 0.8), Vector2::new(0.3, 0.6), Vector2::new(0.5, 0.9), Vector2::new(1.0, 0.7)],
    ///     vec![Vector2::new(0.0, 1.0), Vector2::new(0.1, 1.0), Vector2::new(0.5, 1.0), Vector2::new(0.7, 1.0), Vector2::new(1.0, 1.0)],
    /// ];
    /// let surface = BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts);
    ///
    /// let pt = Vector2::new(0.3, 0.7);
    /// let (u, v) = surface.search_parameter(pt, (0.5, 0.5), 100).unwrap();
    /// assert_near!(&surface.subs(u, v), &pt);
    /// ```
    pub fn search_parameter(&self, pt: Vector2, hint: (f64, f64), trials: usize) -> Option<(f64, f64)> {
        sub_search_parameter2d(self, Point2::from_vec(pt), hint.into(), trials).map(|v| v.into())
    }
}

impl IncludeCurve<BSplineCurve<Vector2>> for BSplineSurface<Vector2> {
    fn include(&self, curve: &BSplineCurve<Vector2>) -> bool {
        let pt = curve.subs(curve.knot_vec()[0]);
        let mut hint = presearch(self, Point2::from_vec(pt));
        hint = match self.search_parameter(pt, hint, INCLUDE_CURVE_TRIALS) {
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
                hint = match self.search_parameter(pt, hint, INCLUDE_CURVE_TRIALS) {
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

impl BSplineSurface<Vector3> {
    /// Serach the parameter `(u, v)` such that `self.subs(u, v)` is near `pt`.
    /// If cannot find, then return `None`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::uniform_knot(2, 2);
    /// let ctrl_pts = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.1, 0.0, 0.5), Vector3::new(0.5, 0.0, 0.3), Vector3::new(1.0, 0.0, 1.0)],
    ///     vec![Vector3::new(0.0, 0.1, 0.1), Vector3::new(0.2, 0.2, 0.1), Vector3::new(0.4, 0.3, 0.4), Vector3::new(1.0, 0.3, 0.7)],
    ///     vec![Vector3::new(0.0, 0.5, 0.4), Vector3::new(0.3, 0.6, 0.5), Vector3::new(0.6, 0.4, 1.0), Vector3::new(1.0, 0.5, 0.0)],
    ///     vec![Vector3::new(0.0, 1.0, 1.0), Vector3::new(0.1, 1.0, 1.0), Vector3::new(0.5, 1.0, 0.5), Vector3::new(1.0, 1.0, 0.3)],
    /// ];
    /// let surface = BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts);
    ///
    /// let pt = surface.subs(0.32, 0.76);
    /// let (u, v) = surface.search_parameter(pt, (0.5, 0.5), 100).unwrap();
    /// assert_near!(&surface.subs(u, v), &pt);
    ///
    /// let pt = surface.subs(0.32, 0.76) + Vector3::new(0.0, 0.0, 0.001);
    /// assert!(surface.search_parameter(pt, (0.5, 0.5), 100).is_none());
    /// ```
    pub fn search_parameter(&self, pt: Vector3, hint: (f64, f64), trials: usize) -> Option<(f64, f64)> {
        let normal = self.normal(hint.0, hint.1);
        let tmp = normal[0].abs() > normal[1].abs();
        let tmp_idx = if tmp { 0 } else { 1 };
        let tmp = normal[tmp_idx].abs() > normal[2].abs();
        let max = if tmp { tmp_idx } else { 2 };
        let idx0 = (max + 1) % 3;
        let idx1 = (max + 2) % 3;
        let knot_vecs = self.knot_vecs().clone();
        let control_points: Vec<Vec<_>> = self
            .control_points()
            .iter()
            .map(move |vec| {
                vec.iter()
                    .map(move |pt| Vector2::new(pt[idx0], pt[idx1]))
                    .collect()
            })
            .collect();
        let newsurface = BSplineSurface::new(knot_vecs, control_points);
        let newpt = Vector2::new(pt[idx0], pt[idx1]);
        newsurface
            .search_parameter(newpt, hint, trials)
            .filter(|(u, v)| self.subs(*u, *v).near(&pt))
    }
}

impl IncludeCurve<BSplineCurve<Vector3>> for BSplineSurface<Vector3> {
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        let pt = curve.subs(curve.knot_vec()[0]);
        let mut hint = presearch(self, Point3::from_vec(pt));
        hint = match self.search_parameter(pt, hint, INCLUDE_CURVE_TRIALS) {
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
                hint = match self.search_parameter(pt, hint, INCLUDE_CURVE_TRIALS) {
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

impl IncludeCurve<NURBSCurve<Vector4>> for BSplineSurface<Vector3> {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let pt = curve.subs(curve.knot_vec()[0]).to_vec();
        let mut hint = presearch(self, Point3::from_vec(pt));
        hint = match self.search_parameter(pt, hint, INCLUDE_CURVE_TRIALS) {
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
                let pt = curve.subs(t).to_vec();
                hint = match self.search_parameter(pt, hint, INCLUDE_CURVE_TRIALS) {
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

pub(super) fn sub_search_parameter2d<S: ParametricSurface<Point = Point2, Vector = Vector2>>(
    surface: &S,
    pt0: Point2,
    hint: Vector2,
    trials: usize,
) -> Option<Vector2> {
    let (u0, v0) = (hint[0], hint[1]);
    let pt = surface.subs(u0, v0);
    let jacobi = Matrix2::from_cols(surface.uder(u0, v0), surface.vder(u0, v0));
    let res = jacobi.invert().map(move |inv| hint - inv * (pt - pt0));
    match res {
        Some(entity) => match surface.subs(entity[0], entity[1]).near(&pt0) {
            true => res,
            false => match trials == 0 {
                true => None,
                false => sub_search_parameter2d(surface, pt0, entity, trials - 1),
            },
        },
        None => res,
    }
}

macro_rules! impl_mat_multi {
    ($vector: ty, $matrix: ty) => {
        impl Mul<BSplineSurface<$vector>> for $matrix {
            type Output = BSplineSurface<$vector>;
            fn mul(self, mut spline: BSplineSurface<$vector>) -> Self::Output {
                spline
                    .control_points
                    .iter_mut()
                    .flat_map(|vec| vec.iter_mut())
                    .for_each(|vec| *vec = self * *vec);
                spline
            }
        }
        impl Mul<&BSplineSurface<$vector>> for $matrix {
            type Output = BSplineSurface<$vector>;
            fn mul(self, spline: &BSplineSurface<$vector>) -> Self::Output { self * spline.clone() }
        }
    };
}

macro_rules! impl_scalar_multi {
    ($vector: ty, $scalar: ty) => {
        impl_mat_multi!($vector, $scalar);
        impl Mul<$scalar> for &BSplineSurface<$vector> {
            type Output = BSplineSurface<$vector>;
            fn mul(self, scalar: $scalar) -> Self::Output { scalar * self }
        }
        impl Mul<$scalar> for BSplineSurface<$vector> {
            type Output = BSplineSurface<$vector>;
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

impl Transformed<Matrix2> for BSplineSurface<Vector2> {
    fn transform_by(&mut self, trans: Matrix2) {
        self.control_points
            .iter_mut()
            .flatten()
            .for_each(|pt| *pt = trans * *pt)
    }
    fn transformed(&self, trans: Matrix2) -> Self {
        let mut surface = self.clone();
        surface.transform_by(trans);
        surface
    }
}

impl Transformed<Matrix3> for BSplineSurface<Vector2> {
    #[inline(always)]
    fn transform_by(&mut self, trans: Matrix3) {
        self.control_points
            .iter_mut()
            .flatten()
            .for_each(|pt| *pt = trans.transform_point(Point2::from_vec(*pt)).to_vec())
    }
    #[inline(always)]
    fn transformed(&self, trans: Matrix3) -> Self {
        let mut surface = self.clone();
        surface.transform_by(trans);
        surface
    }
}

impl Transformed<Matrix3> for BSplineSurface<Vector3> {
    #[inline(always)]
    fn transform_by(&mut self, trans: Matrix3) {
        self.control_points
            .iter_mut()
            .flatten()
            .for_each(|pt| *pt = trans * *pt)
    }
    #[inline(always)]
    fn transformed(&self, trans: Matrix3) -> Self {
        let mut surface = self.clone();
        surface.transform_by(trans);
        surface
    }
}

impl Transformed<Matrix4> for BSplineSurface<Vector3> {
    #[inline(always)]
    fn transform_by(&mut self, trans: Matrix4) {
        self.control_points
            .iter_mut()
            .flatten()
            .for_each(|pt| *pt = trans.transform_point(Point3::from_vec(*pt)).to_vec())
    }
    #[inline(always)]
    fn transformed(&self, trans: Matrix4) -> Self {
        let mut surface = self.clone();
        surface.transform_by(trans);
        surface
    }
}

/// The iterator on the control points in the specified column.
/// This iterator is generated by [`BSplineSurface::ctrl_pts_column_iter()`].
///
/// [`BSplineSurface::ctrl_pts_column_iter()`]: struct.BSplineSurface.html#method.ctrl_pts_column_iter
pub type CPColumnIter<'a, V> = std::slice::Iter<'a, V>;

/// The iterator on the control points in the specified row.
/// This iterator is generated by [`BSplineSurface::ctrl_pts_row_iter()`].
///
/// [`BSplineSurface::ctrl_pts_row_iter()`]: struct.BSplineSurface.html#method.ctrl_pts_row_iter
#[derive(Debug)]
pub struct CPRowIter<'a, V> {
    iter: std::slice::Iter<'a, Vec<V>>,
    idx: usize,
}

impl<'a, V> Iterator for CPRowIter<'a, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<&'a V> { self.iter.next().map(|arr| &arr[self.idx]) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
    fn count(self) -> usize { self.iter.count() }
}

impl<'a, V> DoubleEndedIterator for CPRowIter<'a, V> {
    fn next_back(&mut self) -> Option<&'a V> { self.iter.next_back().map(|arr| &arr[self.idx]) }
}

impl<'a, V> ExactSizeIterator for CPRowIter<'a, V> {
    fn len(&self) -> usize { self.iter.len() }
}

impl<'a, V: VectorSpace> std::iter::FusedIterator for CPRowIter<'a, V> {}

fn combinatorial(n: usize) -> Vec<usize> {
    let mut res = Vec::new();
    res.push(1);
    for i in 1..=n {
        res.push(res[i - 1] * (n - i + 1) / i);
    }
    res
}

#[test]
fn test_parameter_division() {
    let knot_vecs = (KnotVec::bezier_knot(3), KnotVec::bezier_knot(2));
    let ctrl_pts = vec![
        vec![
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.5, -1.0, 2.0),
            Vector3::new(1.0, 0.0, 1.0),
        ],
        vec![
            Vector3::new(0.0, 1.0, 1.0),
            Vector3::new(0.5, 1.0, 1.0),
            Vector3::new(1.0, 1.0, 1.0),
        ],
        vec![
            Vector3::new(0.0, 2.0, 1.0),
            Vector3::new(1.5, 6.0, 3.0),
            Vector3::new(1.0, 2.0, 1.0),
        ],
        vec![
            Vector3::new(0.0, 3.0, 1.0),
            Vector3::new(1.0, 7.0, 2.0),
            Vector3::new(1.0, 3.0, 1.0),
        ],
    ];
    let bspsurface = BSplineSurface::new(knot_vecs, ctrl_pts);
    let tol = 0.01;
    let (div0, div1) = bspsurface.parameter_division(tol);
    for i in 1..div0.len() {
        for j in 1..div1.len() {
            let pt0 = bspsurface.subs(div0[i - 1], div1[j - 1]);
            let pt1 = bspsurface.subs(div0[i - 1], div1[j]);
            let pt2 = bspsurface.subs(div0[i], div1[j - 1]);
            let pt3 = bspsurface.subs(div0[i], div1[j]);
            let value_middle = (pt0 + pt1 + pt2 + pt3) / 4.0;
            let u = (div0[i - 1] + div0[i]) / 2.0;
            let v = (div1[j - 1] + div1[j]) / 2.0;
            let parameter_middle = bspsurface.subs(u, v);
            println!("{}", value_middle.distance(parameter_middle));
            assert!(value_middle.distance(parameter_middle) < tol);
        }
    }
}

#[test]
fn test_include_bspcurve2() {
    let knot_vec = KnotVec::uniform_knot(2, 3);
    let ctrl_pts = vec![
        vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(0.1, 0.0),
            Vector2::new(0.5, 0.0),
            Vector2::new(0.7, 0.0),
            Vector2::new(1.0, 0.0),
        ],
        vec![
            Vector2::new(0.0, 0.1),
            Vector2::new(0.2, 0.2),
            Vector2::new(0.4, 0.3),
            Vector2::new(0.6, 0.2),
            Vector2::new(1.0, 0.3),
        ],
        vec![
            Vector2::new(0.0, 0.5),
            Vector2::new(0.3, 0.6),
            Vector2::new(0.6, 0.4),
            Vector2::new(0.9, 0.6),
            Vector2::new(1.0, 0.5),
        ],
        vec![
            Vector2::new(0.0, 0.7),
            Vector2::new(0.2, 0.8),
            Vector2::new(0.3, 0.6),
            Vector2::new(0.5, 0.9),
            Vector2::new(1.0, 0.7),
        ],
        vec![
            Vector2::new(0.0, 1.0),
            Vector2::new(0.1, 1.0),
            Vector2::new(0.5, 1.0),
            Vector2::new(0.7, 1.0),
            Vector2::new(1.0, 1.0),
        ],
    ];
    let surface = BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts);

    let knot_vec0 = KnotVec::bezier_knot(2);
    let ctrl_pts0 = vec![
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(0.0, 1.0),
    ];
    let curve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    assert!(surface.include(&curve0));

    let knot_vec1 = KnotVec::bezier_knot(2);
    let ctrl_pts1 = vec![
        Vector2::new(0.0, 0.0),
        Vector2::new(2.5, 1.0),
        Vector2::new(0.0, 1.0),
    ];
    let curve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    assert!(!surface.include(&curve1));
}

#[test]
fn test_include_bspcurve3() {
    use std::iter::FromIterator;
    let knot_vec = KnotVec::uniform_knot(2, 3);
    let ctrl_pts = vec![
        vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.1, 0.0, 0.5),
            Vector3::new(0.5, 0.0, 0.3),
            Vector3::new(1.0, 0.0, 1.0),
        ],
        vec![
            Vector3::new(0.0, 0.1, 0.1),
            Vector3::new(0.2, 0.2, 0.1),
            Vector3::new(0.4, 0.3, 0.4),
            Vector3::new(1.0, 0.3, 0.7),
        ],
        vec![
            Vector3::new(0.0, 0.5, 0.4),
            Vector3::new(0.3, 0.6, 0.5),
            Vector3::new(0.6, 0.4, 1.0),
            Vector3::new(1.0, 0.5, 0.0),
        ],
        vec![
            Vector3::new(0.0, 1.0, 1.0),
            Vector3::new(0.1, 1.0, 1.0),
            Vector3::new(0.5, 1.0, 0.5),
            Vector3::new(1.0, 1.0, 0.3),
        ],
    ];
    let surface = BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts);
    let bnd_box = BoundingBox::from_iter(&[Vector2::new(0.2, 0.3), Vector2::new(0.8, 0.6)]);
    let mut curve = surface.sectional_curve(bnd_box);
    assert!(surface.include(&curve));
    *curve.control_point_mut(2) += Vector3::new(0.0, 0.0, 0.001);
    assert!(!surface.include(&curve));
}
