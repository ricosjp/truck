use super::*;

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
}

impl<'a, S: ParametricSurface> ParametricSurface for &'a S {
    type Point = S::Point;
    type Vector = S::Vector;
    fn subs(&self, u: f64, v: f64) -> Self::Point { (*self).subs(u, v) }
    fn uder(&self, u: f64, v: f64) -> Self::Vector { (*self).uder(u, v) }
    fn vder(&self, u: f64, v: f64) -> Self::Vector { (*self).vder(u, v) }
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { (*self).uuder(u, v) }
    fn uvder(&self, u: f64, v: f64) -> Self::Vector { (*self).uvder(u, v) }
    fn vvder(&self, u: f64, v: f64) -> Self::Vector { (*self).vvder(u, v) }
}

impl<S: ParametricSurface> ParametricSurface for Box<S> {
    type Point = S::Point;
    type Vector = S::Vector;
    fn subs(&self, u: f64, v: f64) -> Self::Point { S::subs(&*self, u, v) }
    fn uder(&self, u: f64, v: f64) -> Self::Vector { S::uder(&*self, u, v) }
    fn vder(&self, u: f64, v: f64) -> Self::Vector { S::vder(&*self, u, v) }
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { S::uuder(&*self, u, v) }
    fn uvder(&self, u: f64, v: f64) -> Self::Vector { S::uvder(&*self, u, v) }
    fn vvder(&self, u: f64, v: f64) -> Self::Vector { S::vvder(&*self, u, v) }
}

/// 2D parametric surface
pub trait ParametricSurface2D: ParametricSurface<Point = Point2, Vector = Vector2> {}
impl<S: ParametricSurface<Point = Point2, Vector = Vector2>> ParametricSurface2D for S {}

/// 3D parametric surface
pub trait ParametricSurface3D: ParametricSurface<Point = Point3, Vector = Vector3> {
    /// Returns the normal vector at `(u, v)`.
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        self.uder(u, v).cross(self.vder(u, v)).normalize()
    }
}

impl<'a, S: ParametricSurface3D> ParametricSurface3D for &'a S {
    fn normal(&self, u: f64, v: f64) -> Vector3 { (*self).normal(u, v) }
}

impl<S: ParametricSurface3D> ParametricSurface3D for Box<S> {
    fn normal(&self, u: f64, v: f64) -> Vector3 { S::normal(&*self, u, v) }
}

/// Bounded surface with parametric range
pub trait BoundedSurface: ParametricSurface {
    /// The range of the parameter of the surface.
    fn parameter_range(&self) -> ((f64, f64), (f64, f64));
}

impl<'a, S: BoundedSurface> BoundedSurface for &'a S {
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { (*self).parameter_range() }
}

impl<S: BoundedSurface> BoundedSurface for Box<S> {
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { S::parameter_range(&*self) }
}

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
        S::parameter_division(&*self, range, tol)
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
}

/// Implementation for the test of topological methods.
impl BoundedSurface for () {
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { ((0.0, 1.0), (0.0, 1.0)) }
}

/// Implementation for the test of topological methods.
impl IncludeCurve<()> for () {
    fn include(&self, _: &()) -> bool { true }
}
