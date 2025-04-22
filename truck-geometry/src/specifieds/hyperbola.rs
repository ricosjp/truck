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
    fn der_n(&self, t: f64, n: usize) -> Vector2 {
        match n % 2 {
            0 => Vector2::new(f64::cosh(t), f64::sinh(t)),
            _ => Vector2::new(f64::sinh(t), f64::cosh(t)),
        }
    }
    #[inline]
    fn subs(&self, t: f64) -> Self::Point { Point2::from_vec(self.der_n(t, 0)) }
    #[inline]
    fn der(&self, t: f64) -> Self::Vector { self.der_n(t, 1) }
    #[inline]
    fn der2(&self, t: f64) -> Self::Vector { self.der_n(t, 2) }
}

impl ParametricCurve for UnitHyperbola<Point3> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline]
    fn der_n(&self, t: f64, n: usize) -> Vector3 {
        match n % 2 {
            0 => Vector3::new(f64::cosh(t), f64::sinh(t), 0.0),
            _ => Vector3::new(f64::sinh(t), f64::cosh(t), 0.0),
        }
    }
    #[inline]
    fn subs(&self, t: f64) -> Self::Point { Point3::from_vec(self.der_n(t, 0)) }
    #[inline]
    fn der(&self, t: f64) -> Self::Vector { self.der_n(t, 1) }
    #[inline]
    fn der2(&self, t: f64) -> Self::Vector { self.der_n(t, 2) }
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
