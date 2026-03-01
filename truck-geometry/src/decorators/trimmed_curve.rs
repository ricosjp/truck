use super::*;
use truck_geotrait::ParametricCurve as PcurveTrait;

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

impl<C: PcurveTrait> PcurveTrait for TrimmedCurve<C> {
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn derivative_n(&self, n: usize, t: f64) -> Self::Vector { self.curve.derivative_n(n, t) }
    #[inline(always)]
    fn evaluate(&self, t: f64) -> Self::Point { self.curve.evaluate(t) }
    #[inline(always)]
    fn derivative(&self, t: f64) -> Self::Vector { self.curve.derivative(t) }
    #[inline(always)]
    fn derivative_2(&self, t: f64) -> Self::Vector { self.curve.derivative_2(t) }
    #[inline(always)]
    fn period(&self) -> Option<f64> { self.curve.period() }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange {
        (Bound::Included(self.range.0), Bound::Included(self.range.1))
    }
}

impl<C: PcurveTrait> BoundedCurve for TrimmedCurve<C> {}

impl<C: PcurveTrait> Cut for TrimmedCurve<C> {
    fn cut(&mut self, t: f64) -> Self {
        let (t0, t1) = self.range;
        self.range = (t0, t);
        Self::new(self.curve.clone(), (t, t1))
    }
}

impl<C: SearchNearestParameter<D1>> SearchNearestParameter<D1> for TrimmedCurve<C> {
    type Point = C::Point;
    #[inline(always)]
    fn search_nearest_parameter<H: Into<SearchParameterHint1D>>(
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
    fn search_parameter<H: Into<SearchParameterHint1D>>(
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
