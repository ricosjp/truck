use truck_geotrait::algo::TesselationSplitMethod;

use super::*;

impl<V> NurbsCurve<V> {
    /// Constructs the rationalized B-spline curve.
    #[inline(always)]
    pub const fn new(curve: BSplineCurve<V>) -> Self { NurbsCurve(curve) }

    /// Returns the BSpline curve before rationalized.
    #[inline(always)]
    pub const fn non_rationalized(&self) -> &BSplineCurve<V> { &self.0 }

    /// Returns the BSpline curve before rationalized.
    #[inline(always)]
    pub fn into_non_rationalized(self) -> BSplineCurve<V> { self.0 }

    /// Returns the reference of the knot vector. cf.[`BSplineCurve::knot_vec`]
    #[inline(always)]
    pub const fn knot_vec(&self) -> &KnotVec { &self.0.knot_vec }

    /// Returns the `idx`th knot. cf.[`BSplineCurve::knot`]
    #[inline(always)]
    pub fn knot(&self, idx: usize) -> f64 { self.0.knot_vec[idx] }

    /// Returns the reference of the control points. cf.[`BSplineCurve::control_points`]
    #[inline(always)]
    pub const fn control_points(&self) -> &Vec<V> { &self.0.control_points }

    /// Returns the reference of the control point corresponding to the index `idx`.
    /// cf.[`BSplineCurve::control_point`]
    #[inline(always)]
    pub fn control_point(&self, idx: usize) -> &V { &self.0.control_points[idx] }

    /// Returns the mutable reference of the control point corresponding to index `idx`.
    /// cf.[`BSplineCurve::control_point_mut`]
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx: usize) -> &mut V { &mut self.0.control_points[idx] }

    /// Returns the iterator on all control points. cf.[`BSplineCurve::control_points_mut`]
    #[inline(always)]
    pub fn control_points_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.0.control_points.iter_mut()
    }

    /// Applies the given transformation to all control points. cf.[`BSplineCurve::transform_control_points`]
    #[inline(always)]
    pub fn transform_control_points<F: FnMut(&mut V)>(&mut self, f: F) {
        self.0.transform_control_points(f)
    }

    /// Returns the degree of NURBS curve. cf.[`BSplineCurve::degree`]
    #[inline(always)]
    pub fn degree(&self) -> usize { self.0.degree() }

    /// Returns whether the knot vector is clamped or not. cf.[`BSplineCurve::is_clamped`]
    #[inline(always)]
    pub fn is_clamped(&self) -> bool { self.0.knot_vec.is_clamped(self.0.degree()) }

    /// Normalizes the knot vector. cf.[`BSplineCurve::knot_normalize`]
    #[inline(always)]
    pub fn knot_normalize(&mut self) -> &mut Self {
        self.0.knot_vec.try_normalize().unwrap();
        self
    }

    /// Translates the knot vector. cf.[`BSplineCurve::knot_translate`]
    #[inline(always)]
    pub fn knot_translate(&mut self, x: f64) -> &mut Self {
        self.0.knot_vec.translate(x);
        self
    }
}

impl<V: Homogeneous<Scalar = f64>> NurbsCurve<V> {
    /// Constructs a rationalization curve from the non-rationalized curve and weights.
    /// # Failures
    /// the length of `curve.control_points()` and `weights` must be the same.
    #[inline(always)]
    pub fn try_from_bspline_and_weights(
        curve: BSplineCurve<V::Point>,
        weights: Vec<f64>,
    ) -> Result<Self> {
        let BSplineCurve {
            knot_vec,
            control_points,
        } = curve;
        if control_points.len() != weights.len() {
            return Err(Error::DifferentLength);
        }
        let control_points = control_points
            .into_iter()
            .zip(weights)
            .map(|(pt, w)| V::from_point_weight(pt, w))
            .collect();
        Ok(Self(BSplineCurve::new_unchecked(knot_vec, control_points)))
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> NurbsCurve<V> {
    /// Returns the closure of substitution.
    #[inline(always)]
    pub fn get_closure(&self) -> impl Fn(f64) -> V::Point + '_ { move |t| self.subs(t) }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> NurbsCurve<V>
where V::Point: Tolerance
{
    /// Returns whether all control points are the same or not.
    /// If the knot vector is clamped, it means whether the curve is constant or not.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    ///
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let pt = Vector3::new(1.0, 2.0, 1.0);
    /// // allows differences upto scalars
    /// let mut ctrl_pts = vec![pt.clone(), pt.clone() * 2.0, pt.clone() * 3.0];
    /// let bspcurve = BSplineCurve::new(knot_vec.clone(), ctrl_pts.clone());
    /// assert!(!bspcurve.is_const());
    /// let const_curve = NurbsCurve::new(bspcurve);
    /// assert!(const_curve.is_const());
    ///
    /// ctrl_pts.push(Vector3::new(2.0, 3.0, 1.0));
    /// let curve = NurbsCurve::new(BSplineCurve::new(knot_vec.clone(), ctrl_pts.clone()));
    /// assert!(!curve.is_const());
    /// ```
    /// # Remarks
    /// If the knot vector is not clamped and the BSpline basis function is not partition of unity,
    /// then perhaps returns true even if the curve is not constant.
    /// ```
    /// use truck_geometry::prelude::*;
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
        let pt = self.0.control_points[0].to_point();
        self.0
            .control_points
            .iter()
            .all(move |vec| vec.to_point().near(&pt))
    }
    /// Determines whether `self` and `other` is near as the B-spline curves or not.
    ///
    /// Divides each knot interval into the number of degree equal parts,
    /// and check `|self(t) - other(t)| < TOLERANCE` for each end points `t`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 4.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector3::new(1.0, 1.0, 1.0),
    ///     Vector3::new(3.0, 2.0, 2.0),
    ///     Vector3::new(2.0, 3.0, 1.0),
    ///     Vector3::new(4.0, 5.0, 2.0),
    ///     Vector3::new(5.0, 4.0, 1.0),
    ///     Vector3::new(1.0, 1.0, 2.0),
    /// ];
    /// let curve0 = NurbsCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    /// let mut curve1 = curve0.clone();
    /// assert!(curve0.near_as_curve(&curve1));
    /// *curve1.control_point_mut(1) += Vector3::new(0.01, 0.0002, 0.0);
    /// assert!(!curve0.near_as_curve(&curve1));
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
    /// use truck_geometry::prelude::*;
    /// let knot_vec = KnotVec::from(
    ///     vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 4.0]
    /// );
    /// let ctrl_pts = vec![
    ///     Vector3::new(1.0, 1.0, 1.0),
    ///     Vector3::new(3.0, 2.0, 2.0),
    ///     Vector3::new(2.0, 3.0, 1.0),
    ///     Vector3::new(4.0, 5.0, 2.0),
    ///     Vector3::new(5.0, 4.0, 1.0),
    ///     Vector3::new(1.0, 1.0, 2.0),
    /// ];
    /// let curve0 = NurbsCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    /// let mut curve1 = curve0.clone();
    /// assert!(curve0.near_as_curve(&curve1));
    /// *curve1.control_point_mut(1) += Vector3::new(0.01, TOLERANCE, 0.0);
    /// assert!(!curve0.near2_as_curve(&curve1));
    /// ```
    #[inline(always)]
    pub fn near2_as_curve(&self, other: &Self) -> bool {
        self.0
            .sub_near_as_curve(&other.0, 2, move |x, y| x.to_point().near2(&y.to_point()))
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V> + Tolerance> NurbsCurve<V> {
    /// Adds a knot `x`, and do not change `self` as a curve. cf.[`BSplineCurve::add_knot`]
    pub fn add_knot(&mut self, x: f64) -> &mut Self {
        self.0.add_knot(x);
        self
    }

    /// Removes a knot corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.
    /// cf.[`BSplineCurve::remove_knot`]
    pub fn remove_knot(&mut self, idx: usize) -> &mut Self {
        let _ = self.try_remove_knot(idx);
        self
    }

    /// Removes a knot corresponding to the indice `idx`, and do not change `self` as a curve.
    /// If the knot cannot be removed, returns [`Error::CannotRemoveKnot`].
    /// cf.[`BSplineCurve::try_remove_knot`]
    pub fn try_remove_knot(&mut self, idx: usize) -> Result<&mut Self> {
        self.0.try_remove_knot(idx)?;
        Ok(self)
    }

    /// Elevates 1 degree. cf.[`BSplineCurve::elevate_degree`]
    pub fn elevate_degree(&mut self) -> &mut Self {
        self.0.elevate_degree();
        self
    }

    /// Makes the NURBS curve clamped. cf.[`BSplineCurve::clamp`]
    #[inline(always)]
    pub fn clamp(&mut self) -> &mut Self {
        self.0.clamp();
        self
    }

    /// Repeats `Self::try_remove_knot()` from the back knot in turn until the knot cannot be removed.
    /// cf.[`BSplineCurve::optimize`]
    pub fn optimize(&mut self) -> &mut Self {
        self.0.optimize();
        self
    }

    /// Makes two splines having the same degrees. cf.[`BSplineCurve::syncro_degree`]
    pub fn syncro_degree(&mut self, other: &mut Self) {
        let (degree0, degree1) = (self.degree(), other.degree());
        for _ in degree0..degree1 {
            self.elevate_degree();
        }
        for _ in degree1..degree0 {
            other.elevate_degree();
        }
    }

    /// Makes two splines having the same normalized knot vectors. cf.[`BSplineCurve::syncro_knots`]
    pub fn syncro_knots(&mut self, other: &mut Self) { self.0.syncro_knots(&mut other.0) }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V> + Tolerance> ParameterTransform
    for NurbsCurve<V>
{
    #[inline(always)]
    fn parameter_transform(&mut self, scalar: f64, r#move: f64) -> &mut Self {
        self.0.parameter_transform(scalar, r#move);
        self
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V> + Tolerance> Cut for NurbsCurve<V> {
    #[inline(always)]
    fn cut(&mut self, t: f64) -> Self { NurbsCurve(self.0.cut(t)) }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V> + Tolerance> Concat<NurbsCurve<V>>
    for NurbsCurve<V>
where <V as Homogeneous>::Point: Debug
{
    type Output = NurbsCurve<V>;
    fn try_concat(
        &self,
        other: &Self,
    ) -> std::result::Result<Self, ConcatError<<V as Homogeneous>::Point>> {
        let curve0 = self.clone();
        let mut curve1 = other.clone();
        let w0 = curve0.0.control_points.last().unwrap().weight();
        let w1 = curve1.0.control_points[0].weight();
        curve1.transform_control_points(|pt| *pt *= w0 / w1);
        match curve0.0.try_concat(&curve1.0) {
            Ok(curve) => Ok(NurbsCurve::new(curve)),
            Err(err) => Err(err.point_map(|v| v.to_point())),
        }
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V> + Tolerance> NurbsCurve<V>
where V::Point: Tolerance
{
    /// Makes the rational curve locally injective.
    /// # Example
    /// ```
    /// use truck_geometry::prelude::*;
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
    /// let mut curve = NurbsCurve::new(BSplineCurve::new(knot_vec, control_points));
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
    ///     let pt0 = curve.subs(t);
    ///     let pt1 = curve.subs(t + 1.0 / (N as f64));
    ///     flag = flag || pt0.near(&pt1);
    /// }
    /// // There does not exist t such that bspcurve(t) == bspcurve(t + 0.01).
    /// assert!(!flag);
    /// ```
    /// # Remarks
    /// If `self` is a constant curve, then does nothing.
    /// ```
    /// use truck_geometry::prelude::*;
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0]);
    /// let ctrl_pts = vec![
    ///     Vector3::new(1.0, 1.0, 1.0),
    ///     Vector3::new(2.0, 2.0, 2.0),
    ///     Vector3::new(3.0, 3.0, 3.0),
    ///     Vector3::new(4.0, 4.0, 4.0),
    /// ];
    /// let mut curve = NurbsCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    /// let org_curve = curve.clone();
    /// curve.make_locally_injective();
    /// assert_eq!(curve, org_curve);
    /// ```
    pub fn make_locally_injective(&mut self) -> &mut Self {
        let mut iter = self.0.bezier_decomposition().into_iter();
        while let Some(bezier) = iter.next().map(NurbsCurve::new) {
            if !bezier.is_const() {
                *self = bezier;
                break;
            }
        }
        let mut x = 0.0;
        for mut bezier in iter.map(NurbsCurve::new) {
            if bezier.is_const() {
                x += bezier.0.knot_vec.range_length();
            } else {
                let s0 = self.0.control_points.last().unwrap().weight();
                let s1 = bezier.0.control_points[0].weight();
                bezier
                    .0
                    .control_points
                    .iter_mut()
                    .for_each(move |vec| *vec *= s0 / s1);
                self.concat(bezier.knot_translate(-x));
            }
        }
        self
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> ParameterDivision1D
    for NurbsCurve<V>
where V::Point: MetricSpace<Metric = f64> + HashGen<f64>
{
    type Point = V::Point;
    #[inline(always)]
    fn parameter_division<T: TesselationSplitMethod>(
        &self,
        range: (f64, f64),
        split: T,
    ) -> (Vec<f64>, Vec<Self::Point>) {
        algo::curve::parameter_division(self, range, split)
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> SearchNearestParameter<D1>
    for NurbsCurve<V>
where
    V::Point: MetricSpace<Metric = f64>,
    <V::Point as EuclideanSpace>::Diff: InnerSpace + Tolerance,
{
    type Point = V::Point;
    /// Searches the parameter `t` which minimize |self(t) - point| by Newton's method with initial guess `hint`.
    /// Returns `None` if the number of attempts exceeds `trial` i.e. if `trial == 0`, then the trial is only one time.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    ///
    /// // Defines the half unit circle in x > 0.
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector3::new(0.0, -1.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 1.0)];
    /// let curve = NurbsCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    ///
    /// // search rational nearest parameter
    /// let pt = Point2::new(1.0, 2.0);
    /// let hint = 0.8;
    /// let t = curve.search_nearest_parameter(pt, Some(hint), 100).unwrap();
    ///
    /// // check the answer
    /// let res = curve.subs(t);
    /// let ans = Point2::from_vec(pt.to_vec().normalize());
    /// assert_near!(res, ans);
    /// ```
    /// # Remarks
    /// It may converge to a local solution depending on the hint.
    /// ```
    /// use truck_geometry::prelude::*;
    ///
    /// // Same curve and point as above example
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector3::new(0.0, -1.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 1.0)];
    /// let curve = NurbsCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    /// let pt = Point2::new(1.0, 2.0);
    ///
    /// // another hint
    /// let hint = 0.5;
    ///
    /// // Newton's method is vibration divergent.
    /// assert!(curve.search_nearest_parameter(pt, Some(hint), 100).is_none());
    /// ```
    #[inline(always)]
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        point: V::Point,
        hint: H,
        trial: usize,
    ) -> Option<f64> {
        let hint = match hint.into() {
            SPHint1D::Parameter(hint) => hint,
            SPHint1D::Range(x, y) => {
                algo::curve::presearch(self, point, (x, y), PRESEARCH_DIVISION)
            }
            SPHint1D::None => {
                algo::curve::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::curve::search_nearest_parameter(self, point, hint, trial)
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> SearchParameter<D1>
    for NurbsCurve<V>
where
    V::Point: MetricSpace<Metric = f64>,
    <V::Point as EuclideanSpace>::Diff: InnerSpace + Tolerance,
{
    type Point = V::Point;
    #[inline(always)]
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: V::Point,
        hint: H,
        trial: usize,
    ) -> Option<f64> {
        let hint = match hint.into() {
            SPHint1D::Parameter(hint) => hint,
            SPHint1D::Range(x, y) => {
                algo::curve::presearch(self, point, (x, y), PRESEARCH_DIVISION)
            }
            SPHint1D::None => {
                algo::curve::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::curve::search_parameter(self, point, hint, trial)
    }
}

impl<V: Homogeneous<Scalar = f64>> NurbsCurve<V>
where V::Point: Bounded<Scalar = f64>
{
    /// Returns the bounding box including all control points.
    #[inline(always)]
    pub fn roughly_bounding_box(&self) -> BoundingBox<V::Point> {
        self.0.control_points.iter().map(|p| p.to_point()).collect()
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> ParametricCurve for NurbsCurve<V> {
    type Point = V::Point;
    type Vector = <V::Point as EuclideanSpace>::Diff;
    fn der_n(&self, n: usize, t: f64) -> Self::Vector { self.0.ders(n, t).rat_ders()[n] }
    fn ders(&self, n: usize, t: f64) -> CurveDers<Self::Vector> { self.0.ders(n, t).rat_ders() }
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point { self.0.subs(t).to_point() }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector { rat_der(&[self.0.subs(t), self.0.der(t)]) }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector {
        rat_der(&[self.0.subs(t), self.0.der(t), self.0.der2(t)])
    }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange {
        (
            Bound::Included(self.0.knot_vec[0]),
            Bound::Included(self.0.knot_vec[self.0.knot_vec.len() - 1]),
        )
    }
}

impl<V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V>> BoundedCurve for NurbsCurve<V> {}

impl<V: Clone> Invertible for NurbsCurve<V> {
    #[inline(always)]
    fn invert(&mut self) { self.0.invert(); }
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut curve = self.0.clone();
        curve.invert();
        NurbsCurve(curve)
    }
}

impl<M, V: Copy> Transformed<M> for NurbsCurve<V>
where M: Copy + std::ops::Mul<V, Output = V>
{
    #[inline(always)]
    fn transform_by(&mut self, trans: M) {
        self.0
            .control_points
            .iter_mut()
            .for_each(move |v| *v = trans * *v)
    }
}

impl<V: Homogeneous<Scalar = f64>> From<BSplineCurve<V::Point>> for NurbsCurve<V> {
    #[inline(always)]
    fn from(bspcurve: BSplineCurve<V::Point>) -> NurbsCurve<V> {
        NurbsCurve::new(BSplineCurve::new_unchecked(
            bspcurve.knot_vec,
            bspcurve
                .control_points
                .into_iter()
                .map(V::from_point)
                .collect(),
        ))
    }
}
