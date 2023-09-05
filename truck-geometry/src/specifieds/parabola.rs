use super::*;

impl<P> UnitParabola<P> {
    /// constructor
    #[inline]
    pub const fn new() -> Self { Self(std::marker::PhantomData) }
}

impl ParametricCurve for UnitParabola<Point2> {
    type Point = Point2;
    type Vector = Vector2;
    #[inline]
    fn subs(&self, t: f64) -> Self::Point { Point2::new(t * t, 2.0 * t) }
    #[inline]
    fn der(&self, t: f64) -> Self::Vector { Vector2::new(2.0 * t, 2.0) }
    #[inline]
    fn der2(&self, _: f64) -> Self::Vector { Vector2::new(2.0, 0.0) }
    #[inline]
    fn parameter_range(&self) -> (Bound<f64>, Bound<f64>) {
        (Bound::Unbounded, Bound::Unbounded)
    }
}

impl ParametricCurve for UnitParabola<Point3> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline]
    fn subs(&self, t: f64) -> Self::Point { Point3::new(t * t, 2.0 * t, 0.0) }
    #[inline]
    fn der(&self, t: f64) -> Self::Vector { Vector3::new(2.0 * t, 2.0, 0.0) }
    #[inline]
    fn der2(&self, _: f64) -> Self::Vector { Vector3::new(2.0, 0.0, 0.0) }
    #[inline]
    fn parameter_range(&self) -> (Bound<f64>, Bound<f64>) {
        (Bound::Unbounded, Bound::Unbounded)
    }
}

impl<P> ParameterDivision1D for UnitParabola<P>
where
    UnitParabola<P>: ParametricCurve<Point = P>,
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
{
    type Point = P;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<P>) {
        algo::curve::parameter_division(self, range, tol)
    }
}

impl SearchNearestParameter<D1> for UnitParabola<Point2> {
    type Point = Point2;
    #[inline]
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        pt: Point2,
        _: H,
        _: usize,
    ) -> Option<f64> {
        let p = 2.0 - pt.x;
        let q = -pt.y;
        solver::pre_solve_cubic(p, q)
            .into_iter()
            .filter_map(|x| match x.im.so_small() {
                true => Some(x.re),
                false => None,
            })
            .min_by(|s, t| {
                pt.distance2(self.subs(*s))
                    .partial_cmp(&pt.distance2(self.subs(*t)))
                    .unwrap()
            })
    }
}

impl SearchNearestParameter<D1> for UnitParabola<Point3> {
    type Point = Point3;
    #[inline]
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        pt: Point3,
        _hint: H,
        _trials: usize,
    ) -> Option<f64> {
        UnitParabola::<Point2>::new().search_nearest_parameter(
            Point2::new(pt.x, pt.y),
            _hint,
            _trials,
        )
    }
}

impl SearchParameter<D1> for UnitParabola<Point2> {
    type Point = Point2;
    #[inline]
    fn search_parameter<H: Into<SPHint1D>>(&self, pt: Point2, _: H, _: usize) -> Option<f64> {
        let t = pt.y / 2.0;
        let pt0 = self.subs(t);
        match pt.near(&pt0) {
            true => Some(t),
            false => None,
        }
    }
}

impl SearchParameter<D1> for UnitParabola<Point3> {
    type Point = Point3;
    #[inline]
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        pt: Point3,
        _hint: H,
        _trials: usize,
    ) -> Option<f64> {
        match pt.z.so_small() {
            true => UnitParabola::<Point2>::new().search_parameter(
                Point2::new(pt.x, pt.y),
                _hint,
                _trials,
            ),
            false => None,
        }
    }
}

#[test]
fn snp_test() {
    let curve = UnitParabola::<Point2>::new();

    let p = Point2::new(-3.0, 0.0);
    assert_near!(curve.search_nearest_parameter(p, None, 0).unwrap(), 0.0);
    let p = Point2::new(-3.0, 6.0);
    assert_near!(curve.search_nearest_parameter(p, None, 0).unwrap(), 1.0);
    let p = Point2::new(1.5, 1.5);
    assert_near!(curve.search_nearest_parameter(p, None, 0).unwrap(), 1.0);
}

#[test]
fn sp_test() {
    let curve = UnitParabola::<Point2>::new();

    let p = Point2::new(4.0, -4.0);
    assert_near!(curve.search_parameter(p, None, 0).unwrap(), -2.0);
    let p = Point2::new(-3.0, 6.0);
    assert!(curve.search_parameter(p, None, 0).is_none());
}
