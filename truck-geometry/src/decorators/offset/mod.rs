use super::*;
use truck_base::cgmath_extend_traits::control_point::ControlPoint;

impl<T, N> Offset<T, N> {
    /// constructor
    #[inline(always)]
    pub const fn new(entity: T, offset: N) -> Self { Self { entity, offset } }
    /// Returns entity geometry
    #[inline(always)]
    pub const fn entity(&self) -> &T { &self.entity }
    /// Returns offset
    #[inline(always)]
    pub const fn offset(&self) -> &N { &self.offset }
}

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
    /// `entity`のレンジを採用する
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
    /// `entity`のレンジを採用する
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

impl<T, F> NormalField<T, F> {
    /// constructor
    #[inline(always)]
    pub fn new(entity: T, scalar: F) -> Self { Self { entity, scalar } }
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
            .for_each(|vec| *vec = Vector2::new(vec.y, -vec.x));
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

impl<S, F> ParametricSurface for NormalField<S, F>
where
    S: ParametricSurface3D,
    F: ScalarFunctionD2,
{
    type Point = Vector3;
    type Vector = Vector3;
    #[inline(always)]
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
    fn subs(&self, u: f64, v: f64) -> Self::Point { self.der_mn(0, 0, u, v) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(1, 0, u, v) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(0, 1, u, v) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(2, 0, u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(1, 1, u, v) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(0, 2, u, v) }
}
