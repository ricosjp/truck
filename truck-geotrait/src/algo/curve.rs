use super::*;
use surface::{SsnpVector, SspVector};

/// Divides the domain into equal parts, examines all the values, and returns `t` such that `curve.subs(t)` is closest to `point`.
/// This method is useful to get an efficient hint of [`search_nearest_parameter`].
pub fn presearch<C>(curve: &C, point: C::Point, range: (f64, f64), division: usize) -> f64
where
    C: ParametricCurve,
    C::Point: MetricSpace<Metric = f64> + Copy, {
    let (t0, t1) = range;
    let mut res = t0;
    let mut min = f64::INFINITY;
    for i in 0..=division {
        let p = i as f64 / division as f64;
        let t = t0 * (1.0 - p) + t1 * p;
        let dist = curve.subs(t).distance2(point);
        if dist < min {
            min = dist;
            res = t;
        }
    }
    res
}

/// Searches the nearest parameter by Newton's method.
pub fn search_nearest_parameter<C>(
    curve: &C,
    point: C::Point,
    hint: f64,
    trials: usize,
) -> Option<f64>
where
    C: ParametricCurve,
    C::Point: EuclideanSpace<Scalar = f64, Diff = C::Vector>,
    C::Vector: InnerSpace<Scalar = f64> + Tolerance,
{
    let function = move |t: f64| {
        let diff = curve.subs(t) - point;
        let der = curve.der(t);
        let der2 = curve.der2(t);
        CalcOutput {
            value: der.dot(diff),
            derivation: der2.dot(diff) + der.magnitude2(),
        }
    };
    newton::solve(function, hint, trials).ok()
}

/// Searches the parameter by Newton's method.
pub fn search_parameter<C>(curve: &C, point: C::Point, hint: f64, trials: usize) -> Option<f64>
where
    C: ParametricCurve,
    C::Point: EuclideanSpace<Scalar = f64, Diff = C::Vector>,
    C::Vector: InnerSpace<Scalar = f64> + Tolerance, {
    let function = move |t: f64| {
        let diff = curve.subs(t) - point;
        let der = curve.der(t);
        CalcOutput {
            value: der.dot(diff),
            derivation: der.magnitude2(),
        }
    };
    newton::solve(function, hint, trials).ok().and_then(|t| {
        match curve.subs(t).to_vec().near(&point.to_vec()) {
            true => Some(t),
            false => None,
        }
    })
}

/// Creates the curve division
///
/// # Panics
///
/// `tol` must be greater than or equal to `TOLERANCE`.
pub fn parameter_division<C, S: TesselationSplitMethod>(
    curve: &C,
    range: (f64, f64),
    split: S,
) -> (Vec<f64>, Vec<C::Point>)
where
    C: ParametricCurve,
    C::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
{
    sub_parameter_division(curve, range, split, 100)
}

fn sub_parameter_division<C, T: TesselationSplitMethod>(
    curve: &C,
    range: (f64, f64),
    split: T,
    trials: usize,
) -> (Vec<f64>, Vec<C::Point>)
where
    C: ParametricCurve,
    C::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
{
    if trials != 0 && split.split_curve(curve, range) {
        let mid_param = (range.0 + range.1) / 2.0;
        let (mut params, mut pts) =
            sub_parameter_division(curve, (range.0, mid_param), split, trials - 1);
        let _ = (params.pop(), pts.pop());
        let (new_params, new_pts) =
            sub_parameter_division(curve, (mid_param, range.1), split, trials - 1);
        params.extend(new_params);
        pts.extend(new_pts);
        (params, pts)
    } else {
        (
            vec![range.0, range.1],
            vec![curve.subs(range.0), curve.subs(range.1)],
        )
    }
}

#[derive(Clone, Debug)]
struct SubSurface<C0, C1> {
    curve0: C0,
    curve1: C1,
}

impl<P, C0, C1> ParametricSurface for SubSurface<C0, C1>
where
    P: EuclideanSpace<Scalar = f64>,
    P::Diff: VectorSpace<Scalar = f64>,
    C0: ParametricCurve<Point = P, Vector = P::Diff>,
    C1: ParametricCurve<Point = P, Vector = P::Diff>,
{
    type Point = P;
    type Vector = P::Diff;
    #[inline(always)]
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        match (m, n) {
            (0, 0) => self.curve0.subs(u) - self.curve1.subs(v),
            (_, 0) => self.curve0.der_n(m, u),
            (0, _) => self.curve1.der_n(n, v) * (-1.0),
            _ => Self::Vector::zero(),
        }
    }
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point { P::from_vec(self.der_mn(0, 0, u, v)) }
    #[inline(always)]
    fn uder(&self, u: f64, _: f64) -> Self::Vector { self.curve0.der(u) }
    #[inline(always)]
    fn vder(&self, _: f64, v: f64) -> Self::Vector { self.curve1.der(v) * (-1.0) }
    #[inline(always)]
    fn uuder(&self, u: f64, _: f64) -> Self::Vector { self.curve0.der2(u) }
    #[inline(always)]
    fn vvder(&self, _: f64, v: f64) -> Self::Vector { self.curve1.der2(v) * (-1.0) }
    #[inline(always)]
    fn uvder(&self, _: f64, _: f64) -> Self::Vector { P::Diff::zero() }
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        (self.curve0.parameter_range(), self.curve1.parameter_range())
    }
    #[inline(always)]
    fn u_period(&self) -> Option<f64> { self.curve0.period() }
    #[inline(always)]
    fn v_period(&self) -> Option<f64> { self.curve1.period() }
}

impl<C0, C1> ParametricSurface3D for SubSurface<C0, C1>
where
    C0: ParametricCurve3D,
    C1: ParametricCurve3D,
{
}

/// Divides the domain into equal parts, examines all the values, and returns `(t0, t1)` such that `curve0.subs(t0)` is closest to `curve1.subs(t1`.
/// This method is useful to get an efficient hint of [`search_closest_parameter`].
#[inline(always)]
pub fn presearch_closest_point<P, C0, C1>(
    curve0: &C0,
    curve1: &C1,
    ranges: ((f64, f64), (f64, f64)),
    division: usize,
) -> (f64, f64)
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + Copy,
    P::Diff: VectorSpace<Scalar = f64>,
    C0: ParametricCurve<Point = P, Vector = P::Diff>,
    C1: ParametricCurve<Point = P, Vector = P::Diff>,
{
    surface::presearch(
        &SubSurface { curve0, curve1 },
        P::origin(),
        ranges,
        division,
    )
}

/// Searches the parameters of the closest point of the two curves.
#[inline(always)]
pub fn search_closest_parameter<P, C0, C1>(
    curve0: &C0,
    curve1: &C1,
    hint: (f64, f64),
    trials: usize,
) -> Option<(f64, f64)>
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>,
    P::Diff: SsnpVector<Point = P> + Tolerance,
    C0: ParametricCurve<Point = P, Vector = P::Diff>,
    C1: ParametricCurve<Point = P, Vector = P::Diff>,
{
    surface::search_nearest_parameter(&SubSurface { curve0, curve1 }, P::origin(), hint, trials)
}

/// Searches the parameters of the intersection point of the two curves.
#[inline(always)]
pub fn search_intersection_parameter<P, C0, C1>(
    curve0: &C0,
    curve1: &C1,
    hint: (f64, f64),
    trials: usize,
) -> Option<(f64, f64)>
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + Tolerance,
    P::Diff: SspVector<Point = P> + Tolerance,
    C0: ParametricCurve<Point = P, Vector = P::Diff>,
    C1: ParametricCurve<Point = P, Vector = P::Diff>,
{
    surface::search_parameter(&SubSurface { curve0, curve1 }, P::origin(), hint, trials)
}
