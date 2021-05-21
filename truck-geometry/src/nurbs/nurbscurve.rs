use super::*;

impl<V> NURBSCurve<V> {
    /// Constructs the rationalized B-spline curve.
    #[inline(always)]
    pub const fn new(curve: BSplineCurve<V>) -> Self { NURBSCurve(curve) }

    /// Returns the BSpline curve before rationalized.
    #[inline(always)]
    pub fn non_rationalized(&self) -> &BSplineCurve<V> { &self.0 }

    /// Returns the BSpline curve before rationalized.
    #[inline(always)]
    pub fn into_non_rationalized(self) -> BSplineCurve<V> { self.0 }

    /// Returns the reference of the knot vector.  
    /// cf.[`BSplineCurve::knot_vec`](./struct.BSplineCurve.html#method.knot_vec)
    #[inline(always)]
    pub fn knot_vec(&self) -> &KnotVec { &self.0.knot_vec }

    /// Returns the `idx`th knot.  
    /// cf.[`BSplineCurve::knot`](./struct.BSplineCurve.html#method.knot)
    #[inline(always)]
    pub fn knot(&self, idx: usize) -> f64 { self.0.knot_vec[idx] }

    /// Returns the reference of the control points.  
    /// cf.[`BSplineCurve::control_points`](./struct.BSplineCurve.html#method.control_points)
    #[inline(always)]
    pub fn control_points(&self) -> &Vec<V> { &self.0.control_points }

    /// Returns the reference of the control point corresponding to the index `idx`.  
    /// cf.[`BSplineCurve::control_point`](./struct.BSplineCurve.html#method.control_point)
    #[inline(always)]
    pub fn control_point(&self, idx: usize) -> &V { &self.0.control_points[idx] }

    /// Returns the mutable reference of the control point corresponding to index `idx`.  
    /// cf.[`BSplineCurve::control_point_mut`](./struct.BSplineCurve.html#method.control_point_mut)
    #[inline(always)]
    pub fn control_point_mut(&mut self, idx: usize) -> &mut V { &mut self.0.control_points[idx] }
    /// Returns the iterator on all control points  
    /// cf.[`BSplineCurve::control_points_mut`](./struct.BSplineCurve.html#method.control_points_mut)
    #[inline(always)]
    pub fn control_points_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.0.control_points.iter_mut()
    }

    /// Apply the given transformation to all control points.  
    /// cf.[`BSplineCurve::transform_control_points`](./struct.BSplineCurve.html#method.transform_control_points)
    #[inline(always)]
    pub fn transform_control_points<F: FnMut(&mut V)>(&mut self, f: F) {
        self.0.transform_control_points(f)
    }

    /// Returns the degree of NURBS curve.  
    /// cf.[`BSplineCurve::degree`](./struct.BSplineCurve.html#method.degree)
    #[inline(always)]
    pub fn degree(&self) -> usize { self.0.degree() }

    /// Inverts a curve.  
    /// cf.[`BSplineCurve::invert`](./struct.BSplineCurve.html#method.invert)
    #[inline(always)]
    pub fn invert(&mut self) -> &mut Self {
        self.0.invert();
        self
    }

    /// Returns whether the knot vector is clamped or not.  
    /// cf.[`BSplineCurve::is_clamped`](./struct.BSplineCurve.html#method.is_clamped)
    #[inline(always)]
    pub fn is_clamped(&self) -> bool { self.0.knot_vec.is_clamped(self.0.degree()) }

    /// Normalizes the knot vector.  
    /// cf.[`BSplineCurve::knot_normalize`](./struct.BSplineCurve.html#method.knot_normalize)
    #[inline(always)]
    pub fn knot_normalize(&mut self) -> &mut Self {
        self.0.knot_vec.try_normalize().unwrap();
        self
    }

    /// Translates the knot vector.  
    /// cf.[`BSplineCurve::knot_translate`](./struct.BSplineCurve.html#method.knot_translate)
    #[inline(always)]
    pub fn knot_translate(&mut self, x: f64) -> &mut Self {
        self.0.knot_vec.translate(x);
        self
    }
}

impl<V: Homogeneous<f64>> NURBSCurve<V> {
    /// Returns the closure of substitution.
    ///
    #[inline(always)]
    pub fn get_closure(&self) -> impl Fn(f64) -> V::Point + '_ { move |t| self.subs(t) }
}

impl<V: Homogeneous<f64>> NURBSCurve<V>
where V::Point: Tolerance
{
    /// Returns whether all control points are the same or not.
    /// If the knot vector is clamped, it means whether the curve is constant or not.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let pt = Vector3::new(1.0, 2.0, 1.0);
    /// // allows differences upto scalars
    /// let mut ctrl_pts = vec![pt.clone(), pt.clone() * 2.0, pt.clone() * 3.0];
    /// let bspcurve = BSplineCurve::new(knot_vec.clone(), ctrl_pts.clone());
    /// assert!(!bspcurve.is_const());
    /// let const_curve = NURBSCurve::new(bspcurve);
    /// assert!(const_curve.is_const());
    ///
    /// ctrl_pts.push(Vector3::new(2.0, 3.0, 1.0));
    /// let curve = NURBSCurve::new(BSplineCurve::new(knot_vec.clone(), ctrl_pts.clone()));
    /// assert!(!curve.is_const());
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
        let pt = self.0.control_points[0].to_point();
        self.0
            .control_points
            .iter()
            .all(move |vec| vec.to_point().near(&pt))
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
    ///     Vector3::new(1.0, 1.0, 1.0),
    ///     Vector3::new(3.0, 2.0, 2.0),
    ///     Vector3::new(2.0, 3.0, 1.0),
    ///     Vector3::new(4.0, 5.0, 2.0),
    ///     Vector3::new(5.0, 4.0, 1.0),
    ///     Vector3::new(1.0, 1.0, 2.0),
    /// ];
    /// let curve0 = NURBSCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
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
    /// use truck_geometry::*;
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
    /// let curve0 = NURBSCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
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

impl<V: Homogeneous<f64> + Tolerance> NURBSCurve<V> {
    /// Adds a knot `x`, and do not change `self` as a curve.  
    /// cf.[`BSplineCurve::add_knot`](./struct.BSplineCurve.html#method.add_knot)
    pub fn add_knot(&mut self, x: f64) -> &mut Self {
        self.0.add_knot(x);
        self
    }

    /// Removes a knot corresponding to the indices `idx`, and do not change `self` as a curve.
    /// If cannot remove the knot, do not change `self` and return `self`.  
    /// cf.[`BSplineCurve::remove_knot`](./struct.BSplineCurve.html#method.remove_knot)
    pub fn remove_knot(&mut self, idx: usize) -> &mut Self {
        let _ = self.try_remove_knot(idx);
        self
    }

    /// Removes a knot corresponding to the indice `idx`, and do not change `self` as a curve.  
    /// If the knot cannot be removed, returns
    /// [`Error::CannotRemoveKnot`](./errors/enum.Error.html#variant.CannotRemoveKnot).  
    /// cf.[`BSplineCurve::try_remove_knot`](./struct.BSplineCurve.html#method.try_remove_knot)
    pub fn try_remove_knot(&mut self, idx: usize) -> Result<&mut Self> {
        self.0.try_remove_knot(idx)?;
        Ok(self)
    }

    /// Elevates 1 degree.  
    /// cf.[`BSplineCurve::elevate_degree`](./struct.BSplineCurve.html#method.elevate_degree)
    pub fn elevate_degree(&mut self) -> &mut Self {
        self.0.elevate_degree();
        self
    }

    /// Makes the NURBS curve clamped  
    /// cf.[`BSplineCurve::clamp`](./struct.BSplineCurve.html#method.clamp)
    #[inline(always)]
    pub fn clamp(&mut self) -> &mut Self {
        self.0.clamp();
        self
    }

    /// Repeats `Self::try_remove_knot()` from the back knot in turn until the knot cannot be removed.  
    /// cf.[`BSplineCurve::optimize`](./struct.BSplineCurve.html#method.optimize)
    pub fn optimize(&mut self) -> &mut Self {
        self.0.optimize();
        self
    }

    /// Makes two splines having the same degrees.  
    /// cf.[`BSplineCurve::syncro_degree`](./struct.BSplineCurve.html#method.syncro_degree)
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
    /// cf.[`BSplineCurve::syncro_knots`](./struct.BSplineCurve.html#method.syncro_knots)
    pub fn syncro_knots(&mut self, other: &mut Self) { self.0.syncro_knots(&mut other.0) }

    /// Cuts the curve to two curves at the parameter `t`.  
    /// cf.[`BSplineCurve::syncro_knots`](./struct.BSplineCurve.html#method.syncro_knots)
    pub fn cut(&mut self, t: f64) -> Self { NURBSCurve(self.0.cut(t)) }

    /// Concats two NURBS curves.  
    /// cf.[`BSplineCurve::try_concat`](./struct.BSplineCurve.html#method.try_concat)
    pub fn try_concat(&mut self, other: &mut Self) -> Result<&mut Self> {
        let w0 = self.0.control_points.last().unwrap().weight();
        let w1 = other.0.control_points[0].weight();
        other.transform_control_points(|pt| *pt = *pt * (w0 / w1));
        self.0.try_concat(&mut other.0)?;
        Ok(self)
    }
    /// Concats two NURBS curves.  
    /// cf.[`BSplineCurve::concat`](./struct.BSplineCurve.html#method.concat)
    #[inline(always)]
    pub fn concat(&mut self, other: &mut Self) -> &mut Self {
        self.try_concat(other)
            .unwrap_or_else(|error| panic!("{}", error))
    }
}

impl<V: Homogeneous<f64> + Tolerance> NURBSCurve<V>
where V::Point: Tolerance
{
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
    /// use truck_geometry::*;
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0]);
    /// let ctrl_pts = vec![
    ///     Vector3::new(1.0, 1.0, 1.0),
    ///     Vector3::new(2.0, 2.0, 2.0),
    ///     Vector3::new(3.0, 3.0, 3.0),
    ///     Vector3::new(4.0, 4.0, 4.0),
    /// ];
    /// let mut curve = NURBSCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    /// let org_curve = curve.clone();
    /// curve.make_locally_injective();
    /// assert_eq!(curve, org_curve);
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
}

impl<V: Homogeneous<f64>> ParameterDivision1D for NURBSCurve<V>
where V::Point: MetricSpace<Metric = f64>
{
    #[inline(always)]
    fn parameter_division(&self, tol: f64) -> Vec<f64> {
        algo::curve::parameter_division(self, self.parameter_range(), tol)
    }
}

impl<V: Homogeneous<f64>> NURBSCurve<V>
where <V::Point as EuclideanSpace>::Diff: InnerSpace + Tolerance,
{
    /// Searches the parameter `t` which minimize |self(t) - point| by Newton's method with initial guess `hint`.
    /// Returns `None` if the number of attempts exceeds `trial` i.e. if `trial == 0`, then the trial is only one time.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    ///
    /// // Defines the half unit circle in x > 0.
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector3::new(0.0, -1.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 1.0)];
    /// let curve = NURBSCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    ///
    /// // search rational nearest parameter
    /// let pt = Point2::new(1.0, 2.0);
    /// let hint = 0.8;
    /// let t = curve.search_nearest_parameter(pt, hint, 100).unwrap();
    ///
    /// // check the answer
    /// let res = curve.subs(t);
    /// let ans = Point2::from_vec(pt.to_vec().normalize());
    /// assert_near!(res, ans);
    /// ```
    /// # Remarks
    /// It may converge to a local solution depending on the hint.
    /// ```
    /// use truck_geometry::*;
    ///
    /// // Same curve and point as above example
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector3::new(0.0, -1.0, 1.0), Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 1.0)];
    /// let curve = NURBSCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    /// let pt = Point2::new(1.0, 2.0);
    ///
    /// // another hint
    /// let hint = 0.5;
    ///
    /// // Newton's method is vibration divergent.
    /// assert!(curve.search_nearest_parameter(pt, hint, 100).is_none());
    /// ```
    #[inline(always)]
    pub fn search_nearest_parameter(&self, point: V::Point, hint: f64, trial: usize) -> Option<f64> {
        algo::curve::search_nearest_parameter(self, point, hint, trial)
    }
}

impl<V: Homogeneous<f64>> SearchParameter for NURBSCurve<V>
where <V::Point as EuclideanSpace>::Diff: InnerSpace + Tolerance,
{
    type Point = V::Point;
    type Parameter = f64;
    #[inline(always)]
    fn search_parameter(&self, point: V::Point, hint: f64, trial: usize) -> Option<f64> {
        self.search_nearest_parameter(point, hint, trial)
            .and_then(|t| match point.to_vec().near(&self.subs(t).to_vec()) {
                true => Some(t),
                false => None,
            })
    }
}

impl<V: Homogeneous<f64>> NURBSCurve<V>
where V::Point: MetricSpace<Metric = f64> + std::ops::Index<usize, Output = f64> + Bounded<f64> + Copy
{
    /// Returns the bounding box including all control points.
    #[inline(always)]
    pub fn roughly_bounding_box(&self) -> BoundingBox<V::Point> { self.0.control_points.iter().map(|p| p.to_point()).collect() }
}

impl<V: Homogeneous<f64>> ParametricCurve for NURBSCurve<V> {
    type Point = V::Point;
    type Vector = <V::Point as EuclideanSpace>::Diff;
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point { self.0.subs(t).to_point() }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector {
        let pt = self.0.subs(t);
        let der = self.0.der(t);
        pt.rat_der(der)
    }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector {
        let pt = self.0.subs(t);
        let der = self.0.der(t);
        let der2 = self.0.der2(t);
        pt.rat_der2(der, der2)
    }
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) {
        (
            self.0.knot_vec[0],
            self.0.knot_vec[self.0.knot_vec.len() - 1],
        )
    }
}

impl<'a, V: Homogeneous<f64>> ParametricCurve for &'a NURBSCurve<V> {
    type Point = V::Point;
    type Vector = <V::Point as EuclideanSpace>::Diff;
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point { (*self).subs(t) }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector { (*self).der(t) }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector { (*self).der2(t) }
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) { (*self).parameter_range() }
}

impl<V: Clone> Invertible for NURBSCurve<V> {
    #[inline(always)]
    fn invert(&mut self) { self.invert(); }
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut curve = self.0.clone();
        curve.invert();
        NURBSCurve(curve)
    }
}

impl Transformed<Matrix2> for NURBSCurve<Vector3> {
    #[inline(always)]
    fn transform_by(&mut self, trans: Matrix2) { self.transform_by(Matrix3::from(trans)) }
    #[inline(always)]
    fn transformed(&self, trans: Matrix2) -> Self { self.transformed(Matrix3::from(trans)) }
}

impl Transformed<Matrix3> for NURBSCurve<Vector3> {
    #[inline(always)]
    fn transform_by(&mut self, trans: Matrix3) { self.0.transform_by(trans) }
    #[inline(always)]
    fn transformed(&self, trans: Matrix3) -> Self {
        let mut curve = self.clone();
        curve.transform_by(trans);
        curve
    }
}

impl Transformed<Matrix3> for NURBSCurve<Vector4> {
    #[inline(always)]
    fn transform_by(&mut self, trans: Matrix3) { self.transform_by(Matrix3::from(trans)) }
    #[inline(always)]
    fn transformed(&self, trans: Matrix3) -> Self { self.transformed(Matrix3::from(trans)) }
}

impl Transformed<Matrix4> for NURBSCurve<Vector4> {
    #[inline(always)]
    fn transform_by(&mut self, trans: Matrix4) {
        self.0
            .control_points
            .iter_mut()
            .for_each(|pt| *pt = trans * *pt)
    }
    #[inline(always)]
    fn transformed(&self, trans: Matrix4) -> Self {
        let mut curve = self.clone();
        curve.transform_by(trans);
        curve
    }
}

#[test]
fn test_parameter_division() {
    let knot_vec = KnotVec::uniform_knot(2, 3);
    let ctrl_pts = vec![
        Vector4::new(0.0, 0.0, 0.0, 1.0),
        Vector4::new(2.0, 0.0, 0.0, 2.0),
        Vector4::new(0.0, 3.0, 0.0, 3.0),
        Vector4::new(0.0, 0.0, 2.0, 2.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
    ];
    let curve = NURBSCurve::new(BSplineCurve::new(knot_vec, ctrl_pts));
    let tol = 0.01;
    let div = curve.parameter_division(tol * 0.5);
    let knot_vec = curve.knot_vec();
    assert_eq!(knot_vec[0], div[0]);
    assert_eq!(knot_vec.range_length(), div.last().unwrap() - div[0]);
    for i in 1..div.len() {
        let pt0 = curve.subs(div[i - 1]);
        let pt1 = curve.subs(div[i]);
        let value_middle = pt0.midpoint(pt1);
        let param_middle = curve.subs((div[i - 1] + div[i]) / 2.0);
        println!("{}", value_middle.distance(param_middle));
        assert!(value_middle.distance(param_middle) < tol);
    }
}
