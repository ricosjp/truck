use super::*;

/// Divides the domain into equal parts, examines all the values, and returns `(u, v)` such that `surface.subs(u, v)` is closest to `point`.
/// This method is useful to get an efficient hint of `search_nearest_parameter`.
pub fn presearch<S>(surface: &S, point: S::Point, (urange, vrange): ((f64, f64), (f64, f64)), division: usize) -> (f64, f64)
where
    S: ParametricSurface,
    S::Point: MetricSpace<Metric = f64> + Copy, {
    let mut res = (0.0, 0.0);
    let mut min = std::f64::INFINITY;
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
    (u0, v0): (f64, f64),
    trials: usize,
) -> Option<(f64, f64)>
where
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64, Diff = S::Vector>,
    S::Vector: InnerSpace<Scalar = f64> + Tolerance,
{
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
    let dermag2 = f64::min(1.0, ud.magnitude2());
    let dermag2 = f64::min(dermag2, vd.magnitude2());
    if f.magnitude2() < TOLERANCE2 * dermag2 || fprime.determinant().so_small() {
        Some((u0, v0))
    } else if trials == 0 {
        None
    } else {
        let vec = Vector2::new(u0, v0) - fprime.invert().unwrap() * f;
        search_nearest_parameter(surface, point, (vec[0], vec[1]), trials - 1)
    }
}

/// Searches the parameter by Newton's method.
#[inline(always)]
pub fn search_parameter2d<S: ParametricSurface<Point = Point2, Vector = Vector2>>(
    surface: &S,
    point: Point2,
    (u0, v0): (f64, f64),
    trials: usize,
) -> Option<(f64, f64)> {
    let pt = surface.subs(u0, v0);
    let uder = surface.uder(u0, v0);
    let vder = surface.vder(u0, v0);
    let dermag2 = f64::min(0.05, uder.magnitude2());
    let dermag2 = f64::min(dermag2, vder.magnitude2());
    if pt.distance2(point) < TOLERANCE2 * dermag2 {
        return Some((u0, v0));
    } else if trials == 0 {
        return None;
    }
    let hint = Vector2::new(u0, v0);
    let jacobi = Matrix2::from_cols(uder, vder);
    let res = jacobi.invert().map(move |inv| hint - inv * (pt - point));
    match res {
        Some(vec) => search_parameter2d(surface, point, (vec[0], vec[1]), trials - 1),
        None => None,
    }
}

#[derive(Clone, Debug)]
struct ProjectedSurface<'a, S> {
    surface: &'a S,
    origin: Point3,
    u_axis: Vector3,
    v_axis: Vector3,
}

impl<'a, S: ParametricSurface3D> ProjectedSurface<'a, S> {
    fn new(surface: &'a S, (u, v): (f64, f64)) -> Self {
        let origin = surface.subs(u, v);
        let normal = surface.normal(u, v);
        let tmp = if normal[0].abs() > normal[1].abs() {
            0
        } else {
            1
        };
        let max = if normal[tmp].abs() > normal[2].abs() {
            tmp
        } else {
            2
        };
        let mut u_axis = Vector3::zero();
        u_axis[max] = -normal[(max + 1) % 3];
        u_axis[(max + 1) % 3] = normal[max];
        let mut v_axis = Vector3::zero();
        v_axis[max] = -normal[(max + 2) % 3];
        v_axis[(max + 2) % 3] = normal[max];
        ProjectedSurface {
            surface,
            origin,
            u_axis,
            v_axis,
        }
    }
    #[inline(always)]
    fn vector_proj(&self, vector: Vector3) -> Vector2 {
        Vector2::new(self.u_axis.dot(vector), self.v_axis.dot(vector))
    }
    #[inline(always)]
    fn point_proj(&self, point: Point3) -> Point2 {
        Point2::from_vec(self.vector_proj(point - self.origin))
    }
}

impl<'a, S: ParametricSurface3D> ParametricSurface
    for ProjectedSurface<'a, S>
{
    type Point = Point2;
    type Vector = Vector2;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point2 { self.point_proj(self.surface.subs(u, v)) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector2 { self.vector_proj(self.surface.uder(u, v)) }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector2 { self.vector_proj(self.surface.vder(u, v)) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector2 { self.vector_proj(self.surface.uuder(u, v)) }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector2 { self.vector_proj(self.surface.uvder(u, v)) }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector2 { self.vector_proj(self.surface.vvder(u, v)) }
}

/// Searches the parameter by Newton's method.
#[inline(always)]
pub fn search_parameter3d<S: ParametricSurface3D>(
    surface: &S,
    point: Point3,
    (u0, v0): (f64, f64),
    trials: usize,
) -> Option<(f64, f64)> {
    let proj = ProjectedSurface::new(surface, (u0, v0));
    search_parameter2d(&proj, proj.point_proj(point), (u0, v0), trials).and_then(|(u, v)| {
        match surface.subs(u, v).near(&point) {
            true => Some((u, v)),
            false => {
                println!("not near!");
                None
            },
        }
    })
}

/// Creates the surface division
#[inline(always)]
pub fn parameter_division<S>(
    surface: &S,
    (urange, vrange): ((f64, f64), (f64, f64)),
    tol: f64,
) -> (Vec<f64>, Vec<f64>)
where
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>,
{
    let (mut udiv, mut vdiv) = (vec![urange.0, urange.1], vec![vrange.0, vrange.1]);
    sub_parameter_division(surface, (&mut udiv, &mut vdiv), tol);
    (udiv, vdiv)
}

fn sub_parameter_division<S>(surface: &S, (udiv, vdiv): (&mut Vec<f64>, &mut Vec<f64>), tol: f64)
where
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>, {
    let mut divide_flag0 = vec![false; udiv.len() - 1];
    let mut divide_flag1 = vec![false; vdiv.len() - 1];

    for (u, ub) in udiv.windows(2).zip(&mut divide_flag0) {
        for (v, vb) in vdiv.windows(2).zip(&mut divide_flag1) {
            let p = 0.5 + (0.2 * rand::random::<f64>() - 0.1);
            let q = 0.5 + (0.2 * rand::random::<f64>() - 0.1);
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
            let u = u[0] * (1.0 - p) + u[1] * p;
            let v = v[0] * (1.0 - q) + v[1] * q;
            let far = surface.subs(u, v).distance(pt) > tol;
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
