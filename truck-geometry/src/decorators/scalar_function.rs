use super::*;

impl ScalarFunctionD1 for f64 {
    #[inline]
    fn der_n(&self, n: usize, _: f64) -> f64 {
        match n {
            0 => *self,
            _ => 0.0,
        }
    }
}

macro_rules! impl_radius_1dim {
    ($ty: ty) => {
        impl ScalarFunctionD1 for $ty {
            #[inline]
            fn der_n(&self, n: usize, t: f64) -> f64 { ParametricCurve::der_n(self, n, t).x }
        }
    };
}
impl_radius_1dim!(BSplineCurve<Vector1>);
impl_radius_1dim!(NurbsCurve<Vector2>);

impl<T: ScalarFunctionD1> ScalarFunctionD1 for &T {
    #[inline(always)]
    fn der_n(&self, n: usize, t: f64) -> f64 { (**self).der_n(n, t) }
}

impl<T: ScalarFunctionD2> ScalarFunctionD2 for &T {
    #[inline(always)]
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> f64 { (**self).der_mn(m, n, u, v) }
}

impl ScalarFunctionD2 for f64 {
    #[inline]
    fn der_mn(&self, m: usize, n: usize, _: f64, _: f64) -> f64 {
        match (m, n) {
            (0, 0) => *self,
            _ => 0.0,
        }
    }
}

macro_rules! impl_radius_2dim {
    ($ty: ty) => {
        impl ScalarFunctionD2 for $ty {
            #[inline]
            fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> f64 {
                ParametricSurface::der_mn(self, m, n, u, v).x
            }
        }
    };
}
impl_radius_2dim!(BSplineSurface<Vector1>);
impl_radius_2dim!(NurbsSurface<Vector2>);
