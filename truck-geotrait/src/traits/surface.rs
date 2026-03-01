use super::*;

type Tuple = (f64, f64);
/// A surface defined by a mapping `S(u, v)` from two parameters to a point.
///
/// As `(u, v)` varies over the parameter domain, the returned points sweep out the surface.
/// Partial derivatives give tangent directions (`derivative_u`, `derivative_v`) and curvature
/// information (`derivative_uu`, `derivative_uv`, `derivative_vv`) at any parameter pair.
/// New code should prefer `evaluate` and `derivative*` methods.
/// Legacy `subs` and `*der` methods are kept for compatibility.
pub trait ParametricSurface: Clone {
    /// The point type the surface maps into (e.g. `Point3`).
    type Point;
    /// The derivative vector type (e.g. `Vector3`).
    type Vector: Zero + Copy;
    /// Evaluates the surface at `(u, v)`, returning the point `S(u, v)`.
    fn evaluate(&self, u: f64, v: f64) -> Self::Point;
    /// Returns `∂S/∂u` at `(u, v)`.
    fn derivative_u(&self, u: f64, v: f64) -> Self::Vector;
    /// Returns `∂S/∂v` at `(u, v)`.
    fn derivative_v(&self, u: f64, v: f64) -> Self::Vector;
    /// Returns `∂²S/∂u²` at `(u, v)`.
    fn derivative_uu(&self, u: f64, v: f64) -> Self::Vector;
    /// Returns `∂²S/∂u∂v` at `(u, v)`.
    fn derivative_uv(&self, u: f64, v: f64) -> Self::Vector;
    /// Returns `∂²S/∂v²` at `(u, v)`.
    fn derivative_vv(&self, u: f64, v: f64) -> Self::Vector;
    /// Returns the mixed partial `∂^(m+n)S / ∂u^m ∂v^n` at `(u, v)`.
    fn derivative_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector;
    /// Returns all derivatives at `(u, v)` up to order `max_order`.
    fn derivatives(&self, max_order: usize, u: f64, v: f64) -> SurfaceDerivatives<Self::Vector> {
        let mut derivs = SurfaceDerivatives::new(max_order);
        (0..=max_order)
            .for_each(|m| (0..=max_order - m).for_each(|n| derivs[m][n] = self.derivative_mn(m, n, u, v)));
        derivs
    }
    /// Deprecated: use [`evaluate`](ParametricSurface::evaluate).
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point { self.evaluate(u, v) }
    /// Deprecated: use [`derivative_u`](ParametricSurface::derivative_u).
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector { self.derivative_u(u, v) }
    /// Deprecated: use [`derivative_v`](ParametricSurface::derivative_v).
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Self::Vector { self.derivative_v(u, v) }
    /// Deprecated: use [`derivative_uu`](ParametricSurface::derivative_uu).
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { self.derivative_uu(u, v) }
    /// Deprecated: use [`derivative_uv`](ParametricSurface::derivative_uv).
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Self::Vector { self.derivative_uv(u, v) }
    /// Deprecated: use [`derivative_vv`](ParametricSurface::derivative_vv).
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Self::Vector { self.derivative_vv(u, v) }
    /// Deprecated: use [`derivative_mn`](ParametricSurface::derivative_mn).
    #[inline(always)]
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        self.derivative_mn(m, n, u, v)
    }
    /// Deprecated: use [`derivatives`](ParametricSurface::derivatives).
    #[inline(always)]
    fn ders(&self, max_order: usize, u: f64, v: f64) -> SurfaceDerivatives<Self::Vector> {
        self.derivatives(max_order, u, v)
    }
    /// The range of the parameter of the surface.
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        use Bound::Unbounded as X;
        ((X, X), (X, X))
    }
    /// Return the ends of `parameter_range` by tuple.
    /// If the range is unbounded, return `None`.
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

impl<S: ParametricSurface> ParametricSurface for &S {
    type Point = S::Point;
    type Vector = S::Vector;
    #[inline(always)]
    fn evaluate(&self, u: f64, v: f64) -> Self::Point { (*self).evaluate(u, v) }
    #[inline(always)]
    fn derivative_u(&self, u: f64, v: f64) -> Self::Vector { (*self).derivative_u(u, v) }
    #[inline(always)]
    fn derivative_v(&self, u: f64, v: f64) -> Self::Vector { (*self).derivative_v(u, v) }
    #[inline(always)]
    fn derivative_uu(&self, u: f64, v: f64) -> Self::Vector { (*self).derivative_uu(u, v) }
    #[inline(always)]
    fn derivative_uv(&self, u: f64, v: f64) -> Self::Vector { (*self).derivative_uv(u, v) }
    #[inline(always)]
    fn derivative_vv(&self, u: f64, v: f64) -> Self::Vector { (*self).derivative_vv(u, v) }
    #[inline(always)]
    fn derivative_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        (*self).derivative_mn(m, n, u, v)
    }
    #[inline(always)]
    fn derivatives(&self, max_order: usize, u: f64, v: f64) -> SurfaceDerivatives<Self::Vector> {
        (*self).derivatives(max_order, u, v)
    }
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
    fn evaluate(&self, u: f64, v: f64) -> Self::Point { (**self).evaluate(u, v) }
    #[inline(always)]
    fn derivative_u(&self, u: f64, v: f64) -> Self::Vector { (**self).derivative_u(u, v) }
    #[inline(always)]
    fn derivative_v(&self, u: f64, v: f64) -> Self::Vector { (**self).derivative_v(u, v) }
    #[inline(always)]
    fn derivative_uu(&self, u: f64, v: f64) -> Self::Vector { (**self).derivative_uu(u, v) }
    #[inline(always)]
    fn derivative_uv(&self, u: f64, v: f64) -> Self::Vector { (**self).derivative_uv(u, v) }
    #[inline(always)]
    fn derivative_vv(&self, u: f64, v: f64) -> Self::Vector { (**self).derivative_vv(u, v) }
    #[inline(always)]
    fn derivative_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        (**self).derivative_mn(m, n, u, v)
    }
    #[inline(always)]
    fn derivatives(&self, max_order: usize, u: f64, v: f64) -> SurfaceDerivatives<Self::Vector> {
        (**self).derivatives(max_order, u, v)
    }
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
        self.derivative_u(u, v)
            .cross(self.derivative_v(u, v))
            .normalize()
    }
    /// Returns the derivation by `u` of the normal vector at `(u, v)`.
    fn normal_uder(&self, u: f64, v: f64) -> Vector3 {
        let uder = self.derivative_u(u, v);
        let vder = self.derivative_v(u, v);
        let uuder = self.derivative_uu(u, v);
        let uvder = self.derivative_uv(u, v);
        let cross = uder.cross(vder);
        let cross_uder = uuder.cross(vder) + uder.cross(uvder);
        let abs = cross.magnitude();
        let abs_uder = cross.dot(cross_uder) / abs;
        (cross_uder * abs - cross * abs_uder) / (abs * abs)
    }
    /// Returns the derivation by `u` of the normal vector at `(u, v)`.
    fn normal_vder(&self, u: f64, v: f64) -> Vector3 {
        let uder = self.derivative_u(u, v);
        let vder = self.derivative_v(u, v);
        let uvder = self.derivative_uv(u, v);
        let vvder = self.derivative_vv(u, v);
        let cross = uder.cross(vder);
        let cross_vder = uvder.cross(vder) + uder.cross(vvder);
        let abs = cross.magnitude();
        let abs_vder = cross.dot(cross_vder) / abs;
        (cross_vder * abs - cross * abs_vder) / (abs * abs)
    }
}

impl<S: ParametricSurface3D> ParametricSurface3D for &S {
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

impl<S: BoundedSurface> BoundedSurface for &S {}

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
    /// `tol` must be greater than or equal to `TOLERANCE`.
    fn parameter_division(&self, range: ((f64, f64), (f64, f64)), tol: f64)
        -> (Vec<f64>, Vec<f64>);
}

impl<S: ParameterDivision2D> ParameterDivision2D for &S {
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
