use super::*;
use crate::errors::Error;
use algo::surface::{SsnpVector, SspVector};
use std::iter::FusedIterator;
use std::ops::*;

impl<P> BSplineSurface<P> {
    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Panics
    /// There are 3 rules for construct B-spline curve.
    /// * The number of knots is less than or equal to the one of control points.
    /// * There exist at least two different knots.
    /// * There are at least one control point.
    #[inline(always)]
    pub fn new(knot_vecs: (KnotVec, KnotVec), control_points: Vec<Vec<P>>) -> BSplineSurface<P> {
        BSplineSurface::try_new(knot_vecs, control_points).unwrap_or_else(|e| panic!("{}", e))
    }

    /// constructor.
    /// # Arguments
    /// * `knot_vecs` - the knot vectors
    /// * `control_points` - the vector of the control points
    /// # Failures
    /// There are 3 rules for construct B-spline curve.
    /// * The number of knots is less than or equal to the one of control points.
    /// * There exist at least two different knots.
    /// * There are at least one control point.
    #[inline(always)]
    pub fn try_new(
        knot_vecs: (KnotVec, KnotVec),
        control_points: Vec<Vec<P>>,
    ) -> Result<BSplineSurface<P>> {
        if control_points.is_empty() || control_points[0].is_empty() {
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
            if control_points.iter().any(|vec| vec.len() != len) {
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
        control_points: Vec<Vec<P>>,
    ) -> BSplineSurface<P> {
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
        control_points: Vec<Vec<P>>,
    ) -> BSplineSurface<P> {
        match cfg!(debug_assertions) {
            true => Self::new(knot_vecs, control_points),
            false => Self::new_unchecked(knot_vecs, control_points),
        }
    }
    /// Returns the reference of the knot vectors
    #[inline(always)]
    pub const fn knot_vecs(&self) -> &(KnotVec, KnotVec) { &self.knot_vecs }

    /// Returns the u knot vector.
    #[inline(always)]
    pub const fn uknot_vec(&self) -> &KnotVec { &self.knot_vecs.0 }
    /// Returns the v knot vector.
    #[inline(always)]
    pub const fn vknot_vec(&self) -> &KnotVec { &self.knot_vecs.1 }

    /// Returns the `idx`th u knot.
    #[inline(always)]
    pub fn uknot(&self, idx: usize) -> f64 { self.knot_vecs.0[idx] }
    /// returns the `idx`th v knot.
    #[inline(always)]
    pub fn vknot(&self, idx: usize) -> f64 { self.knot_vecs.1[idx] }

    /// Returns the reference of the vector of the control points
    #[inline(always)]
    pub const fn control_points(&self) -> &Vec<Vec<P>> { &self.control_points }

    /// Returns the reference of the control point corresponding to the index `(idx0, idx1)`.
    #[inline(always)]
    pub fn control_point(&self, idx0: usize, idx1: usize) -> &P { &self.control_points[idx0][idx1] }
    /// Apply the given transformation to all control points.
    #[inline(always)]
    pub fn transform_control_points<F: FnMut(&mut P)>(&mut self, f: F) {
        self.control_points.iter_mut().flatten().for_each(f)
    }

    /// Returns the iterator over the control points in the `column_idx`th row.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn ctrl_pts_row_iter(
        &self,
        column_idx: usize,
    ) -> impl ExactSizeIterator<Item = &P> + FusedIterator<Item = &P> {
        self.control_points.iter().map(move |vec| &vec[column_idx])
    }

    /// Returns the iterator over the control points in the `row_idx`th row.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn ctrl_pts_column_iter(&self, row_idx: usize) -> std::slice::Iter<'_, P> {
        self.control_points[row_idx].iter()
    }

    /// Returns the mutable reference of the control point corresponding to index `(idx0, idx1)`.
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx0: usize, idx1: usize) -> &mut P {
        &mut self.control_points[idx0][idx1]
    }

    /// Returns the iterator on all control points
    #[inline(always)]
    pub fn control_points_mut(&mut self) -> impl Iterator<Item = &mut P> {
        self.control_points.iter_mut().flatten()
    }
    /// Returns the degrees of B-spline surface
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    /// use truck_geometry::prelude::*;
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
    /// use truck_geometry::prelude::*;
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
    /// use truck_geometry::prelude::*;
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
    where P: Clone {
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
    pub fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        // For B-splines, this is [knot[degree], knot[n_cv]] in each direction,
        // which is the valid evaluation domain.
        let udeg = self.knot_vecs.0.len() - self.control_points.len() - 1;
        let vdeg = self.knot_vecs.1.len() - self.control_points[0].len() - 1;
        (
            (
                Bound::Included(self.knot_vecs.0[udeg]),
                Bound::Included(self.knot_vecs.0[self.control_points.len()]),
            ),
            (
                Bound::Included(self.knot_vecs.1[vdeg]),
                Bound::Included(self.knot_vecs.1[self.control_points[0].len()]),
            ),
        )
    }
    /// Creates the curve whose control points are the `idx`th column control points of `self`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn column_curve(&self, row_idx: usize) -> BSplineCurve<P>
    where P: Clone {
        let knot_vec = self.vknot_vec().clone();
        let ctrl_pts = self.control_points[row_idx].clone();
        BSplineCurve::new_unchecked(knot_vec, ctrl_pts)
    }
    /// Creates the column sectional curve.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn row_curve(&self, column_idx: usize) -> BSplineCurve<P>
    where P: Clone {
        let knot_vec = self.uknot_vec().clone();
        let ctrl_pts: Vec<_> = self.ctrl_pts_row_iter(column_idx).cloned().collect();
        BSplineCurve::new_unchecked(knot_vec, ctrl_pts)
    }
}

impl<P: ControlPoint<f64>> BSplineSurface<P> {
    /// Returns the closure of substitution.
    #[inline(always)]
    pub fn get_closure(&self) -> impl Fn(f64, f64) -> P + '_ { move |u, v| self.subs(u, v) }

    #[inline(always)]
    fn udelta_control_points(&self, i: usize, j: usize) -> P::Diff {
        if i == 0 {
            self.control_point(i, j).to_vec()
        } else if i == self.control_points.len() {
            self.control_point(i - 1, j).to_vec() * (-1.0)
        } else {
            *self.control_point(i, j) - *self.control_point(i - 1, j)
        }
    }

    #[inline(always)]
    fn vdelta_control_points(&self, i: usize, j: usize) -> P::Diff {
        if j == 0 {
            self.control_point(i, j).to_vec()
        } else if j == self.control_points[0].len() {
            self.control_point(i, j - 1).to_vec() * (-1.0)
        } else {
            *self.control_point(i, j) - *self.control_point(i, j - 1)
        }
    }

    /// Calculate derived B-spline surface by the first parameter `u`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn uderivation(&self) -> BSplineSurface<P::Diff> {
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
            vec![vec![P::Diff::zero(); n1]; n0]
        };

        BSplineSurface::new_unchecked((uknot_vec, vknot_vec), new_points)
    }

    /// Calculate derived B-spline surface by the second parameter `v`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn vderivation(&self) -> BSplineSurface<P::Diff> {
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
            vec![vec![P::Diff::zero(); n1]; n0]
        };

        BSplineSurface::new_unchecked((uknot_vec, vknot_vec), new_points)
    }

    pub(super) fn sub_near_as_surface<F: Fn(&P, &P) -> bool>(
        &self,
        other: &BSplineSurface<P>,
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

impl<V: Homogeneous> BSplineSurface<V> {
    /// lift up control points to homogeneous coordinate.
    pub fn lift_up(surface: BSplineSurface<V::Point>) -> Self {
        let control_points = surface
            .control_points
            .into_iter()
            .map(|vec| vec.into_iter().map(V::from_point).collect())
            .collect();
        BSplineSurface::new_unchecked(surface.knot_vecs, control_points)
    }
}

impl<P: ControlPoint<f64>> ParametricSurface for BSplineSurface<P> {
    type Point = P;
    type Vector = P::Diff;
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        let (degree0, degree1) = self.degrees();
        let BSplineSurface {
            knot_vecs: (ref uknot_vec, ref vknot_vec),
            ref control_points,
        } = self;
        let basis0 = uknot_vec.bspline_basis_functions(degree0, m, u);
        let basis1 = vknot_vec.bspline_basis_functions(degree1, n, v);
        let mut sum = P::Diff::zero();
        for (vec, b0) in control_points.iter().zip(basis0) {
            for (p, b1) in vec.iter().zip(&basis1) {
                sum += p.to_vec() * (b0 * b1);
            }
        }
        sum
    }
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> P { P::from_vec(self.der_mn(0, 0, u, v)) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> P::Diff { self.der_mn(1, 0, u, v) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> P::Diff { self.der_mn(0, 1, u, v) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> P::Diff { self.der_mn(2, 0, u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> P::Diff { self.der_mn(1, 1, u, v) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> P::Diff { self.der_mn(0, 2, u, v) }

    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) { self.parameter_range() }
}

impl<V: Tolerance> BSplineSurface<V> {
    /// Returns whether all control points are same or not.
    /// If the knot vector is clamped, it means whether the curve is constant or not.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    /// use truck_geometry::prelude::*;
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

impl<P: ControlPoint<f64> + Tolerance> BSplineSurface<P> {
    /// Adds a knot `x` of the first parameter `u`, and do not change `self` as a surface.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
            control_points.insert(0, vec![P::origin(); n1]);
            return self;
        }

        let idx = uknot_vec.add_knot(x);
        let start = idx.saturating_sub(k);
        let end = if idx > n0 {
            control_points.push(vec![P::origin(); n1]);
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
                self.control_points[i0][j] -= p;
            }
        }
        self
    }

    /// Adds a knot `x` for the second parameter, and do not change `self` as a surface.
    /// Return `false` if cannot add the knot, i.e.
    /// * the index of `x` will be lower than the degree, or
    /// * the index of `x` will be higher than the number of control points.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
                .for_each(|vec| vec.insert(0, P::origin()));
            return self;
        }

        let k = self.vdegree();
        let n0 = self.control_points.len();
        let n1 = self.control_points[0].len();

        let idx = self.knot_vecs.1.add_knot(x);
        let start = idx.saturating_sub(k);
        let end = if idx > n1 {
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.push(P::origin()));
            n1 + 1
        } else {
            self.control_points
                .iter_mut()
                .for_each(|vec| vec.insert(idx - 1, vec[idx - 1]));
            idx
        };
        for j in start..end {
            let j0 = end + start - j - 1;
            let delta = self.knot_vecs.1[j0 + k + 1] - self.knot_vecs.1[j0];
            let a = inv_or_zero(delta) * (self.knot_vecs.1[idx] - self.knot_vecs.1[j0]);
            for i in 0..n0 {
                let p = self.vdelta_control_points(i, j0) * (1.0 - a);
                self.control_points[i][j0] -= p;
            }
        }
        self
    }

    /// Removes the uknot corresponding to the indice `idx`, and do not change `self` as a curve.
    /// If the knot cannot be removed, returns
    /// [`Error::CannotRemoveKnot`](./errors/enum.Error.html#variant.CannotRemoveKnot).
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
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
            .cloned()
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
                    .map(|(pt0, pt1)| *pt1 + (*pt0 - *pt1) / a)
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
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
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
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
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
            .cloned()
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
                    .map(|(pt0, pt1)| *pt1 + (*pt0 - *pt1) / a)
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
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
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
    /// use truck_geometry::prelude::*;
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
    /// use truck_geometry::prelude::*;
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
    /// use truck_geometry::prelude::*;
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
    /// use truck_geometry::prelude::*;
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
        use std::cmp::Ordering;
        match usize::cmp(&ulen, &vlen) {
            Ordering::Less => {
                for _ in 0..vlen - ulen {
                    self.add_uknot(1.0);
                }
            }
            Ordering::Greater => {
                for _ in 0..ulen - vlen {
                    self.add_vknot(1.0);
                }
            }
            _ => {}
        }
        self
    }

    /// Cuts the surface into two surfaces at the parameter `u`
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn ucut(&mut self, mut u: f64) -> BSplineSurface<P> {
        let degree = self.udegree();

        let idx = match self.uknot_vec().floor(u) {
            Some(idx) => idx,
            None => {
                let bspline = self.clone();
                let uknot_vec = KnotVec::from(vec![u, self.vknot_vec()[0]]);
                let vknot_vec = self.vknot_vec().clone();
                let ctrl_pts = vec![vec![P::origin(); vknot_vec.len()]];
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
    /// use truck_geometry::prelude::*;
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
    pub fn vcut(&mut self, v: f64) -> BSplineSurface<P> {
        self.swap_axes();
        let mut res = self.ucut(v);
        self.swap_axes();
        res.swap_axes();
        res
    }

    /// Creates a sectional curve with normalized knot vector from the parameter `p` to the parameter `q`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn sectional_curve(&self, bnd_box: BoundingBox<Vector2>) -> BSplineCurve<P> {
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
        let mut cc = CurveCollector::Singleton;
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
                    (0..=k).fold(P::origin(), |sum, i| {
                        if i <= degree && k - i <= degree {
                            let coef = (comb[i] * comb[k - i]) as f64 / comb2[k] as f64;
                            sum + bspsurface.control_points[i][k - i].to_vec() * coef
                        } else {
                            sum
                        }
                    })
                })
                .collect();
            cc.concat(&BSplineCurve::new(knot_vec, ctrl_pts));
            if p + 1 != knots.len() {
                bspsurface = backup.unwrap().vcut(knots[p]);
            }
        }
        let mut curve: BSplineCurve<P> = cc.unwrap();
        curve.knot_normalize();
        curve
    }

    /// Creates a surface with normailized knot vectors connecting two curves.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
        mut bspcurve0: BSplineCurve<P>,
        mut bspcurve1: BSplineCurve<P>,
    ) -> BSplineSurface<P> {
        bspcurve0.syncro_degree(&mut bspcurve1);

        //bspcurve0.optimize();
        //bspcurve1.optimize();

        bspcurve0.syncro_knots(&mut bspcurve1);

        let uknot_vec = bspcurve0.knot_vec().clone();
        let vknot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
        let control_points: Vec<Vec<_>> = (0..bspcurve0.control_points().len())
            .map(|i| vec![*bspcurve0.control_point(i), *bspcurve1.control_point(i)])
            .collect();
        BSplineSurface::new_unchecked((uknot_vec, vknot_vec), control_points)
    }

    /// Creates a surface by its boundary.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    /// use truck_geometry::prelude::*;
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
        mut curve0: BSplineCurve<P>,
        mut curve1: BSplineCurve<P>,
        mut curve2: BSplineCurve<P>,
        mut curve3: BSplineCurve<P>,
    ) -> BSplineSurface<P> {
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
        let mut control_points = vec![curve3.control_points().clone()];
        let n = curve0.control_points().len();
        let m = curve3.control_points().len();
        for i in 1..(n - 1) {
            let u = (i as f64) / (n as f64);
            let pt0 = curve2.control_points[i]
                + (curve0.control_points[i] - curve2.control_points[i]) * u;
            let mut new_row = vec![*curve0.control_point(i)];
            for j in 1..(m - 1) {
                let v = (j as f64) / (m as f64);
                let pt1 = curve1.control_points[j]
                    + (curve3.control_points[j] - curve1.control_points[j]) * v;
                new_row.push(pt0 + (pt1 - pt0) / 2.0);
            }
            new_row.push(*curve2.control_point(i));
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

    /// Gets the boundary by four splitted curves.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn splitted_boundary(&self) -> [BSplineCurve<P>; 4] {
        let (uknot_vec, vknot_vec) = self.knot_vecs.clone();
        let control_points0 = self.control_points.iter().map(|x| x[0]).collect();
        let control_points1 = self.control_points.last().unwrap().clone();
        let control_points2 = self
            .control_points
            .iter()
            .map(|x| *x.last().unwrap())
            .collect();
        let control_points3 = self.control_points[0].clone();
        let curve0 = BSplineCurve::new_unchecked(uknot_vec.clone(), control_points0);
        let curve1 = BSplineCurve::new_unchecked(vknot_vec.clone(), control_points1);
        let mut curve2 = BSplineCurve::new_unchecked(uknot_vec, control_points2);
        let mut curve3 = BSplineCurve::new_unchecked(vknot_vec, control_points3);
        curve2.invert();
        curve3.invert();
        [curve0, curve1, curve2, curve3]
    }

    /// Extracts the boundary of surface
    pub fn boundary(&self) -> BSplineCurve<P> {
        let (uknot_vec, vknot_vec) = self.knot_vecs.clone();
        let (range0, range1) = (uknot_vec.range_length(), vknot_vec.range_length());
        let [bspline0, mut bspline1, mut bspline2, mut bspline3] = self.splitted_boundary();
        bspline2.invert();
        bspline3.invert();
        bspline0
            .concat(bspline1.knot_translate(range0))
            .concat(bspline2.knot_translate(range0 + range1))
            .concat(bspline3.knot_translate(range0 * 2.0 + range1))
    }
    /// Determines whether `self` and `other` is near as the B-spline surfaces or not.
    ///
    /// Divides each knot domain into the number of degree equal parts,
    /// and check `|self(u, v) - other(u, v)| < TOLERANCE` for each end points `(u, v)`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn near_as_surface(&self, other: &BSplineSurface<P>) -> bool {
        self.sub_near_as_surface(other, 1, |x, y| x.near(y))
    }
    /// Determines whether `self` and `other` is near in square order as the B-spline surfaces or not.
    ///
    /// Divides each knot domain into the number of degree equal parts,
    /// and check `|self(u, v) - other(u, v)| < TOLERANCE` for each end points `(u, v)`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
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
    pub fn near2_as_surface(&self, other: &BSplineSurface<P>) -> bool {
        self.sub_near_as_surface(other, 1, |x, y| x.near2(y))
    }
}

impl<V: Bounded> BSplineSurface<V> {
    /// Returns the bounding box including all control points.
    #[inline(always)]
    pub fn roughly_bounding_box(&self) -> BoundingBox<V> {
        self.control_points.iter().flatten().collect()
    }
}

impl<P: ControlPoint<f64>> ParameterDivision2D for BSplineSurface<P>
where P: EuclideanSpace<Scalar = f64, Diff = <P as ControlPoint<f64>>::Diff>
        + MetricSpace<Metric = f64>
        + HashGen<f64>
{
    #[inline(always)]
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        algo::surface::parameter_division(self, range, tol)
    }
}

impl ParametricSurface3D for BSplineSurface<Point3> {}

impl<V> BoundedSurface for BSplineSurface<V> where BSplineSurface<V>: ParametricSurface {}

impl<V: Clone> Invertible for BSplineSurface<V> {
    #[inline(always)]
    fn invert(&mut self) { self.swap_axes(); }
}

impl<P, V> SearchParameter<D2> for BSplineSurface<P>
where
    P: ControlPoint<f64, Diff = V>
        + EuclideanSpace<Scalar = f64, Diff = V>
        + MetricSpace<Metric = f64>
        + Tolerance,
    V: SspVector<Point = P>,
{
    type Point = P;
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: P,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SPHint2D::Parameter(x, y) => (x, y),
            SPHint2D::Range(range0, range1) => {
                algo::surface::presearch(self, point, (range0, range1), PRESEARCH_DIVISION)
            }
            SPHint2D::None => {
                algo::surface::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::surface::search_parameter(self, point, hint, trials)
    }
}

impl<P> SearchNearestParameter<D2> for BSplineSurface<P>
where
    P: ControlPoint<f64>
        + EuclideanSpace<Scalar = f64, Diff = <P as ControlPoint<f64>>::Diff>
        + MetricSpace<Metric = f64>,
    <P as ControlPoint<f64>>::Diff: SsnpVector<Point = P>,
{
    type Point = P;
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: P,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SPHint2D::Parameter(x, y) => (x, y),
            SPHint2D::Range(range0, range1) => {
                algo::surface::presearch(self, point, (range0, range1), PRESEARCH_DIVISION)
            }
            SPHint2D::None => {
                algo::surface::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::surface::search_nearest_parameter(self, point, hint, trials)
    }
}

impl IncludeCurve<BSplineCurve<Point2>> for BSplineSurface<Point2> {
    fn include(&self, curve: &BSplineCurve<Point2>) -> bool {
        let pt = curve.front();
        let mut hint = algo::surface::presearch(self, pt, self.range_tuple(), PRESEARCH_DIVISION);
        hint = match algo::surface::search_parameter(self, pt, hint, INCLUDE_CURVE_TRIALS) {
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
                let pt = ParametricCurve::subs(curve, t);
                hint = match algo::surface::search_parameter(self, pt, hint, INCLUDE_CURVE_TRIALS) {
                    Some(got) => got,
                    None => return false,
                };
                if !ParametricSurface::subs(self, hint.0, hint.1).near(&pt)
                    || hint.0 < uknot_vec[0] - TOLERANCE
                    || hint.0 - uknot_vec[0] > uknot_vec.range_length() + TOLERANCE
                    || hint.1 < vknot_vec[0] - TOLERANCE
                    || hint.1 - vknot_vec[0] > vknot_vec.range_length() + TOLERANCE
                {
                    return false;
                }
            }
        }
        true
    }
}

impl IncludeCurve<BSplineCurve<Point3>> for BSplineSurface<Point3> {
    fn include(&self, curve: &BSplineCurve<Point3>) -> bool {
        let pt = curve.front();
        let mut hint = algo::surface::presearch(self, pt, self.range_tuple(), PRESEARCH_DIVISION);
        hint = match algo::surface::search_parameter(self, pt, hint, INCLUDE_CURVE_TRIALS) {
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
                let pt = ParametricCurve::subs(curve, t);
                hint = match algo::surface::search_parameter(self, pt, hint, INCLUDE_CURVE_TRIALS) {
                    Some(got) => got,
                    None => return false,
                };
                if !ParametricSurface::subs(self, hint.0, hint.1).near(&pt)
                    || hint.0 < uknot_vec[0] - TOLERANCE
                    || hint.0 - uknot_vec[0] > uknot_vec.range_length() + TOLERANCE
                    || hint.1 < vknot_vec[0] - TOLERANCE
                    || hint.1 - vknot_vec[0] > vknot_vec.range_length() + TOLERANCE
                {
                    return false;
                }
            }
        }
        true
    }
}

impl IncludeCurve<NurbsCurve<Vector4>> for BSplineSurface<Point3> {
    fn include(&self, curve: &NurbsCurve<Vector4>) -> bool {
        let pt = curve.subs(curve.knot_vec()[0]);
        let mut hint = algo::surface::presearch(self, pt, self.range_tuple(), PRESEARCH_DIVISION);
        hint = match algo::surface::search_parameter(self, pt, hint, INCLUDE_CURVE_TRIALS) {
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
                hint = match algo::surface::search_parameter(self, pt, hint, INCLUDE_CURVE_TRIALS) {
                    Some(got) => got,
                    None => return false,
                };
                if !ParametricSurface::subs(self, hint.0, hint.1).near(&pt)
                    || hint.0 < uknot_vec[0] - TOLERANCE
                    || hint.0 - uknot_vec[0] > uknot_vec.range_length() + TOLERANCE
                    || hint.1 < vknot_vec[0] - TOLERANCE
                    || hint.1 - vknot_vec[0] > vknot_vec.range_length() + TOLERANCE
                {
                    return false;
                }
            }
        }
        true
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

impl<M, P: EuclideanSpace<Scalar = f64>> Transformed<M> for BSplineSurface<P>
where M: Transform<P>
{
    #[inline(always)]
    fn transform_by(&mut self, trans: M) {
        self.control_points
            .iter_mut()
            .flatten()
            .for_each(|p| *p = trans.transform_point(*p))
    }
}

impl<'de, P> Deserialize<'de> for BSplineSurface<P>
where P: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        #[derive(Deserialize)]
        struct BSplineSurface_<P> {
            knot_vecs: (KnotVec, KnotVec),
            control_points: Vec<Vec<P>>,
        }
        let BSplineSurface_ {
            knot_vecs,
            control_points,
        } = BSplineSurface_::<P>::deserialize(deserializer)?;
        Self::try_new(knot_vecs, control_points).map_err(serde::de::Error::custom)
    }
}

fn combinatorial(n: usize) -> Vec<usize> {
    let mut res = vec![1];
    for i in 1..=n {
        res.push(res[i - 1] * (n - i + 1) / i);
    }
    res
}

#[test]
fn test_include_bspcurve2() {
    let knot_vec = KnotVec::uniform_knot(2, 3);
    let ctrl_pts = vec![
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(0.1, 0.0),
            Point2::new(0.5, 0.0),
            Point2::new(0.7, 0.0),
            Point2::new(1.0, 0.0),
        ],
        vec![
            Point2::new(0.0, 0.1),
            Point2::new(0.2, 0.2),
            Point2::new(0.4, 0.3),
            Point2::new(0.6, 0.2),
            Point2::new(1.0, 0.3),
        ],
        vec![
            Point2::new(0.0, 0.5),
            Point2::new(0.3, 0.6),
            Point2::new(0.6, 0.4),
            Point2::new(0.9, 0.6),
            Point2::new(1.0, 0.5),
        ],
        vec![
            Point2::new(0.0, 0.7),
            Point2::new(0.2, 0.8),
            Point2::new(0.3, 0.6),
            Point2::new(0.5, 0.9),
            Point2::new(1.0, 0.7),
        ],
        vec![
            Point2::new(0.0, 1.0),
            Point2::new(0.1, 1.0),
            Point2::new(0.5, 1.0),
            Point2::new(0.7, 1.0),
            Point2::new(1.0, 1.0),
        ],
    ];
    let surface = BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts);

    let knot_vec0 = KnotVec::bezier_knot(2);
    let ctrl_pts0 = vec![
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(0.0, 1.0),
    ];
    let curve0 = BSplineCurve::new(knot_vec0, ctrl_pts0);
    assert!(surface.include(&curve0));

    let knot_vec1 = KnotVec::bezier_knot(2);
    let ctrl_pts1 = vec![
        Point2::new(0.0, 0.0),
        Point2::new(2.5, 1.0),
        Point2::new(0.0, 1.0),
    ];
    let curve1 = BSplineCurve::new(knot_vec1, ctrl_pts1);
    assert!(!surface.include(&curve1));
}

#[test]
fn test_include_bspcurve3() {
    let knot_vec = KnotVec::uniform_knot(2, 3);
    let ctrl_pts = vec![
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.1, 0.0, 0.5),
            Point3::new(0.5, 0.0, 0.3),
            Point3::new(1.0, 0.0, 1.0),
        ],
        vec![
            Point3::new(0.0, 0.1, 0.1),
            Point3::new(0.2, 0.2, 0.1),
            Point3::new(0.4, 0.3, 0.4),
            Point3::new(1.0, 0.3, 0.7),
        ],
        vec![
            Point3::new(0.0, 0.5, 0.4),
            Point3::new(0.3, 0.6, 0.5),
            Point3::new(0.6, 0.4, 1.0),
            Point3::new(1.0, 0.5, 0.0),
        ],
        vec![
            Point3::new(0.0, 1.0, 1.0),
            Point3::new(0.1, 1.0, 1.0),
            Point3::new(0.5, 1.0, 0.5),
            Point3::new(1.0, 1.0, 0.3),
        ],
    ];
    let surface = BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts);
    let bnd_box = BoundingBox::from_iter(&[Vector2::new(0.2, 0.3), Vector2::new(0.8, 0.6)]);
    let mut curve = surface.sectional_curve(bnd_box);
    assert!(surface.include(&curve));
    *curve.control_point_mut(2) += Vector3::new(0.0, 0.0, 0.001);
    assert!(!surface.include(&curve));
}
