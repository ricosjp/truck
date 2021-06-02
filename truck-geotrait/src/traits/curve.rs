/// Parametric curves
pub trait ParametricCurve: Clone {
    /// The curve is in the space of `Self::Point`.
    type Point;
    /// The derivation vector of the curve.
    type Vector;
    /// Substitutes the parameter `t`.
    fn subs(&self, t: f64) -> Self::Point;
    /// Returns the derivation.
    fn der(&self, t: f64) -> Self::Vector;
    /// Returns the 2nd-order derivation.
    fn der2(&self, t: f64) -> Self::Vector;
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
}

/// Implementation for the test of topological methods.
impl ParametricCurve for () {
    type Point = ();
    type Vector = ();
    fn subs(&self, _: f64) -> Self::Point {}
    fn der(&self, _: f64) -> Self::Vector {}
    fn der2(&self, _: f64) -> Self::Vector {}
    fn parameter_range(&self) -> (f64, f64) { (0.0, 1.0) }
}

/// Dividable curve
pub trait ParameterDivision1D {
    /// Creates the curve division
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> Vec<f64>;
}

impl ParametricCurve for (usize, usize) {
    type Point = usize;
    type Vector = usize;
    fn subs(&self, t: f64) -> Self::Point {
        match t < 0.5 {
            true => self.0,
            false => self.1,
        }
    }
    fn der(&self, _: f64) -> Self::Vector { self.1 - self.0 }
    fn der2(&self, _: f64) -> Self::Vector { self.1 - self.0 }
    fn parameter_range(&self) -> (f64, f64) { (0.0, 1.0) }
}
