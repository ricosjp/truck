use super::*;

impl<C, N> ParametricCurve for Offset<C, N>
where
    C: ParametricCurve,
    N: ParametricCurve<Point = C::Vector, Vector = C::Vector>,
    C::Point: ControlPoint<f64, Diff = C::Vector>,
    C::Vector: ControlPoint<f64, Diff = C::Vector>,
{
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point { self.entity.subs(t) + self.offset.subs(t) }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector { self.entity.der(t) + self.offset.der(t) }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector { self.entity.der2(t) + self.offset.der2(t) }
    #[inline(always)]
    fn der_n(&self, n: usize, t: f64) -> Self::Vector {
        self.entity.der_n(n, t) + self.offset.der_n(n, t)
    }
    #[inline(always)]
    fn ders(&self, n: usize, t: f64) -> CurveDers<Self::Vector> {
        self.entity
            .ders(n, t)
            .element_wise_ders(&self.offset.ders(n, t), |x, y| x + y)
    }
    /// Returns the range of `entity`
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { self.entity.parameter_range() }
    #[inline(always)]
    fn period(&self) -> Option<f64> {
        match (self.entity.period(), self.offset.period()) {
            (Some(x), Some(y)) if x.near(&y) => Some((x + y) / 2.0),
            _ => None,
        }
    }
}

impl<C, F> ParametricCurve for NormalField<C, F>
where
    C: ParametricCurve2D,
    F: ScalarFunctionD1,
{
    type Point = Vector2;
    type Vector = Vector2;
    #[inline(always)]
    fn ders(&self, n: usize, t: f64) -> CurveDers<Vector2> {
        let mut ders = self.entity.ders(n + 1, t).der();
        ders.iter_mut()
            .for_each(|vec| *vec = Vector2::new(-vec.y, vec.x));
        let scalar_der = self.scalar.ders(n, t);
        ders.combinatorial_ders(&scalar_der, |x, y| x * y)
    }
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point { self.ders(0, t)[0] }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector { self.ders(1, t)[1] }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector { self.ders(2, t)[2] }
    #[inline(always)]
    fn der_n(&self, n: usize, t: f64) -> Self::Vector { self.ders(n, t)[n] }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { self.entity.parameter_range() }
}

impl<C, N, P, V> BoundedCurve for Offset<C, N>
where
    C: BoundedCurve<Point = P, Vector = V>,
    N: ParametricCurve<Point = V, Vector = V>,
    P: ControlPoint<f64, Diff = V>,
    V: ControlPoint<f64, Diff = V>,
{
}

impl<C, N, P, V> ParameterDivision1D for Offset<C, N>
where
    C: BoundedCurve<Point = P, Vector = V>,
    N: ParametricCurve<Point = V, Vector = V>,
    P: ControlPoint<f64, Diff = V>
        + EuclideanSpace<Scalar = f64, Diff = V>
        + MetricSpace<Metric = f64>
        + HashGen<f64>,
    V: ControlPoint<f64, Diff = V>,
{
    type Point = P;
    #[inline]
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        algo::curve::parameter_division(self, range, tol)
    }
}

impl<C, N, P, V> SearchParameter<D1> for Offset<C, N>
where
    C: BoundedCurve<Point = P, Vector = V> + SearchNearestParameter<D1, Point = P>,
    N: ParametricCurve<Point = V, Vector = V>,
    P: ControlPoint<f64, Diff = V> + Copy + Tolerance,
    V: ControlPoint<f64, Diff = V>,
{
    type Point = P;
    #[inline]
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let t = self.entity.search_nearest_parameter(point, hint, trials)?;
        match self.subs(t).near(&point) {
            true => Some(t),
            false => None,
        }
    }
}

impl<C, N, P, V> SearchNearestParameter<D1> for Offset<C, N>
where
    C: BoundedCurve<Point = P, Vector = V>,
    N: ParametricCurve<Point = V, Vector = V>,
    P: ControlPoint<f64, Diff = V>
        + EuclideanSpace<Scalar = f64, Diff = V>
        + MetricSpace<Metric = f64>
        + Copy,
    V: ControlPoint<f64, Diff = V> + InnerSpace<Scalar = f64> + Tolerance,
{
    type Point = P;
    #[inline]
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let hint = match hint.into() {
            SPHint1D::Parameter(t) => t,
            SPHint1D::Range(x, y) => {
                algo::curve::presearch(self, point, (x, y), PRESEARCH_DIVISION)
            }
            SPHint1D::None => {
                algo::curve::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::curve::search_nearest_parameter(self, point, hint, trials)
    }
}
