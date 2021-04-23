use crate::*;

impl<C, S> CurveOnSurface<C, S> {
    /// Creates composited
    #[inline(always)]
    pub fn new(curve: C, surface: S) -> CurveOnSurface<C, S> { CurveOnSurface { curve, surface } }

    /// Returns the reference to the previous map
    #[inline(always)]
    pub fn curve(&self) -> &C { &self.curve }

    /// Returns the reference to the previous map
    #[inline(always)]
    pub fn surface(&self) -> &S { &self.surface }
}

impl<C, S> ParametricCurve for CurveOnSurface<C, S>
where
    C: ParametricCurve<Point = Point2, Vector = Vector2>,
    S: ParametricSurface,
    S::Vector: VectorSpace<Scalar = f64>,
{
    type Point = S::Point;
    type Vector = S::Vector;
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point {
        let pt = self.curve.subs(t);
        self.surface.subs(pt[0], pt[1])
    }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector {
        let pt = self.curve.subs(t);
        let der = self.curve.der(t);
        self.surface.uder(pt[0], pt[1]) * der[0] + self.surface.vder(pt[0], pt[1]) * der[1]
    }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector {
        let pt = self.curve.subs(t);
        let der = self.curve.der(t);
        let der2 = self.curve.der2(t);
        self.surface.uuder(pt[0], pt[1]) * der[0] * der[0]
            + self.surface.uvder(pt[0], pt[1]) * der[0] * der[1] * 2.0
            + self.surface.vvder(pt[0], pt[1]) * der[1] * der[1]
            + self.surface.uder(pt[0], pt[1]) * der2[0]
            + self.surface.vder(pt[0], pt[1]) * der2[1]
    }
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) { self.curve.parameter_range() }
}

impl<C, S> ParameterDivision1D for CurveOnSurface<C, S>
where
    C: ParametricCurve<Point = Point2, Vector = Vector2>,
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>,
    S::Vector: VectorSpace<Scalar = f64>,
{
    fn parameter_division(&self, tol: f64) -> Vec<f64> {
        algo::curve::parameter_division(self, self.parameter_range(), tol)
    }
}
