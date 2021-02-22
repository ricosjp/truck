/// Parametric curves
pub trait Curve: Clone {
    /// The curve is in the space of `Self::Point`.
    type Point;
    /// The derivation vector of the curve.
    type Vector;
    /// Substitutes the parameter `t`.
    fn subs(&self, t: f64) -> Self::Point;
    /// Returns the derivation.
    fn der(&self, t: f64) -> Self::Vector;
    /// The range of the parameter of the curve.
    fn parameter_range(&self) -> (f64, f64);
    /// The front end point of the curve.
    fn front(&self) -> Self::Point {
        let (t, _) = self.parameter_range();
        self.subs(t)
    }
    /// The back end point of the curve.
    fn back(&self) -> Self::Point {
        let (_, t) = self.parameter_range();
        self.subs(t)
    }
    /// Returns the inverse of the curve.
    fn inverse(&self) -> Self;
}
/// Parametric surface
pub trait Surface: Clone {
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
    /// Returns the normal vector at `(u, v)`.
    fn normal(&self, u: f64, v: f64) -> Self::Vector;
    /// The range of the parameter of the surface.
    fn parameter_range(&self) -> ((f64, f64), (f64, f64));
    /// Returns the inverse of the surface.
    fn inverse(&self) -> Self;
}

/// Whether the surface includes the boundary curve.
pub trait IncludeCurve<C: Curve>: Surface {
    /// Returns whether the curve `curve` is included in the surface `self`.
    fn include(&self, curve: &C) -> bool;
}

/// Dividable curve
pub trait ParameterDivision1D {
    /// Creates the curve division
    fn parameter_division(&self, tol: f64) -> Vec<f64>;
}

/// Dividable surface
pub trait ParameterDivision2D {
    /// Creates the surface division
    fn parameter_division(&self, tol: f64) -> (Vec<f64>, Vec<f64>);
}

/// Implementation for the test of topological methods.
impl Curve for () {
    type Point = ();
    type Vector = ();
    fn subs(&self, _: f64) -> Self::Point {}
    fn der(&self, _: f64) -> Self::Vector {}
    fn parameter_range(&self) -> (f64, f64) { (0.0, 1.0) }
    fn inverse(&self) -> Self {}
}

/// Implementation for the test of topological methods.
impl Surface for () {
    type Point = ();
    type Vector = ();
    fn subs(&self, _: f64, _: f64) -> Self::Point {}
    fn uder(&self, _: f64, _: f64) -> Self::Vector {}
    fn vder(&self, _: f64, _: f64) -> Self::Vector {}
    fn normal(&self, _: f64, _: f64) -> Self::Vector {}
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) {
        ((0.0, 1.0), (0.0, 1.0))
    }
    fn inverse(&self) -> Self {}
}

/// Implementation for the test of topological methods.
impl Curve for (usize, usize) {
    type Point = usize;
    type Vector = usize;
    fn subs(&self, t: f64) -> Self::Point {
        match t < 0.5 {
            true => self.0,
            false => self.1,
        }
    }
    fn der(&self, _: f64) -> Self::Vector { self.1 - self.0 }
    fn parameter_range(&self) -> (f64, f64) { (0.0, 1.0) }
    fn inverse(&self) -> Self { (self.1, self.0) }
}
