use std::ops::Bound;
use truck_base::cgmath64::*;
use truck_geotrait::*;

// polynomial curve
#[derive(Clone, Debug)]
pub struct PolyCurve<P: EuclideanSpace<Scalar = f64>>(pub Vec<P::Diff>);

impl<P: EuclideanSpace<Scalar = f64>> ParametricCurve for PolyCurve<P> {
    type Point = P;
    type Vector = P::Diff;
    fn der_n(&self, t: f64, n: usize) -> P::Diff {
        let iter = self.0.iter().enumerate().skip(n);
        let closure = |(s, res): (f64, P::Diff), (deg, a): (usize, &P::Diff)| {
            let p = (0..n).fold(1, |p, i| p * (deg - i));
            (s * t, res + *a * s * p as f64)
        };
        iter.fold((1.0, P::Diff::zero()), closure).1
    }
    #[inline(always)]
    fn subs(&self, t: f64) -> P { P::from_vec(self.der_n(t, 0)) }
    #[inline(always)]
    fn der(&self, t: f64) -> P::Diff { self.der_n(t, 1) }
    #[inline(always)]
    fn der2(&self, t: f64) -> P::Diff { self.der_n(t, 2) }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange {
        (Bound::Included(-100.0), Bound::Included(100.0))
    }
}

impl<P: EuclideanSpace<Scalar = f64>> BoundedCurve for PolyCurve<P> {}

impl<P> ParameterDivision1D for PolyCurve<P>
where P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + truck_base::hash::HashGen<f64>
{
    type Point = P;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        algo::curve::parameter_division(self, range, tol)
    }
}

// surface by tensor product of polynomials e.g. `(2u^2 + 3u + 1)(4v^2 - 6v + 2)`
#[derive(Clone, Debug)]
pub struct PolySurface(pub PolyCurve<Point3>, pub PolyCurve<Point3>);

impl ParametricSurface for PolySurface {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn der_mn(&self, u: f64, v: f64, m: usize, n: usize) -> Self::Vector {
        self.0.der_n(u, m).mul_element_wise(self.1.der_n(v, n))
    }
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 { Point3::from_vec(self.der_mn(u, v, 0, 0)) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 { self.der_mn(u, v, 1, 0) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 { self.der_mn(u, v, 0, 1) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector3 { self.der_mn(u, v, 2, 0) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector3 { self.der_mn(u, v, 1, 1) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector3 { self.der_mn(u, v, 0, 2) }
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        (
            (Bound::Included(-100.0), Bound::Included(100.0)),
            (Bound::Included(-50.0), Bound::Included(50.0)),
        )
    }
}

impl ParametricSurface3D for PolySurface {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        self.uder(u, v).cross(self.vder(u, v)).normalize()
    }
}

impl BoundedSurface for PolySurface {}

impl ParameterDivision2D for PolySurface {
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        algo::surface::parameter_division(self, range, tol)
    }
}
