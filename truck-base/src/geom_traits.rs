/// Parametric curves
pub trait Curve {
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
pub trait Surface {
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
