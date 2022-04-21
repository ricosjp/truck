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

impl<C: SearchNearestParameter> SearchNearestParameter for TrimmedCurve<C> {
    type Point = C::Point;
    type Parameter = C::Parameter;
    #[inline(always)]
    fn search_nearest_parameter(
        &self,
        pt: C::Point,
        hint: Option<C::Parameter>,
        trials: usize,
    ) -> Option<C::Parameter> {
        self.curve.search_nearest_parameter(pt, hint, trials)
    }
}

impl<C: SearchParameter> SearchParameter for TrimmedCurve<C> {
    type Point = C::Point;
    type Parameter = C::Parameter;
    #[inline(always)]
    fn search_parameter(
        &self,
        pt: C::Point,
        hint: Option<C::Parameter>,
        trials: usize,
    ) -> Option<C::Parameter> {
        self.curve.search_parameter(pt, hint, trials)
    }
}

impl<C: ParameterDivision1D> ParameterDivision1D for TrimmedCurve<C> {
    type Point = C::Point;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        self.curve.parameter_division(range, tol)
    }
}
