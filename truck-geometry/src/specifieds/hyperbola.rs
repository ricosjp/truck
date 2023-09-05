use super::*;

impl<P> UnitHyperbola<P> {
    /// constructor
    #[inline]
    pub const fn new() -> UnitHyperbola<P> { UnitHyperbola(std::marker::PhantomData) }
}

impl ParametricCurve for UnitHyperbola<Point2> {
    type Point = Point2;
    type Vector = Vector2;
    #[inline]
    fn subs(&self, t: f64) -> Self::Point { Point2::new(f64::cosh(t), f64::sinh(t)) }
    #[inline]
    fn der(&self, t: f64) -> Self::Vector { Vector2::new(f64::sinh(t), f64::cosh(t)) }
    #[inline]
    fn der2(&self, t: f64) -> Self::Vector { Vector2::new(f64::cosh(t), f64::sinh(t)) }
    #[inline]
    fn parameter_range(&self) -> (Bound<f64>, Bound<f64>) {
        (Bound::Unbounded, Bound::Unbounded)
    }
}

impl ParametricCurve for UnitHyperbola<Point3> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline]
    fn subs(&self, t: f64) -> Self::Point { Point3::new(f64::cosh(t), f64::sinh(t), 0.0) }
    #[inline]
    fn der(&self, t: f64) -> Self::Vector { Vector3::new(f64::sinh(t), f64::cosh(t), 0.0) }
    #[inline]
    fn der2(&self, t: f64) -> Self::Vector { Vector3::new(f64::cosh(t), f64::sinh(t), 0.0) }
    #[inline]
    fn parameter_range(&self) -> (Bound<f64>, Bound<f64>) {
        (Bound::Unbounded, Bound::Unbounded)
    }
}

impl<P> ParameterDivision1D for UnitHyperbola<P>
where
    UnitHyperbola<P>: ParametricCurve<Point = P>,
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
{
    type Point = P;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<P>) {
        algo::curve::parameter_division(self, range, tol)
    }
}

impl SearchNearestParameter<D1> for UnitHyperbola<Point2> {
    type Point = Point2;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        p: Point2,
        _: H,
        _: usize,
    ) -> Option<f64> {
        let a = -p.y;
        let b = (p.y * p.y - p.x * p.x) / 4.0 + 1.0;
        let c = -p.y;
        let d = p.y * p.y / 4.0;
        let y = solver::solve_quartic(a, b, c, d)
            .into_iter()
            .filter_map(|z| match z.im.so_small() {
                true => Some(z.re),
                false => None,
            })
            .min_by(|s, t| {
                p.distance2(self.subs(*s))
                    .partial_cmp(&p.distance2(self.subs(*t)))
                    .unwrap()
            })?;
        Some(f64::asinh(y))
    }
}

impl SearchNearestParameter<D1> for UnitHyperbola<Point3> {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        p: Point3,
        _: H,
        _trials: usize,
    ) -> Option<f64> {
        UnitHyperbola::<Point2>::new().search_nearest_parameter(
            Point2::new(p.x, p.y),
            None,
            _trials,
        )
    }
}

impl SearchParameter<D1> for UnitHyperbola<Point2> {
    type Point = Point2;
    fn search_parameter<H: Into<SPHint1D>>(&self, p: Point2, _: H, _: usize) -> Option<f64> {
        let t = f64::asinh(p.y);
        match p.near(&self.subs(t)) {
            true => Some(t),
            false => None,
        }
    }
}

impl SearchParameter<D1> for UnitHyperbola<Point3> {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint1D>>(&self, p: Point3, _: H, _: usize) -> Option<f64> {
        let t = f64::asinh(p.y);
        match p.near(&self.subs(t)) {
            true => Some(t),
            false => None,
        }
    }
}

#[test]
fn snp_test() {
    let curve = UnitHyperbola::<Point2>::new();
    let p = curve.subs(2.0);
    let q = p + Vector2::new(-p.x, p.y);
    let t = curve.search_nearest_parameter(q, None, 0).unwrap();
    assert_near!(t, 2.0);
}

#[test]
fn sp_test() {
    let curve = UnitHyperbola::<Point2>::new();
    let t = 100.0 * rand::random::<f64>() - 50.0;
    let p = curve.subs(t);
    assert_near!(curve.search_parameter(p, None, 0).unwrap(), t);

    let q = Point2::new(-1.0, 0.0);
    assert!(curve.search_parameter(q, None, 0).is_none());
}
