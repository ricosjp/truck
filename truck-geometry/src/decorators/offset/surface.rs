use super::*;

impl<C, N> ParametricSurface for Offset<C, N>
where
    C: ParametricSurface,
    N: ParametricSurface<Point = C::Vector, Vector = C::Vector>,
    C::Point: ControlPoint<f64, Diff = C::Vector>,
    C::Vector: ControlPoint<f64, Diff = C::Vector>,
{
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point {
        self.entity.subs(u, v) + self.offset.subs(u, v)
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector {
        self.entity.uder(u, v) + self.offset.uder(u, v)
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Self::Vector {
        self.entity.vder(u, v) + self.offset.vder(u, v)
    }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Self::Vector {
        self.entity.uuder(u, v) + self.offset.uuder(u, v)
    }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Self::Vector {
        self.entity.uvder(u, v) + self.offset.uvder(u, v)
    }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Self::Vector {
        self.entity.vvder(u, v) + self.offset.vvder(u, v)
    }
    #[inline(always)]
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        self.entity.der_mn(m, n, u, v) + self.offset.der_mn(m, n, u, v)
    }
    #[inline(always)]
    fn ders(&self, n: usize, u: f64, v: f64) -> SurfaceDers<Self::Vector> {
        self.entity
            .ders(n, u, v)
            .element_wise_ders(&self.offset.ders(n, u, v), |x, y| x + y)
    }
    /// Returns the range of `entity`
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) { self.entity.parameter_range() }
    #[inline(always)]
    fn u_period(&self) -> Option<f64> {
        match (self.entity.u_period(), self.offset.u_period()) {
            (Some(x), Some(y)) if x.near(&y) => Some((x + y) / 2.0),
            _ => None,
        }
    }
    #[inline(always)]
    fn v_period(&self) -> Option<f64> {
        match (self.entity.v_period(), self.offset.v_period()) {
            (Some(x), Some(y)) if x.near(&y) => Some((x + y) / 2.0),
            _ => None,
        }
    }
}

impl<S, F> ParametricSurface for NormalField<S, F>
where
    S: ParametricSurface3D,
    F: ScalarFunctionD2,
{
    type Point = Vector3;
    type Vector = Vector3;
    fn ders(&self, max_order: usize, u: f64, v: f64) -> SurfaceDers<Self::Vector> {
        let surface_ders = self.entity.ders(max_order + 1, u, v);
        let uders = surface_ders.uder();
        let vders = surface_ders.vder();

        let normal_ders = uders.combinatorial_ders(&vders, Vector3::cross);
        let normalized_ders = normal_ders
            .element_wise_ders(&normal_ders.abs_ders(), Vector3::extend)
            .rat_ders();

        normalized_ders.combinatorial_ders(&self.scalar.ders(max_order, u, v), |x, y| x * y)
    }
    #[inline(always)]
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        self.ders(m + n, u, v)[m][n]
    }
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point {
        self.entity.normal(u, v) * self.scalar.subs(u, v)
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector {
        self.entity.normal_uder(u, v) * self.scalar.subs(u, v)
            + self.entity.normal(u, v) * self.scalar.uder(u, v)
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Self::Vector {
        self.entity.normal_vder(u, v) * self.scalar.subs(u, v)
            + self.entity.normal(u, v) * self.scalar.vder(u, v)
    }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(2, 0, u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(1, 1, u, v) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(0, 2, u, v) }
}

impl<S, N> ParametricSurface3D for Offset<S, N>
where
    S: ParametricSurface3D,
    N: ParametricSurface<Point = Vector3, Vector = Vector3>,
{
}

impl<S, N, P, V> BoundedSurface for Offset<S, N>
where
    S: BoundedSurface<Point = P, Vector = V>,
    N: ParametricSurface<Point = V, Vector = V>,
    P: ControlPoint<f64, Diff = V>,
    V: ControlPoint<f64, Diff = V>,
{
}

impl<S, N, P, V> ParameterDivision2D for Offset<S, N>
where
    S: BoundedSurface<Point = P, Vector = V>,
    N: ParametricSurface<Point = V, Vector = V>,
    P: ControlPoint<f64, Diff = V>
        + EuclideanSpace<Scalar = f64, Diff = V>
        + MetricSpace<Metric = f64>
        + HashGen<f64>,
    V: ControlPoint<f64, Diff = V>,
{
    #[inline]
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        algo::surface::parameter_division(self, range, tol)
    }
}

impl<S, N, P, V> SearchParameter<D2> for Offset<S, N>
where
    S: BoundedSurface<Point = P, Vector = V> + SearchNearestParameter<D2, Point = P>,
    N: ParametricSurface<Point = V, Vector = V>,
    P: ControlPoint<f64, Diff = V> + Copy + Tolerance,
    V: ControlPoint<f64, Diff = V>,
{
    type Point = P;
    #[inline]
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let (u, v) = self.entity.search_nearest_parameter(point, hint, trials)?;
        match self.subs(u, v).near(&point) {
            true => Some((u, v)),
            false => None,
        }
    }
}

impl<S, N, P, V> SearchNearestParameter<D2> for Offset<S, N>
where
    S: BoundedSurface<Point = P, Vector = V>,
    N: ParametricSurface<Point = V, Vector = V>,
    P: ControlPoint<f64, Diff = V>
        + EuclideanSpace<Scalar = f64, Diff = V>
        + MetricSpace<Metric = f64>
        + Copy,
    V: ControlPoint<f64, Diff = V> + algo::surface::SsnpVector<Point = P>,
{
    type Point = P;
    #[inline]
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SPHint2D::Parameter(u, v) => (u, v),
            SPHint2D::Range(urange, vrange) => {
                algo::surface::presearch(self, point, (urange, vrange), PRESEARCH_DIVISION)
            }
            SPHint2D::None => {
                algo::surface::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::surface::search_nearest_parameter(self, point, hint, trials)
    }
}
