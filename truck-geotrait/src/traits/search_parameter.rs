/// Dimension for search nearest parameter
pub trait SearchParameterDimension {
    /// dimension
    const DIM: usize;
    /// parameter type, curve => f64, surface => (f64, f64)
    type Parameter;
    /// hint, curve => [`SearchParameterHint1D`], surface => [`SearchParameterHint2D`]
    type Hint;
}

/// curve geometry
#[derive(Clone, Copy, Debug)]
pub enum D1 {}

impl SearchParameterDimension for D1 {
    const DIM: usize = 1;
    type Parameter = f64;
    type Hint = SearchParameterHint1D;
}

/// curve geometry
#[derive(Clone, Copy, Debug)]
pub enum D2 {}

impl SearchParameterDimension for D2 {
    const DIM: usize = 2;
    type Parameter = (f64, f64);
    type Hint = SearchParameterHint2D;
}

/// hint for searching parameter for curve
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SearchParameterHint1D {
    /// a parameter near the answer
    Parameter(f64),
    /// the range of parameter including answer
    Range(f64, f64),
    /// There are no hint. In the case of `BoundedCurve`, most of the time the parameter range is applied.
    /// Such as planes, no hinting is needed in the first place.
    None,
}

impl From<f64> for SearchParameterHint1D {
    #[inline(always)]
    fn from(x: f64) -> SearchParameterHint1D { SearchParameterHint1D::Parameter(x) }
}

impl From<(f64, f64)> for SearchParameterHint1D {
    #[inline(always)]
    fn from(range: (f64, f64)) -> SearchParameterHint1D {
        SearchParameterHint1D::Range(range.0, range.1)
    }
}

impl From<Option<f64>> for SearchParameterHint1D {
    #[inline(always)]
    fn from(x: Option<f64>) -> SearchParameterHint1D {
        match x {
            Some(x) => x.into(),
            None => SearchParameterHint1D::None,
        }
    }
}

/// Renamed to [`SearchParameterHint1D`] for clarity.
#[deprecated(note = "renamed to SearchParameterHint1D for clarity")]
pub type SPHint1D = SearchParameterHint1D;

/// hint for searching parameter for surface
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SearchParameterHint2D {
    /// a parameter near the answer
    Parameter(f64, f64),
    /// the range of parameter including answer
    Range((f64, f64), (f64, f64)),
    /// There are no hint. If the algorithm needed a hint, it always returns None.
    None,
}

impl From<(f64, f64)> for SearchParameterHint2D {
    #[inline(always)]
    fn from(x: (f64, f64)) -> Self { Self::Parameter(x.0, x.1) }
}

impl From<((f64, f64), (f64, f64))> for SearchParameterHint2D {
    #[inline(always)]
    fn from(ranges: ((f64, f64), (f64, f64))) -> Self { Self::Range(ranges.0, ranges.1) }
}

impl From<Option<(f64, f64)>> for SearchParameterHint2D {
    #[inline(always)]
    fn from(x: Option<(f64, f64)>) -> Self {
        match x {
            Some(x) => x.into(),
            None => SearchParameterHint2D::None,
        }
    }
}

/// Renamed to [`SearchParameterHint2D`] for clarity.
#[deprecated(note = "renamed to SearchParameterHint2D for clarity")]
pub type SPHint2D = SearchParameterHint2D;

/// Search parameter `t` such that `self.evaluate(t)` is near point.
pub trait SearchParameter<Dim: SearchParameterDimension> {
    /// point
    type Point;
    /// Search parameter `t` such that `self.evaluate(t)` is near point.
    /// Returns `None` if could not find such parameter.
    fn search_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<Dim::Parameter>;
}

impl<Dim: SearchParameterDimension, T: SearchParameter<Dim>> SearchParameter<Dim> for &T {
    type Point = T::Point;
    #[inline(always)]
    fn search_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<Dim::Parameter> {
        T::search_parameter(*self, point, hint, trials)
    }
}

impl<Dim: SearchParameterDimension, T: SearchParameter<Dim>> SearchParameter<Dim> for Box<T> {
    type Point = T::Point;
    #[inline(always)]
    fn search_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<Dim::Parameter> {
        T::search_parameter(&**self, point, hint, trials)
    }
}

/// Search parameter `t` such that `self.evaluate(t)` is nearest point.
pub trait SearchNearestParameter<Dim: SearchParameterDimension> {
    /// point
    type Point;
    /// Search nearest parameter `t` such that `self.evaluate(t)` is nearest point.
    /// Returns `None` if could not find such parameter.
    fn search_nearest_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<Dim::Parameter>;
}

impl<Dim: SearchParameterDimension, T: SearchNearestParameter<Dim>> SearchNearestParameter<Dim>
    for &T
{
    type Point = T::Point;
    #[inline(always)]
    fn search_nearest_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<Dim::Parameter> {
        T::search_nearest_parameter(*self, point, hint, trials)
    }
}

impl<Dim: SearchParameterDimension, T: SearchNearestParameter<Dim>> SearchNearestParameter<Dim>
    for Box<T>
{
    type Point = T::Point;
    #[inline(always)]
    fn search_nearest_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<Dim::Parameter> {
        T::search_nearest_parameter(&**self, point, hint, trials)
    }
}
