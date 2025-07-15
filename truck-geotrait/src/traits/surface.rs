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
    /// Returns $\partial^{m + n} S / \partial u^m \partial v^n$.
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector;
    /// Calculates higher degree derivations at the parameter `(u, v)` and assign them to `out`.
    fn ders<A: AsMut<[Self::Vector]>>(&self, u: f64, v: f64, out: &mut [A]) {
        out.iter_mut().enumerate().for_each(|(i, out)| {
            out.as_mut()
                .iter_mut()
                .enumerate()
                .for_each(|(j, out)| *out = self.der_mn(i, j, u, v))
        })
    }
    /// Returns derivations with order `(0..=m, 0..=n)` at the parameter `(u, v)`.
    fn ders_vec(&self, m: usize, n: usize, u: f64, v: f64) -> Vec<Vec<Self::Vector>> {
        (0..=m)
            .map(|i| (0..=n).map(|j| self.der_mn(i, j, u, v)).collect())
            .collect()
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
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        (*self).der_mn(m, n, u, v)
    }
    #[inline(always)]
    fn ders<A: AsMut<[Self::Vector]>>(&self, u: f64, v: f64, out: &mut [A]) {
        (*self).ders(u, v, out)
    }
    #[inline(always)]
    fn ders_vec(&self, m: usize, n: usize, u: f64, v: f64) -> Vec<Vec<Self::Vector>> {
        (*self).ders_vec(m, n, u, v)
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
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        (**self).der_mn(m, n, u, v)
    }
    #[inline(always)]
    fn ders<A: AsMut<[Self::Vector]>>(&self, u: f64, v: f64, out: &mut [A]) {
        (**self).ders(u, v, out)
    }
    #[inline(always)]
    fn ders_vec(&self, m: usize, n: usize, u: f64, v: f64) -> Vec<Vec<Self::Vector>> {
        (**self).ders_vec(m, n, u, v)
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
        self.uder(u, v).cross(self.vder(u, v)).normalize()
    }
    /// Returns the derivation by `u` of the normal vector at `(u, v)`.
    fn normal_uder(&self, u: f64, v: f64) -> Vector3 {
        let uder = self.uder(u, v);
        let vder = self.vder(u, v);
        let uuder = self.uuder(u, v);
        let uvder = self.uvder(u, v);
        let cross = uder.cross(vder);
        let cross_uder = uuder.cross(vder) + uder.cross(uvder);
        let abs = cross.magnitude();
        let abs_uder = cross.dot(cross_uder) / abs;
        (cross_uder * abs - cross * abs_uder) / (abs * abs)
    }
    /// Returns the derivation by `u` of the normal vector at `(u, v)`.
    fn normal_vder(&self, u: f64, v: f64) -> Vector3 {
        let uder = self.uder(u, v);
        let vder = self.vder(u, v);
        let uvder = self.uvder(u, v);
        let vvder = self.vvder(u, v);
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
    /// `tol` must be more than `TOLERANCE`.
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
    fn der_mn(&self, _: usize, _: usize, _: f64, _: f64) -> Self::Vector {}
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
