use truck_geotrait::algo::TesselationSplitMethod;

use super::*;
use std::f64::consts::PI;

impl Revolution {
    fn new(origin: Point3, axis: Vector3) -> Self {
        Self {
            origin,
            axis: axis.normalize(),
        }
    }
    #[inline(always)]
    fn rotation_matrix(self, v: f64) -> Matrix3 { Matrix3::from_axis_angle(self.axis, Rad(v)) }
    #[inline(always)]
    fn invert(&mut self) { self.axis *= -1.0; }
    #[inline(always)]
    fn inverse(mut self) -> Self {
        self.axis *= -1.0;
        self
    }
    #[inline(always)]
    fn contains(self, p: Point3) -> bool { (p - self.origin).cross(self.axis).so_small() }
    #[inline(always)]
    fn proj_point(&self, p: Point3) -> Point2 {
        let r = p - self.origin;
        let z = r.dot(self.axis);
        let h = r - z * self.axis;
        Point2::new(z, h.magnitude())
    }
    #[inline(always)]
    fn proj_vector(&self, p: Point3, v: Vector3) -> Vector2 {
        let r = self.proj_point(p).y;
        let vz = v.dot(self.axis);
        let vxy = v - vz * self.axis;
        let vq = (p - self.origin).dot(vxy) / r;
        Vector2::new(vz, vq)
    }
    #[inline(always)]
    fn proj_vector2(&self, p: Point3, v: Vector3, v2: Vector3) -> Vector2 {
        let r = self.proj_point(p).y;
        let v2z = v2.dot(self.axis);
        let v2xy = v2 - v2z * self.axis;
        let vz = v.dot(self.axis);
        let vxy = v - vz * self.axis;
        let a = (vxy.dot(vxy) + (p - self.origin).dot(v2xy)) / r;
        let b = f64::powi((p - self.origin).dot(vxy), 2) / (r * r * r);
        Vector2::new(v2z, a - b)
    }
    #[inline(always)]
    fn proj_angle(&self, p: Point3, q: Point3) -> f64 {
        let (p, q) = (p - self.origin, q - self.origin);
        let hp = (p - p.dot(self.axis) * self.axis).normalize();
        let hq = (q - q.dot(self.axis) * self.axis).normalize();
        let t = f64::acos(f64::clamp(hp.dot(hq), -1.0, 1.0));
        match hp.cross(hq).dot(self.axis) < 0.0 {
            false => t,
            true => 2.0 * PI - t,
        }
    }
}

impl<C: ParametricCurve3D> ParametricSurface for RevolutedCurve<C> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Vector3 {
        let center = match (m, n) {
            (0, 0) => self.origin().to_vec(),
            _ => Vector3::zero(),
        };
        let u_part = match m {
            0 => self.curve.subs(u) - self.origin(),
            _ => self.curve.der_n(m, u),
        };
        let v_part = from_axis_angle_derivation(n, self.axis(), Rad(v));
        v_part * u_part + center
    }
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 {
        let mat = self.revolution.rotation_matrix(v);
        let (p, o) = (self.curve.subs(u), self.origin());
        o + mat * (p - o)
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        self.revolution.rotation_matrix(v) * self.curve.der(u)
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        let u_part = self.curve.subs(u) - self.origin();
        let v_part = from_axis_angle_derivation(1, self.axis(), Rad(v));
        v_part * u_part
    }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector3 {
        self.revolution.rotation_matrix(v) * self.curve.der2(u)
    }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector3 {
        let u_part = self.curve.subs(u) - self.origin();
        let v_part = from_axis_angle_derivation(2, self.axis(), Rad(v));
        v_part * u_part
    }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector3 {
        let u_part = self.curve.der(u);
        let v_part = from_axis_angle_derivation(1, self.axis(), Rad(v));
        v_part * u_part
    }
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        (
            self.curve.parameter_range(),
            (Bound::Included(0.0), Bound::Excluded(2.0 * PI)),
        )
    }
    #[inline(always)]
    fn u_period(&self) -> Option<f64> { self.curve.period() }
    #[inline(always)]
    fn v_period(&self) -> Option<f64> { Some(2.0 * PI) }
}

impl<C: ParametricCurve3D + BoundedCurve> ParametricSurface3D for RevolutedCurve<C> {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        let (u0, u1) = self.curve.range_tuple();
        let (uder, vder) = if u.near(&u0) {
            let pt = self.curve.subs(u);
            let radius = self.axis().cross(pt - self.origin());
            if radius.so_small() {
                let uder = self.curve.der(u);
                (uder, self.axis().cross(uder))
            } else {
                (self.uder(u, v), self.vder(u, v))
            }
        } else if u.near(&u1) {
            let pt = self.curve.subs(u);
            let radius = self.axis().cross(pt - self.origin());
            if radius.so_small() {
                let uder = self.curve.der(u);
                (uder, uder.cross(self.axis()))
            } else {
                (self.uder(u, v), self.vder(u, v))
            }
        } else {
            (self.uder(u, v), self.vder(u, v))
        };
        uder.cross(vder).normalize()
    }
}

impl<C: ParametricCurve3D + BoundedCurve> BoundedSurface for RevolutedCurve<C> {}

impl<C: Clone> Invertible for RevolutedCurve<C> {
    #[inline(always)]
    fn invert(&mut self) { self.revolution.invert() }
    #[inline(always)]
    fn inverse(&self) -> Self {
        RevolutedCurve {
            curve: self.curve.clone(),
            revolution: self.revolution.inverse(),
        }
    }
}

#[derive(Clone, Debug)]
struct ProjectedCurve<C> {
    curve: C,
    revolution: Revolution,
}

impl<C: ParametricCurve3D> ParametricCurve for ProjectedCurve<C> {
    type Point = Point2;
    type Vector = Vector2;
    #[inline(always)]
    fn der_n(&self, n: usize, t: f64) -> Self::Vector {
        match n {
            0 => self.subs(t).to_vec(),
            1 => self.der(t),
            2 => self.der2(t),
            _ => unimplemented!(),
        }
    }
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point { self.revolution.proj_point(self.curve.subs(t)) }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector {
        self.revolution
            .proj_vector(self.curve.subs(t), self.curve.der(t))
    }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector {
        self.revolution
            .proj_vector2(self.curve.subs(t), self.curve.der(t), self.curve.der2(t))
    }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { self.curve.parameter_range() }
    #[inline(always)]
    fn period(&self) -> Option<f64> { self.curve.period() }
}

impl<C: ParametricCurve3D + BoundedCurve> BoundedCurve for ProjectedCurve<C> {}

impl<C: ParametricCurve3D + BoundedCurve> SearchParameter<D1> for ProjectedCurve<C> {
    type Point = Point2;
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let hint = match hint.into() {
            SPHint1D::Parameter(t) => t,
            SPHint1D::Range(x, y) => {
                algo::curve::presearch(self, point, (x, y), PRESEARCH_DIVISION)
            }
            SPHint1D::None => {
                algo::curve::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::curve::search_parameter(self, point, hint, trials)
    }
}

impl<C: ParametricCurve3D + BoundedCurve> SearchNearestParameter<D1> for ProjectedCurve<C> {
    type Point = Point2;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let hint = match hint.into() {
            SPHint1D::Parameter(t) => t,
            SPHint1D::Range(x, y) => {
                algo::curve::presearch(self, point, (x, y), PRESEARCH_DIVISION)
            }
            SPHint1D::None => {
                algo::curve::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::curve::search_nearest_parameter(self, point, hint, trials)
    }
}

impl<C> RevolutedCurve<C> {
    /// Creates a surface by revoluting a curve.
    #[inline(always)]
    pub fn by_revolution(curve: C, origin: Point3, axis: Vector3) -> Self {
        RevolutedCurve {
            curve,
            revolution: Revolution::new(origin, axis),
        }
    }
    /// Returns the curve before revoluted.
    #[inline(always)]
    pub const fn entity_curve(&self) -> &C { &self.curve }
    /// Into the curve before revoluted.
    #[inline(always)]
    pub fn into_entity_curve(self) -> C { self.curve }
    /// Returns origin of revolution
    #[inline(always)]
    pub const fn origin(&self) -> Point3 { self.revolution.origin }
    /// Returns axis of revolution
    #[inline(always)]
    pub const fn axis(&self) -> Vector3 { self.revolution.axis }
}

impl<C: ParametricCurve3D + BoundedCurve> RevolutedCurve<C> {
    /// Returns true if the front point of the curve is on the axis of rotation.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let line = BSplineCurve::new(
    ///     KnotVec::bezier_knot(1),
    ///     vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 1.0)],
    /// );
    /// let surface0 = RevolutedCurve::by_revolution(line.clone(), Point3::origin(), Vector3::unit_y());
    /// assert!(surface0.is_front_fixed());
    /// let surface1 = RevolutedCurve::by_revolution(line, Point3::new(1.0, 0.0, 0.0), Vector3::unit_y());
    /// assert!(!surface1.is_front_fixed());
    /// ```
    #[inline(always)]
    pub fn is_front_fixed(&self) -> bool { self.revolution.contains(self.curve.front()) }
    /// Returns true if the back point of the curve is on the axis of rotation.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let line = BSplineCurve::new(
    ///     KnotVec::bezier_knot(1),
    ///     vec![Point3::new(0.0, 0.0, 1.0), Point3::new(0.0, 0.0, 0.0)],
    /// );
    /// let surface0 = RevolutedCurve::by_revolution(line.clone(), Point3::origin(), Vector3::unit_y());
    /// assert!(surface0.is_back_fixed());
    /// let surface1 = RevolutedCurve::by_revolution(line, Point3::new(1.0, 0.0, 0.0), Vector3::unit_y());
    /// assert!(!surface1.is_back_fixed());
    /// ```
    #[inline(always)]
    pub fn is_back_fixed(&self) -> bool { self.revolution.contains(self.curve.back()) }
}

impl<C: ParametricCurve3D + BoundedCurve> SearchParameter<D2> for RevolutedCurve<C> {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let (t0, t1) = self.curve.range_tuple();
        if self.is_front_fixed() && self.curve.front().near(&point) {
            match hint.into() {
                SPHint2D::Parameter(_, y) => Some((t0, y)),
                SPHint2D::Range((_, y), _) => Some((t0, y)),
                SPHint2D::None => Some((t0, 0.0)),
            }
        } else if self.is_back_fixed() && self.curve.back().near(&point) {
            match hint.into() {
                SPHint2D::Parameter(_, y) => Some((t1, y)),
                SPHint2D::Range(_, (_, y)) => Some((t1, y)),
                SPHint2D::None => Some((t1, 2.0 * PI)),
            }
        } else {
            let proj_curve = ProjectedCurve {
                curve: &self.curve,
                revolution: self.revolution,
            };
            let p = self.revolution.proj_point(point);
            let hint0 = match hint.into() {
                SPHint2D::Parameter(x, _) => SPHint1D::Parameter(x),
                SPHint2D::Range((x0, _), (x1, _)) => SPHint1D::Range(x0, x1),
                SPHint2D::None => SPHint1D::None,
            };
            let t = proj_curve.search_parameter(p, hint0, trials)?;
            let p = self.curve.subs(t);
            let ang = self.revolution.proj_angle(p, point);
            match self.subs(t, ang).near(&point) {
                true => Some((t, ang)),
                false => None,
            }
        }
    }
}

impl<C: ParametricCurve3D + BoundedCurve> SearchNearestParameter<D2> for RevolutedCurve<C> {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let (t0, t1) = self.curve.range_tuple();
        let on_axis = move |o: Point3, normal: Vector3| {
            let op = point - o;
            op.cross(self.revolution.axis).so_small() && op.dot(normal) >= 0.0
        };
        if self.is_front_fixed() && on_axis(self.curve.front(), self.normal(t0, 0.0)) {
            match hint.into() {
                SPHint2D::Parameter(_, y) => Some((t0, y)),
                SPHint2D::Range((_, y), _) => Some((t0, y)),
                SPHint2D::None => Some((t0, 0.0)),
            }
        } else if self.is_back_fixed() && on_axis(self.curve.back(), self.normal(t1, 0.0)) {
            match hint.into() {
                SPHint2D::Parameter(_, y) => Some((t1, y)),
                SPHint2D::Range(_, (_, y)) => Some((t1, y)),
                SPHint2D::None => Some((t1, 2.0 * PI)),
            }
        } else {
            let proj_curve = ProjectedCurve {
                curve: &self.curve,
                revolution: self.revolution,
            };
            let p = self.revolution.proj_point(point);
            let hint0 = match hint.into() {
                SPHint2D::Parameter(x, _) => SPHint1D::Parameter(x),
                SPHint2D::Range((x0, _), (x1, _)) => SPHint1D::Range(x0, x1),
                SPHint2D::None => SPHint1D::None,
            };
            let t = proj_curve.search_nearest_parameter(p, hint0, trials)?;
            let p = self.curve.subs(t);
            Some((t, self.revolution.proj_angle(p, point)))
        }
    }
}

fn sub_include<C0, C1>(
    surface: &RevolutedCurve<C0>,
    curve: &C1,
    knots: &[f64],
    degree: usize,
) -> bool
where
    C0: ParametricCurve3D + BoundedCurve,
    C1: ParametricCurve3D + BoundedCurve,
{
    let first = curve.subs(knots[0]);
    let mut hint = match surface.search_parameter(first, None, INCLUDE_CURVE_TRIALS) {
        Some(hint) => hint,
        None => return false,
    };
    knots
        .windows(2)
        .flat_map(move |knot| {
            (1..=degree).map(move |i| {
                let s = i as f64 / degree as f64;
                knot[0] * (1.0 - s) + knot[1] * s
            })
        })
        .all(move |t| {
            let pt = ParametricCurve::subs(curve, t);
            surface
                .search_parameter(pt, Some(hint), INCLUDE_CURVE_TRIALS)
                .or_else(|| surface.search_parameter(pt, None, INCLUDE_CURVE_TRIALS))
                .map(|res| hint = res)
                .is_some()
        })
}

impl IncludeCurve<BSplineCurve<Point3>> for RevolutedCurve<&BSplineCurve<Point3>> {
    fn include(&self, curve: &BSplineCurve<Point3>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let degree = usize::max(2, usize::max(curve.degree(), self.curve.degree()));
        sub_include(self, curve, &knots, degree)
    }
}

impl IncludeCurve<BSplineCurve<Point3>> for RevolutedCurve<BSplineCurve<Point3>> {
    fn include(&self, curve: &BSplineCurve<Point3>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let degree = usize::max(2, usize::max(curve.degree(), self.curve.degree()));
        sub_include(self, curve, &knots, degree)
    }
}

impl IncludeCurve<BSplineCurve<Point3>> for RevolutedCurve<&NurbsCurve<Vector4>> {
    fn include(&self, curve: &BSplineCurve<Point3>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let degree = curve.degree() + usize::max(2, self.curve.degree());
        sub_include(self, curve, &knots, degree)
    }
}

impl IncludeCurve<BSplineCurve<Point3>> for RevolutedCurve<NurbsCurve<Vector4>> {
    fn include(&self, curve: &BSplineCurve<Point3>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let degree = curve.degree() + usize::max(2, self.curve.degree());
        sub_include(self, curve, &knots, degree)
    }
}

impl IncludeCurve<NurbsCurve<Vector4>> for RevolutedCurve<&BSplineCurve<Point3>> {
    fn include(&self, curve: &NurbsCurve<Vector4>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let degree = curve.degree() + usize::max(2, self.curve.degree());
        sub_include(self, curve, &knots, degree)
    }
}

impl IncludeCurve<NurbsCurve<Vector4>> for RevolutedCurve<BSplineCurve<Point3>> {
    fn include(&self, curve: &NurbsCurve<Vector4>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let degree = curve.degree() + usize::max(2, self.curve.degree());
        sub_include(self, curve, &knots, degree)
    }
}

impl IncludeCurve<NurbsCurve<Vector4>> for RevolutedCurve<&NurbsCurve<Vector4>> {
    fn include(&self, curve: &NurbsCurve<Vector4>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let degree = curve.degree() + usize::max(2, self.curve.degree());
        sub_include(self, curve, &knots, degree)
    }
}

impl IncludeCurve<NurbsCurve<Vector4>> for RevolutedCurve<NurbsCurve<Vector4>> {
    fn include(&self, curve: &NurbsCurve<Vector4>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let degree = curve.degree() + usize::max(2, self.curve.degree());
        sub_include(self, curve, &knots, degree)
    }
}

impl<C> ParameterDivision2D for RevolutedCurve<C>
where C: ParametricCurve3D + ParameterDivision1D<Point = Point3>
{
    fn parameter_division<T: TesselationSplitMethod>(
        &self,
        (urange, vrange): ((f64, f64), (f64, f64)),
        split: T,
    ) -> (Vec<f64>, Vec<f64>) {
        let curve_division = self.curve.parameter_division(urange, split);
        let max = curve_division
            .1
            .into_iter()
            .fold(0.0, |max2, pt| {
                let h = self.revolution.proj_point(pt).y;
                f64::max(max2, h)
            })
            .sqrt();
        let acos = f64::acos(1.0 - split.tol() / max);
        let div: usize = 1 + ((vrange.1 - vrange.0) / acos).floor() as usize;
        let circle_division = (0..=div)
            .map(|j| vrange.0 + (vrange.1 - vrange.0) * j as f64 / div as f64)
            .collect();
        (curve_division.0, circle_division)
    }
}

fn from_axis_angle_derivation(n: usize, axis: Vector3, angle: Rad<f64>) -> Matrix3 {
    let (s, c) = Rad::sin_cos(angle);
    let (s, c) = match n % 4 {
        0 => (s, c),
        1 => (c, -s),
        2 => (-s, -c),
        _ => (-c, s),
    };
    let _1subc = match n {
        0 => 1.0 - c,
        _ => -c,
    };

    #[allow(clippy::deprecated_cfg_attr)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    Matrix3::new(
        _1subc * axis.x * axis.x + c,
        _1subc * axis.x * axis.y + s * axis.z,
        _1subc * axis.x * axis.z - s * axis.y,

        _1subc * axis.x * axis.y - s * axis.z,
        _1subc * axis.y * axis.y + c,
        _1subc * axis.y * axis.z + s * axis.x,

        _1subc * axis.x * axis.z + s * axis.y,
        _1subc * axis.y * axis.z - s * axis.x,
        _1subc * axis.z * axis.z + c,
    )
}
