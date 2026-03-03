use super::*;
use algo::surface::{SearchNearestParameterVector, SearchParameterVector};

impl<V> NurbsSurface<V> {
    /// constructor
    #[inline(always)]
    pub const fn new(bspsurface: BsplineSurface<V>) -> Self { NurbsSurface(bspsurface) }

    /// Returns the nurbs surface before rationalized
    #[inline(always)]
    pub const fn non_rationalized(&self) -> &BsplineSurface<V> { &self.0 }
    /// Returns the nurbs surface before rationalized
    #[inline(always)]
    pub fn non_rationalized_mut(&mut self) -> &mut BsplineSurface<V> { &mut self.0 }

    /// Returns the nurbs surface before rationalized
    #[inline(always)]
    pub fn into_non_rationalized(self) -> BsplineSurface<V> { self.0 }

    /// Returns the reference of the knot vectors
    #[inline(always)]
    pub const fn knot_vecs(&self) -> &(KnotVector, KnotVector) { &self.0.knot_vecs }

    /// Returns the u knot vector.
    #[inline(always)]
    pub const fn knot_vector_u(&self) -> &KnotVector { &self.0.knot_vecs.0 }
    /// Returns the v knot vector.
    #[inline(always)]
    pub const fn knot_vector_v(&self) -> &KnotVector { &self.0.knot_vecs.1 }

    /// Returns the `idx`th u knot.
    #[inline(always)]
    pub fn knot_u(&self, idx: usize) -> f64 { self.0.knot_vecs.0[idx] }
    /// returns the `idx`th v knot.
    #[inline(always)]
    pub fn knot_v(&self, idx: usize) -> f64 { self.0.knot_vecs.1[idx] }

    /// Returns the reference of the vector of the control points
    #[inline(always)]
    pub const fn control_points(&self) -> &Vec<Vec<V>> { &self.0.control_points }

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
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vector_u = KnotVector::bezier_knot(1);
    /// let knot_vector_v = KnotVector::bezier_knot(2);
    /// let knot_vecs = (knot_vector_u, knot_vector_v);
    /// let control_points = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// ];
    /// let bspsurface = BsplineSurface::new(knot_vecs, control_points);
    /// let mut iter = bspsurface.control_points_row_iter(1);
    /// assert_eq!(iter.next(), Some(&Vector3::new(1.0, 0.0, 1.0)));
    /// assert_eq!(iter.next(), Some(&Vector3::new(1.0, 1.0, 1.0)));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline(always)]
    pub fn control_points_row_iter(
        &self,
        column_idx: usize,
    ) -> impl ExactSizeIterator<Item = &V> + std::iter::FusedIterator<Item = &V> {
        self.0.control_points_row_iter(column_idx)
    }

    /// Returns the iterator over the control points in the `row_idx`th row.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vector_u = KnotVector::bezier_knot(1);
    /// let knot_vector_v = KnotVector::bezier_knot(2);
    /// let knot_vecs = (knot_vector_u, knot_vector_v);
    /// let control_points = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// ];
    /// let bspsurface = BsplineSurface::new(knot_vecs, control_points);
    /// let mut iter = bspsurface.control_points_column_iter(1);
    /// assert_eq!(iter.next(), Some(&Vector3::new(0.0, 1.0, 0.0)));
    /// assert_eq!(iter.next(), Some(&Vector3::new(1.0, 1.0, 1.0)));
    /// assert_eq!(iter.next(), Some(&Vector3::new(2.0, 1.0, 2.0)));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline(always)]
    pub fn control_points_column_iter(&self, row_idx: usize) -> std::slice::Iter<'_, V> {
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
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vector_u = KnotVector::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let knot_vector_v = KnotVector::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vecs = (knot_vector_u, knot_vector_v);
    /// let control_points = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// ];
    /// let bspsurface = BsplineSurface::new(knot_vecs, control_points);
    /// assert_eq!(bspsurface.udegree(), 1);
    /// ```
    #[inline(always)]
    pub fn udegree(&self) -> usize { self.0.udegree() }

    /// Returns the degrees of B-spline surface
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vector_u = KnotVector::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let knot_vector_v = KnotVector::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vecs = (knot_vector_u, knot_vector_v);
    /// let control_points = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// ];
    /// let bspsurface = BsplineSurface::new(knot_vecs, control_points);
    /// assert_eq!(bspsurface.vdegree(), 2);
    /// ```
    #[inline(always)]
    pub fn vdegree(&self) -> usize { self.0.vdegree() }

    /// Returns the degrees of B-spline surface
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vector_u = KnotVector::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let knot_vector_v = KnotVector::from(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    /// let knot_vecs = (knot_vector_u, knot_vector_v);
    /// let control_points = vec![
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 1.0), Vector3::new(2.0, 0.0, 2.0)],
    ///     vec![Vector3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(2.0, 1.0, 2.0)],
    /// ];
    /// let bspsurface = BsplineSurface::new(knot_vecs, control_points);
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
    pub fn parameter_range(&self) -> (ParameterRange, ParameterRange) { self.0.parameter_range() }
    /// Creates the curve whose control points are the `idx`th column control points of `self`.
    #[inline(always)]
    pub fn column_curve(&self, row_idx: usize) -> NurbsCurve<V>
    where V: Clone {
        NurbsCurve(self.0.column_curve(row_idx))
    }
    /// Creates the column sectional curve.
    #[inline(always)]
    pub fn row_curve(&self, column_idx: usize) -> NurbsCurve<V>
    where V: Clone {
        NurbsCurve(self.0.row_curve(column_idx))
    }
}

impl<V: Homogeneous<Scalar = f64>> NurbsSurface<V> {
    /// Constructs a rationalization surface from the non-rationalized surface and weights.
    /// # Failures
    /// the length of `surface.control_points()` and `weights` must be the same.
    #[inline(always)]
    pub fn try_from_bspline_and_weights(
        surface: BsplineSurface<V::Point>,
        weights: Vec<Vec<f64>>,
    ) -> Result<Self> {
        let BsplineSurface {
            knot_vecs,
            control_points,
        } = surface;
        if control_points.len() != weights.len() {
            return Err(Error::DifferentLength);
        }
        let control_points = control_points
            .into_iter()
            .zip(weights)
            .map(|(control_points, weights)| {
                if control_points.len() != weights.len() {
                    return Err(Error::DifferentLength);
                }
                Ok(control_points
                    .into_iter()
                    .zip(weights)
                    .map(|(pt, w)| V::from_point_weight(pt, w))
                    .collect::<Vec<_>>())
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self(BsplineSurface::new_unchecked(
            knot_vecs,
            control_points,
        )))
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> NurbsSurface<V> {
    /// Returns the closure of substitution.
    #[inline(always)]
    pub fn get_closure(&self) -> impl Fn(f64, f64) -> V::Point + '_ { move |u, v| self.subs(u, v) }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> NurbsSurface<V>
where V::Point: Tolerance
{
    /// Returns whether constant curve or not, i.e. all control points are same or not.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vector_u = KnotVector::bezier_knot(1);
    /// let knot_vector_v = KnotVector::bezier_knot(2);
    /// let pt = Vector3::new(1.0, 2.0, 1.0);
    /// // allows differences upto scalars
    /// let control_points = vec![
    ///     vec![pt.clone(), pt.clone() * 2.0, pt.clone() * 3.0],
    ///     vec![pt.clone() * 0.5, pt.clone() * 0.25, pt.clone() * 0.125],
    /// ];
    /// let mut surface = NurbsSurface::new(BsplineSurface::new((knot_vector_u, knot_vector_v), control_points));
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
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vecs = (KnotVector::bezier_knot(3), KnotVector::bezier_knot(2));
    /// let control_points = vec![
    ///     vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.5, -1.0, 2.0), Vector3::new(1.0, 0.0, 1.0)],
    ///     vec![Vector3::new(0.0, 1.0, 1.0), Vector3::new(0.5, 1.0, 1.0), Vector3::new(1.0, 1.0, 1.0)],
    ///     vec![Vector3::new(0.0, 2.0, 1.0), Vector3::new(0.5, 2.0, 3.0), Vector3::new(1.0, 2.0, 1.0)],
    ///     vec![Vector3::new(0.0, 3.0, 1.0), Vector3::new(0.5, 3.5, 2.0), Vector3::new(1.0, 3.0, 1.0)],
    /// ];
    /// let surface0 = NurbsSurface::new(BsplineSurface::new(knot_vecs, control_points));
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
    /// use monstertruck_geometry::prelude::*;
    /// let eps = TOLERANCE;
    /// let knot_vecs = (KnotVector::bezier_knot(3), KnotVector::bezier_knot(2));
    /// let control_points = vec![
    ///     vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.5, -1.0, 2.0), Vector3::new(1.0, 0.0, 1.0)],
    ///     vec![Vector3::new(0.0, 1.0, 1.0), Vector3::new(0.5, 1.0, 1.0), Vector3::new(1.0, 1.0, 1.0)],
    ///     vec![Vector3::new(0.0, 2.0, 1.0), Vector3::new(0.5, 2.0, 3.0), Vector3::new(1.0, 2.0, 1.0)],
    ///     vec![Vector3::new(0.0, 3.0, 1.0), Vector3::new(0.5, 3.5, 2.0), Vector3::new(1.0, 3.0, 1.0)],
    /// ];
    /// let surface0 = NurbsSurface::new(BsplineSurface::new(knot_vecs, control_points));
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

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V> + Tolerance> NurbsSurface<V> {
    /// Adds a knot `x` of the first parameter `u`, and do not change `self` as a surface.
    #[inline(always)]
    pub fn add_knot_u(&mut self, x: f64) -> &mut Self {
        self.0.add_knot_u(x);
        self
    }
    /// Adds a knot `x` of the first parameter `u`, and do not change `self` as a surface.
    #[inline(always)]
    pub fn add_knot_v(&mut self, x: f64) -> &mut Self {
        self.0.add_knot_v(x);
        self
    }
    /// Removes the knot_u corresponding to the indice `idx`, and do not change `self` as a curve.
    /// If the knot cannot be removed, returns [`Error::CannotRemoveKnot`].
    #[inline(always)]
    pub fn try_remove_knot_u(&mut self, idx: usize) -> Result<&mut Self> {
        match self.0.try_remove_knot_u(idx) {
            Ok(_) => Ok(self),
            Err(error) => Err(error),
        }
    }
    /// Removes the knot_u corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    #[inline(always)]
    pub fn remove_knot_u(&mut self, idx: usize) -> &mut Self {
        self.0.remove_knot_u(idx);
        self
    }
    /// Removes the knot_u corresponding to the indice `idx`, and do not change `self` as a curve.
    /// If the knot cannot be removed, returns [`Error::CannotRemoveKnot`].
    #[inline(always)]
    pub fn try_remove_knot_v(&mut self, idx: usize) -> Result<&mut Self> {
        match self.0.try_remove_knot_v(idx) {
            Ok(_) => Ok(self),
            Err(error) => Err(error),
        }
    }
    /// Removes the knot_u corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    #[inline(always)]
    pub fn remove_knot_v(&mut self, idx: usize) -> &mut Self {
        self.0.remove_knot_v(idx);
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
    /// Makes the knot_u vector and the knot_v vector the same knot vector.
    #[inline(always)]
    pub fn syncro_uvknots(&mut self) -> &mut Self {
        self.0.syncro_uvknots();
        self
    }

    /// Cuts the surface into two surfaces at the parameter `u`
    #[inline(always)]
    pub fn cut_u(&mut self, u: f64) -> Self { Self::new(self.0.cut_u(u)) }

    /// Cuts the surface into two surfaces at the parameter `v`
    #[inline(always)]
    pub fn cut_v(&mut self, v: f64) -> Self { Self::new(self.0.cut_v(v)) }

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
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vecs = (KnotVector::bezier_knot(3), KnotVector::bezier_knot(2));
    /// let control_points = vec![
    ///     vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.5, -1.0, 2.0), Vector3::new(1.0, 0.0, 1.0)],
    ///     vec![Vector3::new(0.0, 1.0, 2.0), Vector3::new(0.5, 1.0, 3.0), Vector3::new(1.0, 1.0, 2.0)],
    ///     vec![Vector3::new(0.0, 2.0, 2.0), Vector3::new(0.5, 2.0, 3.0), Vector3::new(1.0, 2.0, 2.0)],
    ///     vec![Vector3::new(0.0, 3.0, 1.0), Vector3::new(0.5, 3.5, 2.0), Vector3::new(1.0, 3.0, 1.0)],
    /// ];
    /// let bspsurface = NurbsSurface::new(BsplineSurface::new(knot_vecs, control_points));
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
    pub fn splitted_boundary(&self) -> [NurbsCurve<V>; 4] {
        TryFrom::try_from(
            self.0
                .splitted_boundary()
                .iter()
                .cloned()
                .map(NurbsCurve::new)
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    /// Extracts the boundary of surface
    #[inline(always)]
    pub fn boundary(&self) -> NurbsCurve<V> { NurbsCurve::new(self.0.boundary()) }
}

impl<V: Homogeneous<Scalar = f64>> SearchNearestParameter<D2> for NurbsSurface<V>
where
    Self: ParametricSurface<Point = V::Point, Vector = <V::Point as EuclideanSpace>::Diff>,
    V::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>,
    <V::Point as EuclideanSpace>::Diff: SearchNearestParameterVector<Point = V::Point>,
{
    type Point = V::Point;
    /// Searches the parameter `(u, v)` which minimize `|self(u, v) - point|` by Newton's method
    /// with initial guess `(u0, v0)`. If the repeated trial does not converge, then returns `None`.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vecs = (KnotVector::bezier_knot(3), KnotVector::bezier_knot(2));
    /// let control_points = vec![
    ///     vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(1.0, -2.0, 2.0), Vector3::new(1.0, 0.0, 1.0)],
    ///     vec![Vector3::new(0.0, 2.0, 2.0), Vector3::new(2.0, 4.0, 4.0), Vector3::new(2.0, 2.0, 2.0)],
    ///     vec![Vector3::new(0.0, 4.0, 2.0), Vector3::new(2.0, 8.0, 4.0), Vector3::new(2.0, 4.0, 2.0)],
    ///     vec![Vector3::new(0.0, 3.0, 1.0), Vector3::new(1.0, 7.0, 2.0), Vector3::new(1.0, 3.0, 1.0)],
    /// ];
    /// let surface = NurbsSurface::new(BsplineSurface::new(knot_vecs, control_points));
    /// let pt = surface.subs(0.3, 0.7);
    /// let (u, v) = surface.search_nearest_parameter(pt, Some((0.5, 0.5)), 100).unwrap();
    /// assert!(u.near(&0.3) && v.near(&0.7));
    /// ```
    /// # Remarks
    /// It may converge to a local solution depending on the hint.
    /// cf. [`BsplineCurve::search_rational_nearest_parameter`](struct.BsplineCurve.html#method.search_rational_nearest_parameter)
    #[inline(always)]
    fn search_nearest_parameter<H: Into<SearchParameterHint2D>>(
        &self,
        point: V::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SearchParameterHint2D::Parameter(x, y) => (x, y),
            SearchParameterHint2D::Range(range0, range1) => {
                algo::surface::presearch(self, point, (range0, range1), PRESEARCH_DIVISION)
            }
            SearchParameterHint2D::None => {
                algo::surface::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::surface::search_nearest_parameter(self, point, hint, trials)
    }
}

impl<V: Homogeneous<Scalar = f64>> NurbsSurface<V>
where V::Point: Bounded<Scalar = f64>
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

impl<V: Clone> Invertible for NurbsSurface<V> {
    #[inline(always)]
    fn invert(&mut self) { self.swap_axes(); }
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut surface = self.clone();
        surface.swap_axes();
        surface
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> ParametricSurface
    for NurbsSurface<V>
{
    type Point = V::Point;
    type Vector = <V::Point as EuclideanSpace>::Diff;
    #[inline(always)]
    fn derivative_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        if m < 7 && n < 7 {
            let mut ders = [[V::zero(); 8]; 8];
            (0..=m).for_each(|i| (0..=n).for_each(|j| ders[i][j] = self.0.derivative_mn(i, j, u, v)));
            let ders = std::array::from_fn::<_, 8, _>(|i| &ders[i][..=n]);
            multi_rat_der(&ders[..=m])
        } else {
            let ders = (0..=m)
                .map(|i| (0..=n).map(|j| self.0.derivative_mn(i, j, u, v)).collect())
                .collect::<Vec<Vec<_>>>();
            multi_rat_der(&ders)
        }
    }
    #[inline(always)]
    fn evaluate(&self, u: f64, v: f64) -> V::Point { self.0.evaluate(u, v).to_point() }
    #[inline(always)]
    fn derivative_u(&self, u: f64, v: f64) -> Self::Vector {
        rat_der(&[self.0.evaluate(u, v), self.0.derivative_u(u, v)])
    }
    #[inline(always)]
    fn derivative_v(&self, u: f64, v: f64) -> <V::Point as EuclideanSpace>::Diff {
        rat_der(&[self.0.evaluate(u, v), self.0.derivative_v(u, v)])
    }
    #[inline(always)]
    fn derivative_uu(&self, u: f64, v: f64) -> <V::Point as EuclideanSpace>::Diff {
        rat_der(&[self.0.evaluate(u, v), self.0.derivative_u(u, v), self.0.derivative_uu(u, v)])
    }
    #[inline(always)]
    fn derivative_uv(&self, u: f64, v: f64) -> <V::Point as EuclideanSpace>::Diff {
        multi_rat_der(&[
            [self.0.evaluate(u, v), self.0.derivative_v(u, v)],
            [self.0.derivative_u(u, v), self.0.derivative_uv(u, v)],
        ])
    }
    #[inline(always)]
    fn derivative_vv(&self, u: f64, v: f64) -> <V::Point as EuclideanSpace>::Diff {
        rat_der(&[self.0.evaluate(u, v), self.0.derivative_v(u, v), self.0.derivative_vv(u, v)])
    }
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) { self.parameter_range() }
}

impl ParametricSurface3D for NurbsSurface<Vector4> {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        let pt = self.0.evaluate(u, v);
        let ud = self.0.derivative_u(u, v);
        let vd = self.0.derivative_v(u, v);
        rat_der(&[pt, ud]).cross(rat_der(&[pt, vd])).normalize()
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> ParameterDivision2D
    for NurbsSurface<V>
where V::Point: MetricSpace<Metric = f64> + HashGen<f64>
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

impl<V> BoundedSurface for NurbsSurface<V> where Self: ParametricSurface {}

impl IncludeCurve<NurbsCurve<Vector3>> for NurbsSurface<Vector3> {
    #[inline(always)]
    fn include(&self, curve: &NurbsCurve<Vector3>) -> bool {
        let pt = curve.subs(curve.knot_vec()[0]);
        let mut hint = match self.search_parameter(pt, None, INCLUDE_CURVE_TRIALS) {
            Some(got) => got,
            None => return false,
        };
        let knot_vector_u = self.knot_vector_u();
        let knot_vector_v = self.knot_vector_v();
        let degree = curve.degree() * 6;
        let (knots, _) = curve.knot_vec().to_single_multi();
        for i in 1..knots.len() {
            for j in 1..=degree {
                let p = j as f64 / degree as f64;
                let t = knots[i - 1] * (1.0 - p) + knots[i] * p;
                let pt = curve.subs(t);
                hint = match self.search_parameter(pt, Some(hint), INCLUDE_CURVE_TRIALS) {
                    Some(got) => got,
                    None => return false,
                };
                if !self.subs(hint.0, hint.1).near(&pt)
                    || hint.0 < knot_vector_u[0] - TOLERANCE
                    || hint.0 - knot_vector_u[0] > knot_vector_u.range_length() + TOLERANCE
                    || hint.1 < knot_vector_v[0] - TOLERANCE
                    || hint.1 - knot_vector_v[0] > knot_vector_v.range_length() + TOLERANCE
                {
                    return false;
                }
            }
        }
        true
    }
}

impl IncludeCurve<BsplineCurve<Point3>> for NurbsSurface<Vector4> {
    #[inline(always)]
    fn include(&self, curve: &BsplineCurve<Point3>) -> bool {
        let pt = curve.front();
        let mut hint = match self.search_parameter(pt, None, INCLUDE_CURVE_TRIALS) {
            Some(got) => got,
            None => return false,
        };
        let knot_vector_u = self.knot_vector_u();
        let knot_vector_v = self.knot_vector_v();
        let degree = curve.degree() * 6;
        let (knots, _) = curve.knot_vec().to_single_multi();
        for i in 1..knots.len() {
            for j in 1..=degree {
                let p = j as f64 / degree as f64;
                let t = knots[i - 1] * (1.0 - p) + knots[i] * p;
                let pt = curve.subs(t);
                hint = match self.search_parameter(pt, Some(hint), INCLUDE_CURVE_TRIALS) {
                    Some(got) => got,
                    None => return false,
                };
                if !self.subs(hint.0, hint.1).near(&pt)
                    || hint.0 < knot_vector_u[0] - TOLERANCE
                    || hint.0 - knot_vector_u[0] > knot_vector_u.range_length() + TOLERANCE
                    || hint.1 < knot_vector_v[0] - TOLERANCE
                    || hint.1 - knot_vector_v[0] > knot_vector_v.range_length() + TOLERANCE
                {
                    return false;
                }
            }
        }
        true
    }
}

impl IncludeCurve<NurbsCurve<Vector4>> for NurbsSurface<Vector4> {
    #[inline(always)]
    fn include(&self, curve: &NurbsCurve<Vector4>) -> bool {
        let pt = curve.front();
        let mut hint = match self.search_parameter(pt, None, INCLUDE_CURVE_TRIALS) {
            Some(got) => got,
            None => return false,
        };
        let knot_vector_u = self.knot_vector_u();
        let knot_vector_v = self.knot_vector_v();
        let degree = curve.degree() * 6;
        let (knots, _) = curve.knot_vec().to_single_multi();
        for i in 1..knots.len() {
            for j in 1..=degree {
                let p = j as f64 / degree as f64;
                let t = knots[i - 1] * (1.0 - p) + knots[i] * p;
                let pt = curve.subs(t);
                hint = match self.search_parameter(pt, Some(hint), INCLUDE_CURVE_TRIALS) {
                    Some(got) => got,
                    None => return false,
                };
                if !self.subs(hint.0, hint.1).near(&pt)
                    || hint.0 < knot_vector_u[0] - TOLERANCE
                    || hint.0 - knot_vector_u[0] > knot_vector_u.range_length() + TOLERANCE
                    || hint.1 < knot_vector_v[0] - TOLERANCE
                    || hint.1 - knot_vector_v[0] > knot_vector_v.range_length() + TOLERANCE
                {
                    return false;
                }
            }
        }
        true
    }
}

impl<M, V: Copy> Transformed<M> for NurbsSurface<V>
where M: Copy + std::ops::Mul<V, Output = V>
{
    #[inline(always)]
    fn transform_by(&mut self, trans: M) {
        self.0
            .control_points
            .iter_mut()
            .flatten()
            .for_each(move |v| *v = trans * *v)
    }
}

impl<V> SearchParameter<D2> for NurbsSurface<V>
where
    V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>,
    V::Point: ControlPoint<f64, Diff = <V::Point as EuclideanSpace>::Diff>
        + MetricSpace<Metric = f64>
        + Tolerance,
    <V::Point as EuclideanSpace>::Diff: SearchParameterVector<Point = V::Point>,
{
    type Point = V::Point;
    /// Search the parameter `(u, v)` such that `self.subs(u, v).rational_projection()` is near `pt`.
    /// If cannot find, then return `None`.
    /// # Examples
    /// ```
    /// use monstertruck_geometry::prelude::*;
    /// let knot_vec = KnotVector::uniform_knot(2, 2);
    /// let control_points = vec![
    ///     vec![Vector4::new(0.0, 0.0, 0.0, 1.0), Vector4::new(0.1, 0.0, 0.5, 0.4), Vector4::new(1.0, 0.0, 0.6, 2.0), Vector4::new(0.4, 0.0, 0.4, 0.4)],
    ///     vec![Vector4::new(0.0, 0.2, 0.2, 2.0), Vector4::new(0.24, 0.24, 0.1, 1.2), Vector4::new(2.4, 1.8, 2.4, 0.6), Vector4::new(1.4, 0.42, 0.98, 1.4)],
    ///     vec![Vector4::new(0.0, 1.5, 1.2, 3.0), Vector4::new(1.02, 2.04, 1.7, 3.4), Vector4::new(0.42, 0.28, 0.7, 0.7), Vector4::new(0.6, 0.3, 0.0, 0.6)],
    ///     vec![Vector4::new(0.0, 1.0, 1.0, 1.0), Vector4::new(0.2, 2.0, 2.0, 2.0), Vector4::new(0.85, 1.7, 0.85, 1.7), Vector4::new(1.0, 1.0, 0.3, 1.0)],
    /// ];
    /// let bspsurface = BsplineSurface::new((knot_vec.clone(), knot_vec), control_points);
    /// let surface = NurbsSurface::new(bspsurface);
    ///
    /// let pt = surface.subs(0.3, 0.7);
    /// let (u, v) = surface.search_parameter(pt, Some((0.5, 0.5)), 100).unwrap();
    /// assert_near!(surface.subs(u, v), pt);
    /// ```
    fn search_parameter<H: Into<SearchParameterHint2D>>(
        &self,
        point: V::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SearchParameterHint2D::Parameter(x, y) => (x, y),
            SearchParameterHint2D::Range(range0, range1) => {
                algo::surface::presearch(self, point, (range0, range1), PRESEARCH_DIVISION)
            }
            SearchParameterHint2D::None => {
                algo::surface::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::surface::search_parameter(self, point, hint, trials)
    }
}

impl<V: Homogeneous<Scalar = f64>> From<BsplineSurface<V::Point>> for NurbsSurface<V> {
    fn from(bsp: BsplineSurface<V::Point>) -> Self {
        let control_points = bsp
            .control_points
            .into_iter()
            .map(|vec| vec.into_iter().map(|p| V::from_point(p)).collect())
            .collect();
        Self(BsplineSurface {
            knot_vecs: bsp.knot_vecs,
            control_points,
        })
    }
}

#[test]
fn test_include2d() {
    let knot_vec = KnotVector::uniform_knot(2, 3);
    let control_points = vec![
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
    let surface = BsplineSurface::new((knot_vec.clone(), knot_vec), control_points);
    let bnd_box = BoundingBox::from_iter(&[Vector2::new(0.2, 0.3), Vector2::new(0.8, 0.6)]);
    let mut curve = surface.sectional_curve(bnd_box);
    curve.control_points_mut().for_each(|pt| *pt *= 3.0);
    let surface = NurbsSurface::new(surface);
    let curve = NurbsCurve::new(curve);
    assert!(surface.include(&curve));
}

#[test]
fn test_include3d() {
    let knot_vec = KnotVector::bezier_knot(2);
    let control_points = vec![
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
    let surface = NurbsSurface::new(BsplineSurface::new((knot_vec.clone(), knot_vec), control_points));

    let knot_vec = KnotVector::from(vec![
        0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0,
    ]);
    let control_points = vec![
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
    let mut curve = NurbsCurve::new(BsplineCurve::new(knot_vec, control_points));
    assert!(surface.include(&curve));
    *curve.control_point_mut(1) += Vector4::new(0.0, 0.0, 0.00001, 0.0);
    assert!(!surface.include(&curve));
}
