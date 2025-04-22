use super::*;
use algo::surface::SspVector;

impl<C, V: Copy> ExtrudedCurve<C, V> {
    /// Creates a linear extruded curve by extrusion.
    #[inline(always)]
    pub const fn by_extrusion(curve: C, vector: V) -> Self { Self { curve, vector } }

    /// Returns the curve before extruded.
    #[inline(always)]
    pub const fn entity_curve(&self) -> &C { &self.curve }
    /// Into the curve before revoluted.
    #[inline(always)]
    pub fn into_entity_curve(self) -> C { self.curve }

    /// Returns the vector of extruded curve.
    #[inline(always)]
    pub const fn extruding_vector(&self) -> V { self.vector }
}

impl<C> ParametricSurface for ExtrudedCurve<C, C::Vector>
where
    C: ParametricCurve,
    C::Point: EuclideanSpace<Scalar = f64, Diff = C::Vector>,
    C::Vector: VectorSpace<Scalar = f64>,
{
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn der_mn(&self, u: f64, v: f64, m: usize, n: usize) -> Self::Vector {
        match (m, n) {
            (0, 0) => self.subs(u, v).to_vec(),
            (0, 1) => self.vector,
            (_, 0) => self.curve.der_n(u, m),
            _ => C::Vector::zero(),
        }
    }
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> C::Point { self.curve.subs(u) + self.vector * v }
    #[inline(always)]
    fn uder(&self, u: f64, _: f64) -> C::Vector { self.curve.der(u) }
    #[inline(always)]
    fn vder(&self, _: f64, _: f64) -> C::Vector { self.vector }
    #[inline(always)]
    fn uuder(&self, u: f64, _: f64) -> C::Vector { self.curve.der2(u) }
    #[inline(always)]
    fn uvder(&self, _: f64, _: f64) -> C::Vector { C::Vector::zero() }
    #[inline(always)]
    fn vvder(&self, _: f64, _: f64) -> C::Vector { C::Vector::zero() }
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        (
            self.curve.parameter_range(),
            (Bound::Included(0.0), Bound::Included(1.0)),
        )
    }
    #[inline(always)]
    fn u_period(&self) -> Option<f64> { self.curve.period() }
}

impl<C: ParametricCurve3D> ParametricSurface3D for ExtrudedCurve<C, Vector3> {
    #[inline(always)]
    fn normal(&self, u: f64, _: f64) -> C::Vector {
        self.curve.der(u).cross(self.vector).normalize()
    }
}

impl<C, V> BoundedSurface for ExtrudedCurve<C, V>
where
    C: BoundedCurve,
    Self: ParametricSurface,
{
}

impl<C: ParameterDivision1D, V> ParameterDivision2D for ExtrudedCurve<C, V> {
    #[inline(always)]
    fn parameter_division(
        &self,
        (urange, vrange): ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        (
            self.curve.parameter_division(urange, tol).0,
            vec![vrange.0, vrange.1],
        )
    }
}

impl<P, C> SearchParameter<D2> for ExtrudedCurve<C, P::Diff>
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + Tolerance,
    P::Diff: SspVector<Point = P>,
    C: ParametricCurve<Point = P, Vector = P::Diff> + BoundedCurve,
{
    type Point = P;
    #[inline(always)]
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: P,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SPHint2D::Parameter(x, y) => (x, y),
            SPHint2D::Range(range0, range1) => {
                algo::surface::presearch(self, point, (range0, range1), PRESEARCH_DIVISION)
            }
            SPHint2D::None => {
                algo::surface::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::surface::search_parameter(self, point, hint, trials)
    }
}

impl<C: ParametricCurve3D + BoundedCurve> SearchNearestParameter<D2> for ExtrudedCurve<C, Vector3> {
    type Point = Point3;
    #[inline(always)]
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SPHint2D::Parameter(x, y) => (x, y),
            SPHint2D::Range(range0, range1) => {
                algo::surface::presearch(self, point, (range0, range1), PRESEARCH_DIVISION)
            }
            SPHint2D::None => {
                algo::surface::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::surface::search_nearest_parameter(self, point, hint, trials)
    }
}

impl<C: Invertible> Invertible for ExtrudedCurve<C, Vector3> {
    #[inline(always)]
    fn invert(&mut self) { self.curve.invert() }
    #[inline(always)]
    fn inverse(&self) -> Self {
        Self {
            curve: self.curve.inverse(),
            vector: self.vector,
        }
    }
}

impl<C: Transformed<Matrix4>> Transformed<Matrix4> for ExtrudedCurve<C, Vector3> {
    fn transform_by(&mut self, trans: Matrix4) {
        self.curve.transform_by(trans);
        self.vector = trans.transform_vector(self.vector);
    }
    fn transformed(&self, trans: Matrix4) -> Self {
        Self {
            curve: self.curve.transformed(trans),
            vector: trans.transform_vector(self.vector),
        }
    }
}
