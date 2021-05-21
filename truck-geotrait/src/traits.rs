use truck_base::cgmath64::*;

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

/// Search parameter `t` such that `self.subs(t)` is near point.
pub trait SearchParameter {
    /// point
    type Point;
    /// curve => `f64`, surface => `(f64, f64)`
    type Parameter;
    /// Search parameter `t` such that `self.subs(t)` is near point.  
    /// Returns `None` if could not find such parameter.
    fn search_parameter(
        &self,
        point: Self::Point,
        hint: Self::Parameter,
        trial: usize,
    ) -> Option<Self::Parameter>;
}

/// Whether the surface includes the boundary curve.
pub trait IncludeCurve<C: ParametricCurve> {
    /// Returns whether the curve `curve` is included in the surface `self`.
    fn include(&self, curve: &C) -> bool;
}

/// Oriented and reversible
pub trait Invertible {
    /// Inverts `self`
    fn invert(&mut self);
    /// Returns the inverse.
    fn inverse(&self) -> Self;
}

/// Transform geometry
pub trait Transformed<T> {
    /// transform by `trans`.
    fn transform_by(&mut self, trans: T);
    /// transformed geometry by `trans`.
    fn transformed(&self, trans: T) -> Self;
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
impl ParametricCurve for () {
    type Point = ();
    type Vector = ();
    fn subs(&self, _: f64) -> Self::Point {}
    fn der(&self, _: f64) -> Self::Vector {}
    fn der2(&self, _: f64) -> Self::Vector {}
    fn parameter_range(&self) -> (f64, f64) { (0.0, 1.0) }
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

impl Invertible for (usize, usize) {
    fn invert(&mut self) { *self = (self.1, self.0); }
    fn inverse(&self) -> Self { (self.1, self.0) }
}
