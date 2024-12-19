use super::*;
use truck_base::cgmath64::control_point::ControlPoint;

impl<P: Copy> Line<P> {
    /// initialize line from vector
    #[inline]
    pub fn from_origin_direction<V>(origin: P, direction: V) -> Self
    where P: std::ops::Add<V, Output = P> {
        Self(origin, origin + direction)
    }
}

impl<P> Line<P>
where
    P: EuclideanSpace<Scalar = f64>,
    P::Diff: InnerSpace<Scalar = f64>,
{
    /// Returns the projected point to the line.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let line = Line(Point2::new(0.0, 0.0), Point2::new(1.0, 2.0));
    /// let pt = Point2::new(0.0, 1.0);
    /// assert_near!(line.projection(pt), Point2::new(0.4, 0.8));
    /// ```
    pub fn projection(self, point: P) -> P {
        let (u, v) = (point - self.0, self.1 - self.0);
        self.0 + v * u.dot(v) / v.dot(v)
    }
    /// Returns the distance between the line and the point `point`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let line = Line(Point2::new(0.0, 0.0), Point2::new(3.0, 4.0));
    ///
    /// // The foot of the perpendicular line is on a line segment
    /// let pt = Point2::new(3.0, 0.0);
    /// assert_near!(line.distance_to_point(pt), 2.4);
    ///
    /// // The foot of the perpendicular line is not on a line segment
    /// let pt = Point2::new(0.0, -4.0);
    /// assert_near!(line.distance_to_point(pt), 2.4);
    /// ```
    pub fn distance_to_point(self, point: P) -> f64 {
        let (u, v) = (point - self.0, self.1 - self.0);
        (u - v * u.dot(v) / v.dot(v)).magnitude()
    }
    /// Returns the distance between the sengment and the point `point`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let line = Line(Point2::new(0.0, 0.0), Point2::new(3.0, 4.0));
    ///
    /// // The foot of the perpendicular line is on a line segment
    /// let pt = Point2::new(3.0, 0.0);
    /// assert_near!(line.distance_to_point_as_segment(pt), 2.4);
    ///
    /// // The foot of the perpendicular line is not on a line segment
    /// let pt = Point2::new(0.0, -4.0);
    /// assert_near!(line.distance_to_point_as_segment(pt), 4.0);
    /// ```
    pub fn distance_to_point_as_segment(self, point: P) -> f64 {
        let (u, v) = (point - self.0, self.1 - self.0);
        let t = f64::clamp(u.dot(v) / v.dot(v), 0.0, 1.0);
        (u - v * t).magnitude()
    }
}

impl Line<Point2> {
    /// Returns the intersection of two lines and its parameters.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let line0 = Line(Point2::new(0.0, 0.0), Point2::new(9.0, 3.0));
    /// let line1 = Line(Point2::new(0.0, 6.0), Point2::new(9.0, 0.0));
    /// let (s, t, p) = line0.intersection(line1).unwrap();
    /// assert_near!(line0.subs(s), Point2::new(6.0, 2.0));
    /// assert_near!(line1.subs(t), Point2::new(6.0, 2.0));
    /// assert_near!(p, Point2::new(6.0, 2.0));
    /// ```
    /// # Failures
    /// Returns `None` if two lines are parallel.
    /// ```
    /// use truck_geometry::prelude::*;
    /// let line0 = Line(Point2::new(0.0, 0.0), Point2::new(9.0, 3.0));
    /// let line1 = Line(Point2::new(-1.0, 0.0), Point2::new(8.0, 3.0));
    /// assert!(line0.intersection(line1).is_none());
    /// ```
    pub fn intersection(self, other: Line<Point2>) -> Option<(f64, f64, Point2)> {
        let mat = Matrix2::from_cols(self.1 - self.0, other.0 - other.1);
        let v = other.0 - self.0;
        let params = mat.invert().map(|inv| inv * v)?;
        Some((params.x, params.y, self.subs(params.x)))
    }
}

impl<P: ControlPoint<f64>> ParametricCurve for Line<P> {
    type Point = P;
    type Vector = P::Diff;
    #[inline]
    fn subs(&self, t: f64) -> Self::Point { self.0 + (self.1 - self.0) * t }
    #[inline]
    fn der(&self, _: f64) -> Self::Vector { self.1 - self.0 }
    #[inline]
    fn der2(&self, _: f64) -> Self::Vector { Self::Vector::zero() }
    /// Return `0.0..=1.0` i.e. we regard it as a segment
    #[inline]
    fn parameter_range(&self) -> ParameterRange { (Bound::Included(0.0), Bound::Included(1.0)) }
}

impl<P: ControlPoint<f64>> BoundedCurve for Line<P> {}

impl<P: ControlPoint<f64>> Cut for Line<P> {
    #[inline]
    fn cut(&mut self, t: f64) -> Self {
        let r = self.subs(t);
        let res = Self(r, self.1);
        self.1 = r;
        res
    }
}

impl<P: ControlPoint<f64>> ParameterDivision1D for Line<P> {
    type Point = P;
    #[inline]
    fn parameter_division(&self, range: (f64, f64), _: f64) -> (Vec<f64>, Vec<P>) {
        (
            vec![range.0, range.1],
            vec![self.subs(range.0), self.subs(range.1)],
        )
    }
}

impl<P: Copy> Invertible for Line<P> {
    #[inline]
    fn invert(&mut self) { *self = Self(self.1, self.0); }
    #[inline]
    fn inverse(&self) -> Self { Self(self.1, self.0) }
}

impl<P> SearchNearestParameter<D1> for Line<P>
where
    P: ControlPoint<f64>,
    P::Diff: InnerSpace<Scalar = f64>,
{
    type Point = P;
    #[inline]
    fn search_nearest_parameter<H: Into<SPHint1D>>(&self, pt: P, _: H, _: usize) -> Option<f64> {
        let b = self.1 - self.0;
        Some((pt - self.0).dot(b) / b.dot(b))
    }
}

impl<P> SearchParameter<D1> for Line<P>
where
    P: ControlPoint<f64> + Tolerance,
    P::Diff: InnerSpace<Scalar = f64>,
{
    type Point = P;
    #[inline]
    fn search_parameter<H: Into<SPHint1D>>(&self, pt: P, _: H, _: usize) -> Option<f64> {
        let b = self.1 - self.0;
        let t = (pt - self.0).dot(b) / b.dot(b);
        match self.subs(t).near(&pt) {
            true => Some(t),
            false => None,
        }
    }
}

impl<P: EuclideanSpace, M: Transform<P>> Transformed<M> for Line<P> {
    #[inline]
    fn transform_by(&mut self, trans: M) {
        self.0 = trans.transform_point(self.0);
        self.1 = trans.transform_point(self.1);
    }
    #[inline]
    fn transformed(&self, trans: M) -> Self {
        Line(trans.transform_point(self.0), trans.transform_point(self.1))
    }
}

impl<P> From<Line<P>> for BSplineCurve<P> {
    fn from(Line(p, q): Line<P>) -> Self {
        BSplineCurve::new_unchecked(KnotVec::bezier_knot(1), vec![p, q])
    }
}

impl<P: Copy> ToSameGeometry<BSplineCurve<P>> for Line<P> {
    fn to_same_geometry(&self) -> BSplineCurve<P> { BSplineCurve::from(*self) }
}

#[test]
fn line() {
    let line = Line(Point2::new(1.0, 0.0), Point2::new(0.0, 1.0));

    // subs
    assert_near!(line.subs(0.4), Point2::new(0.6, 0.4));

    // inverse
    let line_inverse = line.inverse();
    assert_eq!(line.0, line_inverse.1);
    assert_eq!(line.1, line_inverse.0);

    // cut
    let mut line0 = line;
    let line1 = line0.cut(0.4);
    assert_eq!(line.0, line0.0);
    assert_near!(line0.1, line.subs(0.4));
    assert_eq!(line0.1, line1.0);
    assert_eq!(line1.1, line.1);

    // SNP
    assert_near!(
        line.search_nearest_parameter(Point2::new(1.0, 1.0), None, 0)
            .unwrap(),
        0.5
    );
    assert!(line
        .search_parameter(Point2::new(1.0, 1.0), None, 0)
        .is_none());
}
