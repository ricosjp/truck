/// Dimension for search nearest parameter
pub trait SPDimension {
    /// dimension
    const DIM: usize;
    /// parameter type, curve => f64, surface => (f64, f64)
    type Parameter;
    /// `SPHintXX`
    type Hint;
}

/// curve geometry
#[derive(Clone, Copy, Debug)]
pub enum D1 {}

impl SPDimension for D1 {
    const DIM: usize = 1;
    type Parameter = f64;
    type Hint = SPHint1D;
}

/// curve geometry
#[derive(Clone, Copy, Debug)]
pub enum D2 {}

impl SPDimension for D2 {
    const DIM: usize = 2;
    type Parameter = (f64, f64);
    type Hint = SPHint2D;
}

/// hint for searching parameter for curve
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SPHint1D {
    /// a parameter near the answer
    Parameter(f64),
    /// the range of parameter including answer
    Range(f64, f64),
    /// There are no hint. In the case of `BoundedCurve`, most of the time the parameter range is applied.
    /// Such as planes, no hinting is needed in the first place.
    #[default]
    None,
}

impl From<f64> for SPHint1D {
    fn from(x: f64) -> SPHint1D { SPHint1D::Parameter(x) }
}

impl From<(f64, f64)> for SPHint1D {
    fn from(range: (f64, f64)) -> SPHint1D { SPHint1D::Range(range.0, range.1) }
}

impl From<Option<f64>> for SPHint1D {
    fn from(x: Option<f64>) -> SPHint1D {
        match x {
            Some(x) => x.into(),
            None => SPHint1D::None,
        }
    }
}

/// hint for searching parameter for surface
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SPHint2D {
    /// a parameter near the answer
    Parameter(f64, f64),
    /// the range of parameter including answer
    Range((f64, f64), (f64, f64)),
    /// There are no hint. If the algorithm needed a hint, it always returns None.
    #[default]
    None,
}

impl From<(f64, f64)> for SPHint2D {
    fn from(x: (f64, f64)) -> Self { Self::Parameter(x.0, x.1) }
}

impl From<((f64, f64), (f64, f64))> for SPHint2D {
    fn from(ranges: ((f64, f64), (f64, f64))) -> Self { Self::Range(ranges.0, ranges.1) }
}

impl From<Option<(f64, f64)>> for SPHint2D {
    fn from(x: Option<(f64, f64)>) -> Self {
        match x {
            Some(x) => x.into(),
            None => SPHint2D::None,
        }
    }
}

/// Search parameter `t` such that `self.subs(t)` is near point.
pub trait SearchParameter<Dim: SPDimension> {
    /// point
    type Point;
    /// Search parameter `t` such that `self.subs(t)` is near point.  
    /// Returns `None` if could not find such parameter.
    fn search_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trial: usize,
    ) -> Option<Dim::Parameter>;
}

impl<'a, Dim: SPDimension, T: SearchParameter<Dim>> SearchParameter<Dim> for &'a T {
    type Point = T::Point;
    fn search_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trial: usize,
    ) -> Option<Dim::Parameter> {
        T::search_parameter(*self, point, hint, trial)
    }
}

impl<Dim: SPDimension, T: SearchParameter<Dim>> SearchParameter<Dim> for Box<T> {
    type Point = T::Point;
    fn search_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trial: usize,
    ) -> Option<Dim::Parameter> {
        T::search_parameter(&**self, point, hint, trial)
    }
}

/// Search parameter `t` such that `self.subs(t)` is nearest point.
pub trait SearchNearestParameter<Dim: SPDimension> {
    /// point
    type Point;
    /// Search nearest parameter `t` such that `self.subs(t)` is nearest point.  
    /// Returns `None` if could not find such parameter.
    fn search_nearest_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trial: usize,
    ) -> Option<Dim::Parameter>;
}

impl<'a, Dim: SPDimension, T: SearchNearestParameter<Dim>> SearchNearestParameter<Dim> for &'a T {
    type Point = T::Point;
    fn search_nearest_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trial: usize,
    ) -> Option<Dim::Parameter> {
        T::search_nearest_parameter(*self, point, hint, trial)
    }
}

impl<Dim: SPDimension, T: SearchNearestParameter<Dim>> SearchNearestParameter<Dim> for Box<T> {
    type Point = T::Point;
    fn search_nearest_parameter<H: Into<Dim::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trial: usize,
    ) -> Option<Dim::Parameter> {
        T::search_nearest_parameter(&**self, point, hint, trial)
    }
}
