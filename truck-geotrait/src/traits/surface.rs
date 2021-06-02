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

/// 3D parametric surface
pub trait ParametricSurface3D: ParametricSurface<Point = Point3, Vector = Vector3> {
    /// Returns the normal vector at `(u, v)`.
    fn normal(&self, u: f64, v: f64) -> Vector3;
}

/// Bounded surface with parametric range
pub trait BoundedSurface: ParametricSurface {
    /// The range of the parameter of the surface.
    fn parameter_range(&self) -> ((f64, f64), (f64, f64));
}

/// Whether the surface includes the boundary curve.
pub trait IncludeCurve<C: ParametricCurve> {
    /// Returns whether the curve `curve` is included in the surface `self`.
    fn include(&self, curve: &C) -> bool;
}

/// Dividable surface
pub trait ParameterDivision2D {
    /// Creates the surface division
    fn parameter_division(&self, range: ((f64, f64), (f64, f64)), tol: f64) -> (Vec<f64>, Vec<f64>);
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
