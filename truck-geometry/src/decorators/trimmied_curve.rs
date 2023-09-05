use super::*;

impl<C> TrimmedCurve<C> {
    /// constructor
    #[inline(always)]
    pub const fn new(curve: C, range: (f64, f64)) -> Self { Self { curve, range } }
    /// Returns the reference of non-trimmed curve
    #[inline(always)]
    pub const fn curve(&self) -> &C { &self.curve }
    /// Returns the mutable reference of non-trimmed curve
    #[inline(always)]
    pub fn curve_mut(&mut self) -> &mut C { &mut self.curve }
}

impl<C: ParametricCurve> ParametricCurve for TrimmedCurve<C> {
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point { self.curve.subs(t) }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector { self.curve.der(t) }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector { self.curve.der2(t) }
}

impl<C: ParametricCurve> BoundedCurve for TrimmedCurve<C> {
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) { self.range }
}

impl<C: SearchNearestParameter<D1>> SearchNearestParameter<D1> for TrimmedCurve<C> {
    type Point = C::Point;
    #[inline(always)]
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        pt: C::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        self.curve.search_nearest_parameter(pt, hint, trials)
    }
}

impl<C: SearchParameter<D1>> SearchParameter<D1> for TrimmedCurve<C> {
    type Point = C::Point;
    #[inline(always)]
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        pt: C::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        self.curve.search_parameter(pt, hint, trials)
    }
}

impl<C: ParameterDivision1D> ParameterDivision1D for TrimmedCurve<C> {
    type Point = C::Point;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        self.curve.parameter_division(range, tol)
    }
}
