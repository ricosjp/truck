use newton::Jacobian;

use super::*;

/// Divides the domain into equal parts, examines all the values, and returns `(u, v)` such that `surface.subs(u, v)` is closest to `point`.
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
            let dist = surface.subs(u, v).distance2(point);
            if dist < min {
                min = dist;
                res = (u, v);
            }
        }
    }
    res
}

/// Vectors whose points returned by the surface that can be the target of [`search_nearest_parameter`].
pub trait SsnpVector: InnerSpace<Scalar = f64> + Tolerance {
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

impl SsnpVector for Vector2 {
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
            value: surface.subs(u, v) - point,
            derivation: Matrix2::from_cols(surface.uder(u, v), surface.vder(u, v)),
        }
    }
    fn into_param(self) -> (f64, f64) { self.into() }
    fn from_param(param: (f64, f64)) -> Self { param.into() }
}

impl SsnpVector for Vector3 {
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
        let diff = surface.subs(u, v) - point;
        let uder = surface.uder(u, v);
        let vder = surface.vder(u, v);
        let uuder = surface.uuder(u, v);
        let uvder = surface.uvder(u, v);
        let vvder = surface.vvder(u, v);
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
    P::Diff: SsnpVector<Point = P>,
    S: ParametricSurface<Point = P, Vector = P::Diff>,
{
    let function = move |param: P::Diff| SsnpVector::subs(surface, point, param);
    let res = newton::solve(function, P::Diff::from_param(hint), trials);
    res.ok().map(P::Diff::into_param)
}

/// Vectors whose points returned by the surface that can be the target of [`search_parameter`].
pub trait SspVector: InnerSpace<Scalar = f64> + Tolerance {
    #[doc(hidden)]
    type Point;
    #[doc(hidden)]
    fn subs<S>(surface: &S, point: Self::Point, param: Vector2) -> CalcOutput<Vector2, Matrix2>
    where S: ParametricSurface<Point = Self::Point, Vector = Self>;
}

impl SspVector for Vector2 {
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
            value: surface.subs(u, v) - point,
            derivation: Matrix2::from_cols(surface.uder(u, v), surface.vder(u, v)),
        }
    }
}

impl SspVector for Vector3 {
    type Point = Point3;
    fn subs<S>(
        surface: &S,
        point: Self::Point,
        Vector2 { x: u, y: v }: Vector2,
    ) -> CalcOutput<Vector2, Matrix2>
    where
        S: ParametricSurface<Point = Self::Point, Vector = Self>,
    {
        let diff = surface.subs(u, v) - point;
        let uder = surface.uder(u, v);
        let vder = surface.vder(u, v);
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
    P::Diff: SspVector<Point = P>,
    S: ParametricSurface<Point = P, Vector = P::Diff>,
{
    let function = move |param: Vector2| SspVector::subs(surface, point, param);
    let res = newton::solve(function, hint.into(), trials);
    res.ok().and_then(
        |Vector2 { x: u, y: v }| match surface.subs(u, v).near(&point) {
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
        value: surface.subs(x, y) - curve.subs(z),
        derivation: Matrix3::from_cols(surface.uder(x, y), surface.vder(x, y), -curve.der(z)),
    };
    let hint = Vector3::new(hint0.0, hint0.1, hint1);
    let Vector3 { x, y, z } = newton::solve(function, hint, trials).ok()?;
    match surface.subs(x, y).near(&curve.subs(z)) {
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
pub fn parameter_division<S, T: TesselationSplitMethod>(
    surface: &S,
    (urange, vrange): ((f64, f64), (f64, f64)),
    split: T,
) -> (Vec<f64>, Vec<f64>)
where
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
{
    let (mut udiv, mut vdiv) = (vec![urange.0, urange.1], vec![vrange.0, vrange.1]);
    sub_parameter_division(surface, (&mut udiv, &mut vdiv), split);
    (udiv, vdiv)
}

fn sub_parameter_division<S, T: TesselationSplitMethod>(
    surface: &S,
    (udiv, vdiv): (&mut Vec<f64>, &mut Vec<f64>),
    split: T,
) where
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
{
    let mut divide_flag0 = vec![false; udiv.len() - 1];
    let mut divide_flag1 = vec![false; vdiv.len() - 1];

    for (u, ub) in udiv.windows(2).zip(&mut divide_flag0) {
        for (v, vb) in vdiv.windows(2).zip(&mut divide_flag1) {
            if *ub && *vb {
                continue;
            }
            let (split_u, split_v) = split.split_surface(surface, (u[0], u[1]), (v[0], v[1]));
            *ub = *ub || split_u;
            *vb = *vb || split_v;
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
        sub_parameter_division(surface, (udiv, vdiv), split);
    }
}
