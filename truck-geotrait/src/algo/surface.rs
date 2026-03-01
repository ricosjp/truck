use newton::Jacobian;

use super::*;

/// Divides the domain into equal parts, examines all the values, and returns `(u, v)` such that
/// `surface.evaluate(u, v)` is closest to `point`.
/// This method is useful to get an efficient hint of `search_nearest_parameter`.
pub fn presearch<S>(
    surface: &S,
    point: S::Point,
    (urange, vrange): ((f64, f64), (f64, f64)),
    division: usize,
) -> (f64, f64)
where
    S: ParametricSurface,
    S::Point: MetricSpace<Metric = f64> + Copy,
{
    let mut res = (0.0, 0.0);
    let mut min = f64::INFINITY;
    let ((u0, u1), (v0, v1)) = (urange, vrange);
    for i in 0..=division {
        for j in 0..=division {
            let p = i as f64 / division as f64;
            let q = j as f64 / division as f64;
            let u = u0 * (1.0 - p) + u1 * p;
            let v = v0 * (1.0 - q) + v1 * q;
            let dist = surface.evaluate(u, v).distance2(point);
            if dist < min {
                min = dist;
                res = (u, v);
            }
        }
    }
    res
}

/// Vectors whose points returned by the surface that can be the target of [`search_nearest_parameter`].
pub trait SearchNearestParameterVector: InnerSpace<Scalar = f64> + Tolerance {
    #[doc(hidden)]
    type Point;
    #[doc(hidden)]
    type Matrix: Jacobian<Self>;
    #[doc(hidden)]
    fn subs<S>(surface: &S, point: Self::Point, param: Self) -> CalcOutput<Self, Self::Matrix>
    where S: ParametricSurface<Point = Self::Point, Vector = Self>;
    #[doc(hidden)]
    fn into_param(self) -> (f64, f64);
    #[doc(hidden)]
    fn from_param(param: (f64, f64)) -> Self;
}

impl SearchNearestParameterVector for Vector2 {
    type Point = Point2;
    type Matrix = Matrix2;
    fn subs<S>(
        surface: &S,
        point: Point2,
        Vector2 { x: u, y: v }: Vector2,
    ) -> CalcOutput<Self, Matrix2>
    where
        S: ParametricSurface<Point = Point2, Vector = Vector2>,
    {
        CalcOutput {
            value: surface.evaluate(u, v) - point,
            derivation: Matrix2::from_cols(surface.derivative_u(u, v), surface.derivative_v(u, v)),
        }
    }
    fn into_param(self) -> (f64, f64) { self.into() }
    fn from_param(param: (f64, f64)) -> Self { param.into() }
}

impl SearchNearestParameterVector for Vector3 {
    type Point = Point3;
    type Matrix = Matrix3;
    fn subs<S>(
        surface: &S,
        point: Self::Point,
        Vector3 { x: u, y: v, z: w }: Vector3,
    ) -> CalcOutput<Self, Self::Matrix>
    where
        S: ParametricSurface<Point = Self::Point, Vector = Self>,
    {
        let diff = surface.evaluate(u, v) - point;
        let uder = surface.derivative_u(u, v);
        let vder = surface.derivative_v(u, v);
        let uuder = surface.derivative_uu(u, v);
        let uvder = surface.derivative_uv(u, v);
        let vvder = surface.derivative_vv(u, v);
        let uv_cross = uder.cross(vder);
        CalcOutput {
            value: diff + uv_cross * w,
            derivation: Matrix3::from_cols(
                uder + (uuder.cross(vder) + uder.cross(uvder)) * w,
                vder + (uvder.cross(vder) + uder.cross(vvder)) * w,
                uv_cross,
            ),
        }
    }
    fn into_param(self) -> (f64, f64) { self.truncate().into() }
    fn from_param((u, v): (f64, f64)) -> Self { Self::new(u, v, 0.0) }
}

/// Searches the parameter by Newton's method.
#[inline(always)]
pub fn search_nearest_parameter<P, S>(
    surface: &S,
    point: P,
    hint: (f64, f64),
    trials: usize,
) -> Option<(f64, f64)>
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>,
    P::Diff: SearchNearestParameterVector<Point = P>,
    S: ParametricSurface<Point = P, Vector = P::Diff>,
{
    let function = move |param: P::Diff| SearchNearestParameterVector::subs(surface, point, param);
    let res = newton::solve(function, P::Diff::from_param(hint), trials);
    res.ok().map(P::Diff::into_param)
}

/// Vectors whose points returned by the surface that can be the target of [`search_parameter`].
pub trait SearchParameterVector: InnerSpace<Scalar = f64> + Tolerance {
    #[doc(hidden)]
    type Point;
    #[doc(hidden)]
    fn subs<S>(surface: &S, point: Self::Point, param: Vector2) -> CalcOutput<Vector2, Matrix2>
    where S: ParametricSurface<Point = Self::Point, Vector = Self>;
}

impl SearchParameterVector for Vector2 {
    type Point = Point2;
    fn subs<S>(
        surface: &S,
        point: Point2,
        Vector2 { x: u, y: v }: Vector2,
    ) -> CalcOutput<Vector2, Matrix2>
    where
        S: ParametricSurface<Point = Point2, Vector = Vector2>,
    {
        CalcOutput {
            value: surface.evaluate(u, v) - point,
            derivation: Matrix2::from_cols(surface.derivative_u(u, v), surface.derivative_v(u, v)),
        }
    }
}

impl SearchParameterVector for Vector3 {
    type Point = Point3;
    fn subs<S>(
        surface: &S,
        point: Self::Point,
        Vector2 { x: u, y: v }: Vector2,
    ) -> CalcOutput<Vector2, Matrix2>
    where
        S: ParametricSurface<Point = Self::Point, Vector = Self>,
    {
        let diff = surface.evaluate(u, v) - point;
        let uder = surface.derivative_u(u, v);
        let vder = surface.derivative_v(u, v);
        CalcOutput {
            value: Vector2::new(uder.dot(diff), vder.dot(diff)),
            derivation: Matrix2::new(
                uder.dot(uder),
                uder.dot(vder),
                uder.dot(vder),
                vder.dot(vder),
            ),
        }
    }
}

/// Searches the parameter by Newton's method.
#[inline(always)]
pub fn search_parameter<P, S>(
    surface: &S,
    point: P,
    hint: (f64, f64),
    trials: usize,
) -> Option<(f64, f64)>
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + Tolerance,
    P::Diff: SearchParameterVector<Point = P>,
    S: ParametricSurface<Point = P, Vector = P::Diff>,
{
    let function = move |param: Vector2| SearchParameterVector::subs(surface, point, param);
    let res = newton::solve(function, hint.into(), trials);
    res.ok().and_then(
        |Vector2 { x: u, y: v }| match surface.evaluate(u, v).near(&point) {
            true => Some((u, v)),
            false => None,
        },
    )
}

/// Searches the parameters of the intersection point of `surface` and `curve`.
pub fn search_intersection_parameter<C, S>(
    surface: &S,
    hint0: (f64, f64),
    curve: &C,
    hint1: f64,
    trials: usize,
) -> Option<((f64, f64), f64)>
where
    C: ParametricCurve3D,
    S: ParametricSurface3D,
{
    let function = move |Vector3 { x, y, z }| CalcOutput {
        value: surface.evaluate(x, y) - curve.evaluate(z),
        derivation: Matrix3::from_cols(
            surface.derivative_u(x, y),
            surface.derivative_v(x, y),
            -curve.derivative(z),
        ),
    };
    let hint = Vector3::new(hint0.0, hint0.1, hint1);
    let Vector3 { x, y, z } = newton::solve(function, hint, trials).ok()?;
    match surface.evaluate(x, y).near(&curve.evaluate(z)) {
        true => Some(((x, y), z)),
        false => None,
    }
}

/// Creates the surface division
///
/// # Panics
///
/// `tol` must be more than `TOLERANCE`.
#[inline(always)]
pub fn parameter_division<S>(
    surface: &S,
    (urange, vrange): ((f64, f64), (f64, f64)),
    tol: f64,
) -> (Vec<f64>, Vec<f64>)
where
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
{
    nonpositive_tolerance!(tol);
    let (mut udiv, mut vdiv) = (vec![urange.0, urange.1], vec![vrange.0, vrange.1]);
    sub_parameter_division(surface, (&mut udiv, &mut vdiv), tol);
    (udiv, vdiv)
}

fn sub_parameter_division<S>(surface: &S, (udiv, vdiv): (&mut Vec<f64>, &mut Vec<f64>), tol: f64)
where
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>, {
    let mut divide_flag0 = vec![false; udiv.len() - 1];
    let mut divide_flag1 = vec![false; vdiv.len() - 1];

    for (u, ub) in udiv.windows(2).zip(&mut divide_flag0) {
        for (v, vb) in vdiv.windows(2).zip(&mut divide_flag1) {
            if *ub && *vb {
                continue;
            }
            let (u_gen, v_gen) = ((u[0] + u[1]) / 2.0, (v[0] + v[1]) / 2.0);
            let generated = surface.evaluate(u_gen, v_gen);
            let p = 0.5 + (0.2 * HashGen::hash1(generated) - 0.1);
            let q = 0.5 + (0.2 * HashGen::hash1(generated) - 0.1);
            let u0 = u[0] * (1.0 - p) + u[1] * p;
            let v0 = v[0] * (1.0 - q) + v[1] * q;
            let p0 = surface.evaluate(u0, v0);
            let pt00 = surface.evaluate(u[0], v[0]);
            let pt01 = surface.evaluate(u[0], v[1]);
            let pt10 = surface.evaluate(u[1], v[0]);
            let pt11 = surface.evaluate(u[1], v[1]);
            let pt = S::Point::from_vec(
                pt00.to_vec() * (1.0 - p) * (1.0 - q)
                    + pt01.to_vec() * (1.0 - p) * q
                    + pt10.to_vec() * p * (1.0 - q)
                    + pt11.to_vec() * p * q,
            );
            if p0.distance2(pt) > tol * tol {
                let delu = pt00.midpoint(pt01).distance(p0) + pt10.midpoint(pt11).distance(p0);
                let delv = pt00.midpoint(pt10).distance(p0) + pt01.midpoint(pt11).distance(p0);
                if delu > delv * 2.0 {
                    *ub = true;
                } else if delv > delu * 2.0 {
                    *vb = true;
                } else {
                    (*ub, *vb) = (true, true);
                }
            }
        }
    }

    let mut new_udiv = vec![udiv[0]];
    for (u, ub) in udiv.windows(2).zip(divide_flag0) {
        if ub {
            new_udiv.push((u[0] + u[1]) / 2.0);
        }
        new_udiv.push(u[1]);
    }

    let mut new_vdiv = vec![vdiv[0]];
    for (v, vb) in vdiv.windows(2).zip(divide_flag1) {
        if vb {
            new_vdiv.push((v[0] + v[1]) / 2.0);
        }
        new_vdiv.push(v[1]);
    }

    if udiv.len() != new_udiv.len() || vdiv.len() != new_vdiv.len() {
        *udiv = new_udiv;
        *vdiv = new_vdiv;
        sub_parameter_division(surface, (udiv, vdiv), tol);
    }
}
