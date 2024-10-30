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

/// Searches the nearest parameter by Newton's method.
pub fn search_nearest_parameter<S>(
    surface: &S,
    point: S::Point,
    hint: (f64, f64),
    trials: usize,
) -> Option<(f64, f64)>
where
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64, Diff = S::Vector>,
    S::Vector: InnerSpace<Scalar = f64> + Tolerance,
{
    let mut log = NewtonLog::new(trials);
    let mut vec: Vector2 = hint.into();
    for _ in 0..=trials {
        log.push(vec);
        let Vector2 { x: u0, y: v0 } = vec;
        let s = surface.subs(u0, v0);
        let ud = surface.uder(u0, v0);
        let vd = surface.vder(u0, v0);
        let uud = surface.uuder(u0, v0);
        let uvd = surface.uvder(u0, v0);
        let vvd = surface.vvder(u0, v0);
        let f = Vector2::new(ud.dot(s - point), vd.dot(s - point));
        let a = uud.dot(s - point) + ud.dot(ud);
        let c = uvd.dot(s - point) + ud.dot(vd);
        let b = vvd.dot(s - point) + vd.dot(vd);
        let fprime = Matrix2::new(a, c, c, b);
        let w = vec - fprime.invert()? * f;
        if w.near2(&vec) {
            return Some(vec.into());
        }
        vec = w;
    }
    log.print_error();
    None
}

/// Vectors whose points returned by the surface that can be the target of [`search_parameter`].
pub trait SspVector: InnerSpace<Scalar = f64> + Tolerance {
    #[doc(hidden)]
    fn advance_newton<S>(self, surface: &S, diff: Self) -> Option<Self>
    where S: ParametricSurface<Vector = Self>;
    #[doc(hidden)]
    fn into_param(self) -> (f64, f64);
    #[doc(hidden)]
    fn from_param(param: (f64, f64)) -> Self;
}

impl SspVector for Vector2 {
    fn advance_newton<S>(self, surface: &S, diff: Self) -> Option<Self>
    where S: ParametricSurface<Vector = Self> {
        let Vector2 { x: u, y: v } = self;
        let uder = surface.uder(u, v);
        let vder = surface.vder(u, v);
        Some(self - Matrix2::from_cols(uder, vder).invert()? * diff)
    }
    fn into_param(self) -> (f64, f64) { self.into() }
    fn from_param(param: (f64, f64)) -> Self { param.into() }
}

impl SspVector for Vector3 {
    fn advance_newton<S>(self, surface: &S, diff: Self) -> Option<Self>
    where S: ParametricSurface<Vector = Self> {
        let Vector3 { x: u, y: v, z: w } = self;
        let uder = surface.uder(u, v);
        let vder = surface.vder(u, v);
        let uuder = surface.uuder(u, v);
        let uvder = surface.uvder(u, v);
        let vvder = surface.vvder(u, v);
        let uv_cross = uder.cross(vder);
        let b = diff + uv_cross * w;
        let b_uder = uder + (uuder.cross(vder) + uder.cross(uvder)) * w;
        let b_vder = vder + (uvder.cross(vder) + uder.cross(vvder)) * w;
        let b_wder = uv_cross;
        Some(self - Matrix3::from_cols(b_uder, b_vder, b_wder).invert()? * b)
    }
    fn into_param(self) -> (f64, f64) { self.truncate().into() }
    fn from_param((u, v): (f64, f64)) -> Self { Self::new(u, v, 0.0) }
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
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>,
    P::Diff: SspVector,
    S: ParametricSurface<Point = P, Vector = P::Diff>,
{
    let mut log = NewtonLog::new(trials);
    let mut vec = P::Diff::from_param(hint);
    for _ in 0..=trials {
        let (u0, v0) = vec.into_param();
        log.push(vec);
        let diff = surface.subs(u0, v0) - point;
        let w = vec.advance_newton(surface, diff)?;
        if vec.near2(&w) {
            return Some((u0, v0));
        }
        vec = w;
    }
    log.print_error();
    None
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
            let gen = surface.subs(u_gen, v_gen);
            let p = 0.5 + (0.2 * HashGen::hash1(gen) - 0.1);
            let q = 0.5 + (0.2 * HashGen::hash1(gen) - 0.1);
            let u0 = u[0] * (1.0 - p) + u[1] * p;
            let v0 = v[0] * (1.0 - q) + v[1] * q;
            let p0 = surface.subs(u0, v0);
            let pt00 = surface.subs(u[0], v[0]);
            let pt01 = surface.subs(u[0], v[1]);
            let pt10 = surface.subs(u[1], v[0]);
            let pt11 = surface.subs(u[1], v[1]);
            let pt = S::Point::from_vec(
                pt00.to_vec() * (1.0 - p) * (1.0 - q)
                    + pt01.to_vec() * (1.0 - p) * q
                    + pt10.to_vec() * p * (1.0 - q)
                    + pt11.to_vec() * p * q,
            );
            let far = p0.distance2(pt) > tol * tol;

            /*
            let ptu0 = surface.subs(u[0], v0);
            let ptu1 = surface.subs(u[1], v0);
            let ptv0 = surface.subs(u0, v[0]);
            let ptv1 = surface.subs(u0, v[1]);
            let ptu = ptu0 + (ptu1 - ptu0) * p;
            let ptv = ptv0 + (ptv1 - ptv0) * q;
            */

            *ub = *ub || far;
            *vb = *vb || far;
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
