use super::*;

type Tuple = (f64, f64);
/// Parametric surface
pub trait ParametricSurface: Clone {
    /// The surface is in the space of `Self::Point`.
    type Point;
    /// The derivation vector of the curve.
    type Vector;
    /// Substitutes the parameter `(u, v)`.
    fn subs(&self, u: f64, v: f64) -> Self::Point;
    /// Returns the derivation by `u`.
    fn uder(&self, u: f64, v: f64) -> Self::Vector;
    /// Returns the derivation by `v`.
    fn vder(&self, u: f64, v: f64) -> Self::Vector;
    /// Returns the 2nd-order derivation by `u`.
    fn uuder(&self, u: f64, v: f64) -> Self::Vector;
    /// Returns the 2nd-order derivation by both `u` and `v`.
    fn uvder(&self, u: f64, v: f64) -> Self::Vector;
    /// Returns the 2nd-order derivation by `v`.
    fn vvder(&self, u: f64, v: f64) -> Self::Vector;
    /// The range of the parameter of the surface.
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        use Bound::Unbounded as X;
        ((X, X), (X, X))
    }
    /// Return the ends of `parameter_range` by tuple.
    /// If the range is unbounded, return `None``.
    #[inline(always)]
    fn try_range_tuple(&self) -> (Option<Tuple>, Option<Tuple>) {
        let ((u0, u1), (v0, v1)) = self.parameter_range();
        (
            bound2opt(u0).and_then(move |u0| bound2opt(u1).map(move |u1| (u0, u1))),
            bound2opt(v0).and_then(move |v0| bound2opt(v1).map(move |v1| (v0, v1))),
        )
    }
    /// `None` in default; `Some(period)` if periodic w.r.t. parameter u.
    #[inline(always)]
    fn u_period(&self) -> Option<f64> { None }
    /// `None` in default; `Some(period)` if periodic w.r.t. parameter v.
    #[inline(always)]
    fn v_period(&self) -> Option<f64> { None }
}

impl<'a, S: ParametricSurface> ParametricSurface for &'a S {
    type Point = S::Point;
    type Vector = S::Vector;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point { (*self).subs(u, v) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector { (*self).uder(u, v) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Self::Vector { (*self).vder(u, v) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { (*self).uuder(u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Self::Vector { (*self).uvder(u, v) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Self::Vector { (*self).vvder(u, v) }
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) { (*self).parameter_range() }
    #[inline(always)]
    fn u_period(&self) -> Option<f64> { (*self).u_period() }
    #[inline(always)]
    fn v_period(&self) -> Option<f64> { (*self).v_period() }
}

impl<S: ParametricSurface> ParametricSurface for Box<S> {
    type Point = S::Point;
    type Vector = S::Vector;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point { (**self).subs(u, v) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector { (**self).uder(u, v) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Self::Vector { (**self).vder(u, v) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { (**self).uuder(u, v) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Self::Vector { (**self).uvder(u, v) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Self::Vector { (**self).vvder(u, v) }
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) { (**self).parameter_range() }
    #[inline(always)]
    fn u_period(&self) -> Option<f64> { (**self).u_period() }
    #[inline(always)]
    fn v_period(&self) -> Option<f64> { (**self).v_period() }
}

/// 2D parametric surface
pub trait ParametricSurface2D: ParametricSurface<Point = Point2, Vector = Vector2> {}
impl<S: ParametricSurface<Point = Point2, Vector = Vector2>> ParametricSurface2D for S {}

/// 3D parametric surface
pub trait ParametricSurface3D: ParametricSurface<Point = Point3, Vector = Vector3> {
    /// Returns the normal vector at `(u, v)`.
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        self.uder(u, v).cross(self.vder(u, v)).normalize()
    }
}

impl<'a, S: ParametricSurface3D> ParametricSurface3D for &'a S {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 { (*self).normal(u, v) }
}

impl<S: ParametricSurface3D> ParametricSurface3D for Box<S> {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 { (**self).normal(u, v) }
}

/// Bounded surface with parametric range i.e. it is guaranteed that the return value of `parameter_range` is not `Bound::Unbounded`.
pub trait BoundedSurface: ParametricSurface {
    /// Return the ends of `parameter_range` by tuple.
    #[inline(always)]
    fn range_tuple(&self) -> ((f64, f64), (f64, f64)) {
        let (urange, vrange) = self.try_range_tuple();
        (
            urange.expect(UNBOUNDED_ERROR),
            vrange.expect(UNBOUNDED_ERROR),
        )
    }
}

impl<'a, S: BoundedSurface> BoundedSurface for &'a S {}

impl<S: BoundedSurface> BoundedSurface for Box<S> {}

/// Whether the surface includes the boundary curve.
pub trait IncludeCurve<C: ParametricCurve> {
    /// Returns whether the curve `curve` is included in the surface `self`.
    fn include(&self, curve: &C) -> bool;
}

/// Dividable surface
pub trait ParameterDivision2D {
    /// Creates the surface division
    ///
    /// # Panics
    ///
    /// `tol` must be more than `TOLERANCE`.
    fn parameter_division(&self, range: ((f64, f64), (f64, f64)), tol: f64)
        -> (Vec<f64>, Vec<f64>);
}

impl<'a, S: ParameterDivision2D> ParameterDivision2D for &'a S {
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        (*self).parameter_division(range, tol)
    }
}

impl<S: ParameterDivision2D> ParameterDivision2D for Box<S> {
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        (**self).parameter_division(range, tol)
    }
}

/// Implementation for the test of topological methods.
impl ParametricSurface for () {
    type Point = ();
    type Vector = ();
    fn subs(&self, _: f64, _: f64) -> Self::Point {}
    fn uder(&self, _: f64, _: f64) -> Self::Vector {}
    fn vder(&self, _: f64, _: f64) -> Self::Vector {}
    fn uuder(&self, _: f64, _: f64) -> Self::Vector {}
    fn uvder(&self, _: f64, _: f64) -> Self::Vector {}
    fn vvder(&self, _: f64, _: f64) -> Self::Vector {}
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        (
            (Bound::Included(0.0), Bound::Included(1.0)),
            (Bound::Included(0.0), Bound::Included(1.0)),
        )
    }
}

/// Implementation for the test of topological methods.
impl BoundedSurface for () {}

/// Implementation for the test of topological methods.
impl IncludeCurve<()> for () {
    fn include(&self, _: &()) -> bool { true }
}
