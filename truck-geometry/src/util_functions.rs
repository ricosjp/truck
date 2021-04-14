use crate::*;

#[doc(hidden)]
#[inline(always)]
pub fn inv_or_zero(delta: f64) -> f64 {
    if delta.so_small() {
        0.0
    } else {
        1.0 / delta
    }
}

#[doc(hidden)]
pub fn presearch<S>(surface: &S, point: S::Point) -> (f64, f64)
where
    S: BoundedSurface,
    S::Point: MetricSpace<Metric = f64> + Copy, {
    const N: usize = 50;
    let mut res = (0.0, 0.0);
    let mut min = std::f64::INFINITY;
    for i in 0..=N {
        for j in 0..=N {
            let p = i as f64 / N as f64;
            let q = j as f64 / N as f64;
            let ((u0, u1), (v0, v1)) = surface.parameter_range();
            let u = u0 * (1.0 - p) + u1 * p;
            let v = v0 * (1.0 - q) + v1 * q;
            let dist = surface.subs(u, v).distance2(point);
            if dist < min {
                min = dist;
                res = (u, v);
            }
        }
    }
    res
}

/// Search the nearest parameter
#[doc(hidden)]
pub fn curve_search_nearest_parameter<C: ParametricCurve>(
    curve: &C,
    point: C::Point,
    hint: f64,
    trials: usize,
) -> Option<f64>
where
    C::Point: EuclideanSpace<Scalar = f64, Diff = C::Vector>,
    C::Vector: InnerSpace<Scalar = f64> + Tolerance,
{
    let pt = curve.subs(hint);
    let der = curve.der(hint);
    let der2 = curve.der2(hint);
    let f = der.dot(pt - point);
    let fprime = der2.dot(pt - point) + der.magnitude2();
    if fprime.so_small() {
        return Some(hint);
    }
    let t = hint - f / fprime;
    if t.near(&hint) {
        Some(t)
    } else if trials == 0 {
        None
    } else {
        curve_search_nearest_parameter(curve, point, t, trials - 1)
    }
}

#[doc(hidden)]
pub fn surface_search_nearest_parameter<S: ParametricSurface>(
    surface: &S,
    point: S::Point,
    (u0, v0): (f64, f64),
    trials: usize,
) -> Option<(f64, f64)>
where
    S::Point: EuclideanSpace<Scalar = f64, Diff = S::Vector>,
    S::Vector: InnerSpace<Scalar = f64> + Tolerance,
{
    let s = surface.subs(u0, v0);
    let ud = surface.uder(u0, v0);
    let vd = surface.vder(u0, v0);
    let uud = surface.uuder(u0, v0);
    let uvd = surface.uvder(u0, v0);
    let vvd = surface.vvder(u0, v0);
    let f_u = ud.dot(s - point);
    let f_v = vd.dot(s - point);
    let a = uud.dot(s - point) + ud.dot(ud);
    let c = uvd.dot(s - point) + ud.dot(vd);
    let b = vvd.dot(s - point) + vd.dot(vd);
    let det = a * b - c * c;
    if det.so_small() {
        return Some((u0, v0));
    }
    let u = u0 - (b * f_u - c * f_v) / det;
    let v = v0 - (-c * f_u + a * f_v) / det;
    if u.near2(&u0) && v.near2(&v0) {
        Some((u, v))
    } else if trials == 0 {
        None
    } else {
        surface_search_nearest_parameter(surface, point, (u, v), trials - 1)
    }
}
