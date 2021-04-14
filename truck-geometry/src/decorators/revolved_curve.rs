use super::*;
use std::f64::consts::PI;

impl<C> RevolutedCurve<C> {
    /// Creates a surface by revoluting a curve.
    #[inline(always)]
    pub fn by_revolution(curve: C, origin: Point3, axis: Vector3) -> Self {
        RevolutedCurve {
            curve,
            origin,
            axis: axis.normalize(),
        }
    }
    #[inline(always)]
    fn point_rotation_matrix(&self, v: f64) -> Matrix4 {
        Matrix4::from_translation(self.origin.to_vec())
            * Matrix4::from_axis_angle(self.axis, Rad(v))
            * Matrix4::from_translation(-self.origin.to_vec())
    }
    #[inline(always)]
    fn vector_rotation_matrix(&self, v: f64) -> Matrix4 {
        Matrix4::from_axis_angle(self.axis, Rad(v))
    }
    #[inline(always)]
    fn derivation_rotation_matrix(&self, v: f64) -> Matrix4 {
        let n = self.axis;
        Matrix3::new(
            n[0] * n[0] * f64::sin(v) - f64::sin(v),
            n[0] * n[1] * f64::sin(v) + n[2] * f64::cos(v),
            n[0] * n[2] * f64::sin(v) - n[1] * f64::cos(v),
            n[0] * n[1] * f64::sin(v) - n[2] * f64::cos(v),
            n[1] * n[1] * f64::sin(v) - f64::sin(v),
            n[1] * n[2] * f64::sin(v) + n[0] * f64::cos(v),
            n[0] * n[2] * f64::sin(v) + n[1] * f64::cos(v),
            n[1] * n[2] * f64::sin(v) - n[0] * f64::cos(v),
            n[2] * n[2] * f64::sin(v) - f64::sin(v),
        )
        .into()
    }
    /// Returns the entity curve
    #[inline(always)]
    pub fn entity_curve(&self) -> &C { &self.curve }
    /// Returns the entity curve
    #[inline(always)]
    pub fn entity_curve_mut(&mut self) -> &mut C { &mut self.curve }
    /// Returns origin of revolution
    #[inline(always)]
    pub fn origin(&self) -> Point3 { self.origin }
    /// Returns axis of revolution
    #[inline(always)]
    pub fn axis(&self) -> Vector3 { self.axis }

    #[inline(always)]
    fn proj_point(&self, pt: Point3) -> (f64, f64) {
        let r = pt - self.origin;
        let z = r.dot(self.axis);
        let h = r - z * self.axis;
        (z, h.magnitude2())
    }
}

impl<C: ParametricCurve<Point = Point3, Vector = Vector3>> RevolutedCurve<C> {
    /// Returns true if the front point of the curve is on the axis of rotation.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let line = BSplineCurve::new(
    ///     KnotVec::bezier_knot(1),
    ///     vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)],
    /// );
    /// let surface0 = RevolutedCurve::by_revolution(line.clone(), Point3::origin(), Vector3::unit_y());
    /// assert!(surface0.is_front_fixed());
    /// let surface1 = RevolutedCurve::by_revolution(line, Point3::new(1.0, 0.0, 0.0), Vector3::unit_y());
    /// assert!(!surface1.is_front_fixed());
    /// ```
    #[inline(always)]
    pub fn is_front_fixed(&self) -> bool {
        (self.curve.front() - self.origin)
            .cross(self.axis)
            .so_small()
    }
    /// Returns true if the back point of the curve is on the axis of rotation.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let line = BSplineCurve::new(
    ///     KnotVec::bezier_knot(1),
    ///     vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0)],
    /// );
    /// let surface0 = RevolutedCurve::by_revolution(line.clone(), Point3::origin(), Vector3::unit_y());
    /// assert!(surface0.is_back_fixed());
    /// let surface1 = RevolutedCurve::by_revolution(line, Point3::new(1.0, 0.0, 0.0), Vector3::unit_y());
    /// assert!(!surface1.is_back_fixed());
    /// ```
    #[inline(always)]
    pub fn is_back_fixed(&self) -> bool {
        (self.curve.back() - self.origin)
            .cross(self.axis)
            .so_small()
    }

    fn sub_search_parameter(
        &self,
        point: Point3,
        hint: (f64, f64),
        trials: usize,
    ) -> Option<(f64, f64)> {
        surface_search_nearest_parameter(self, point, hint, trials).and_then(|(u, v)| {
            if self.subs(u, v).near(&point) {
                Some((u, v))
            } else {
                let v = if v > PI { v - PI } else { v + PI };
                if self.subs(u, v).near(&point) {
                    Some((u, v))
                } else {
                    None
                }
            }
        })
    }

    /// Searches the parameter `(u, v)` such that `self.subs(u, v).near(&point)` by Newton's method.
    /// Returns `None` if:
    /// - the converged parameter `(u, v)` is not satisfied `self.subs(u, v).near(&point)`.
    /// - the number of attempts exceeds `trial` i.e. if `trial == 0`, then the trial is only one time.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let line = BSplineCurve::new(
    ///     KnotVec::bezier_knot(1),
    ///     vec![Vector3::new(0.0, 2.0, 1.0), Vector3::new(1.0, 0.0, 0.0)],
    /// );
    /// let surface = RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_y());
    /// let pt = Point3::new(-0.5, 1.0, 0.5);
    /// let (u, v) = surface.search_parameter(pt, (0.4, 1.2) ,100).unwrap();
    /// assert_near!(surface.subs(u, v), pt);
    /// ```
    #[inline(always)]
    pub fn search_parameter(
        &self,
        point: Point3,
        hint: (f64, f64),
        trials: usize,
    ) -> Option<(f64, f64)> {
        let (t0, t1) = self.curve.parameter_range();
        if self.is_front_fixed() && self.curve.front().near(&point) {
            Some((t0, hint.1))
        } else if self.is_back_fixed() && self.curve.back().near(&point) {
            Some((t1, hint.1))
        } else {
            match self.sub_search_parameter(point, hint, trials) {
                Some(got) => Some(got),
                None => self.sub_search_parameter(point, presearch(self, point), trials),
            }
        }
    }
}

impl<C: ParametricCurve<Point = Point3, Vector = Vector3>> ParametricSurface for RevolutedCurve<C> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 {
        self.point_rotation_matrix(v)
            .transform_point(self.curve.subs(u))
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        self.vector_rotation_matrix(v)
            .transform_vector(self.curve.der(u))
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        let pt = self.curve.subs(u);
        let radius = self.axis.cross(pt - self.origin);
        self.vector_rotation_matrix(v).transform_vector(radius)
    }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector3 {
        self.vector_rotation_matrix(v)
            .transform_vector(self.curve.der2(u))
    }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector3 {
        let pt = self.curve.subs(u);
        let z = self.proj_point(pt).0;
        let radius = pt - self.origin - z * self.axis;
        -self.vector_rotation_matrix(v).transform_vector(radius)
    }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector3 {
        self.derivation_rotation_matrix(v)
            .transform_vector(self.curve.der(u))
    }
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        let (u0, u1) = self.curve.parameter_range();
        let (uder, vder) = if u.near(&u0) {
            let pt = self.curve.subs(u);
            let radius = self.axis.cross(pt - self.origin);
            if radius.so_small() {
                let uder = self.curve.der(u);
                (uder, self.axis.cross(uder))
            } else {
                (self.uder(u, v), self.vder(u, v))
            }
        } else if u.near(&u1) {
            let pt = self.curve.subs(u);
            let radius = self.axis.cross(pt - self.origin);
            if radius.so_small() {
                let uder = self.curve.der(u);
                (uder, uder.cross(self.axis))
            } else {
                (self.uder(u, v), self.vder(u, v))
            }
        } else {
            (self.uder(u, v), self.vder(u, v))
        };
        uder.cross(vder).normalize()
    }
}

impl<C: ParametricCurve<Point = Point3, Vector = Vector3>> BoundedSurface for RevolutedCurve<C> {
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) {
        (self.curve.parameter_range(), (0.0, 2.0 * PI))
    }
}

impl<C: Clone> Invertible for RevolutedCurve<C> {
    #[inline(always)]
    fn invert(&mut self) { self.axis = -self.axis; }
    #[inline(always)]
    fn inverse(&self) -> Self {
        RevolutedCurve {
            curve: self.curve.clone(),
            origin: self.origin,
            axis: -self.axis,
        }
    }
}

impl<'a> IncludeCurve<BSplineCurve<Vector3>> for RevolutedCurve<&'a BSplineCurve<Vector3>> {
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let first = ParametricCurve::subs(curve, knots[0]);
        let degree = usize::max(2, usize::max(curve.degree(), self.curve.degree()));
        let mut hint = presearch(self, first);
        if self
            .search_parameter(first, hint, INCLUDE_CURVE_TRIALS)
            .is_none()
        {
            return false;
        }
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=degree).map(move |i| {
                    let s = i as f64 / degree as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                match self.search_parameter(
                    ParametricCurve::subs(curve, t),
                    hint,
                    INCLUDE_CURVE_TRIALS,
                ) {
                    Some(got) => {
                        hint = got;
                        true
                    }
                    None => false,
                }
            })
    }
}

impl IncludeCurve<BSplineCurve<Vector3>> for RevolutedCurve<BSplineCurve<Vector3>> {
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        let surface = RevolutedCurve::by_revolution(&self.curve, self.origin, self.axis);
        surface.include(curve)
    }
}

impl<'a> IncludeCurve<BSplineCurve<Vector3>> for RevolutedCurve<&'a NURBSCurve<Vector4>> {
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let first = ParametricCurve::subs(curve, knots[0]);
        let degree = usize::max(2, curve.degree() + self.curve.degree());
        let mut hint = presearch(self, first);
        if self
            .search_parameter(first, hint, INCLUDE_CURVE_TRIALS)
            .is_none()
        {
            return false;
        }
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=degree).map(move |i| {
                    let s = i as f64 / degree as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                match self.search_parameter(
                    ParametricCurve::subs(curve, t),
                    hint,
                    INCLUDE_CURVE_TRIALS,
                ) {
                    Some(got) => {
                        hint = got;
                        true
                    }
                    None => false,
                }
            })
    }
}

impl IncludeCurve<BSplineCurve<Vector3>> for RevolutedCurve<NURBSCurve<Vector4>> {
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        let surface = RevolutedCurve::by_revolution(&self.curve, self.origin, self.axis);
        surface.include(curve)
    }
}

impl<'a> IncludeCurve<NURBSCurve<Vector4>> for RevolutedCurve<&'a BSplineCurve<Vector3>> {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let first = ParametricCurve::subs(curve, knots[0]);
        let degree = usize::max(2, curve.degree() + self.curve.degree());
        let mut hint = presearch(self, first);
        if self
            .search_parameter(first, hint, INCLUDE_CURVE_TRIALS)
            .is_none()
        {
            return false;
        }
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=degree).map(move |i| {
                    let s = i as f64 / degree as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                match self.search_parameter(
                    ParametricCurve::subs(curve, t),
                    hint,
                    INCLUDE_CURVE_TRIALS,
                ) {
                    Some(got) => {
                        hint = got;
                        true
                    }
                    None => false,
                }
            })
    }
}

impl IncludeCurve<NURBSCurve<Vector4>> for RevolutedCurve<BSplineCurve<Vector3>> {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let surface = RevolutedCurve::by_revolution(&self.curve, self.origin, self.axis);
        surface.include(curve)
    }
}

impl<'a> IncludeCurve<NURBSCurve<Vector4>> for RevolutedCurve<&'a NURBSCurve<Vector4>> {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let knots = curve.knot_vec().to_single_multi().0;
        let first = ParametricCurve::subs(curve, knots[0]);
        let degree = usize::max(2, curve.degree() + self.curve.degree());
        let mut hint = presearch(self, first);
        if self
            .search_parameter(first, hint, INCLUDE_CURVE_TRIALS)
            .is_none()
        {
            return false;
        }
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=degree).map(move |i| {
                    let s = i as f64 / degree as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                match self.search_parameter(
                    ParametricCurve::subs(curve, t),
                    hint,
                    INCLUDE_CURVE_TRIALS,
                ) {
                    Some(got) => {
                        hint = got;
                        true
                    }
                    None => false,
                }
            })
    }
}

impl IncludeCurve<NURBSCurve<Vector4>> for RevolutedCurve<NURBSCurve<Vector4>> {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let surface = RevolutedCurve::by_revolution(&self.curve, self.origin, self.axis);
        surface.include(curve)
    }
}

impl<C> ParameterDivision2D for RevolutedCurve<C>
where C: ParametricCurve<Point = Point3, Vector = Vector3> + ParameterDivision1D
{
    fn parameter_division(&self, tol: f64) -> (Vec<f64>, Vec<f64>) {
        let curve_division = self.curve.parameter_division(tol);
        let max = curve_division
            .iter()
            .fold(0.0, |max2, t| {
                let pt = self.curve.subs(*t);
                let h = self.proj_point(pt).1;
                f64::max(max2, h)
            })
            .sqrt();
        let acos = f64::acos(1.0 - tol / max);
        let div: usize = 1 + (2.0 * PI / acos).floor() as usize;
        let circle_division = (0..=div)
            .map(|j| 2.0 * PI * j as f64 / div as f64)
            .collect();
        (curve_division, circle_division)
    }
}

#[test]
fn revolve_test() {
    let pt0 = Vector3::new(0.0, 2.0, 1.0);
    let pt1 = Vector3::new(1.0, 0.0, 0.0);
    let curve = BSplineCurve::new(KnotVec::bezier_knot(1), vec![pt0, pt1]);
    let surface = RevolutedCurve::by_revolution(curve, Point3::origin(), Vector3::unit_y());
    const N: usize = 100;
    for i in 0..=N {
        for j in 0..=N {
            let u = i as f64 / N as f64;
            let v = 2.0 * PI * j as f64 / N as f64;
            let res = surface.subs(u, v);
            let ans = Point3::new(
                u * f64::cos(v) + (1.0 - u) * f64::sin(v),
                2.0 * (1.0 - u),
                -u * f64::sin(v) + (1.0 - u) * f64::cos(v),
            );
            assert_near!(res, ans);
            let res_uder = surface.uder(u, v);
            let ans_uder =
                Vector3::new(f64::cos(v) - f64::sin(v), -2.0, -f64::sin(v) - f64::cos(v));
            assert_near!(res_uder, ans_uder);
            let res_vder = surface.vder(u, v);
            let ans_vder = Vector3::new(
                -u * f64::sin(v) + (1.0 - u) * f64::cos(v),
                0.0,
                -u * f64::cos(v) - (1.0 - u) * f64::sin(v),
            );
            assert_near!(res_vder, ans_vder);
            let res_uuder = surface.uuder(u, v);
            let ans_uuder = Vector3::zero();
            assert_near!(res_uuder, ans_uuder);
            let res_uvder = surface.uvder(u, v);
            let ans_uvder =
                Vector3::new(-f64::sin(v) - f64::cos(v), 0.0, -f64::cos(v) + f64::sin(v));
            assert_near!(res_uvder, ans_uvder);
            let res_vvder = surface.vvder(u, v);
            let ans_vvder = Vector3::new(
                -u * f64::cos(v) - (1.0 - u) * f64::sin(v),
                0.0,
                u * f64::sin(v) - (1.0 - u) * f64::cos(v),
            );
            assert_near!(res_vvder, ans_vvder);
            let normal = surface.normal(u, v);
            assert!(normal.dot(res_uder).so_small());
            assert!(normal.dot(res_vder).so_small());
        }
    }
}

#[test]
fn search_parameter_with_fixed_points() {
    let line = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, -1.0, 0.0),
        ],
    );
    let surface = RevolutedCurve::by_revolution(line, Point3::origin(), Vector3::unit_y());

    let para = surface
        .search_parameter(Point3::new(0.0, 1.0, 0.0), (0.5, 0.3), 10)
        .unwrap();
    assert_near!(para.0, 0.0);
    assert_near!(para.1, 0.3);

    let para = surface
        .search_parameter(Point3::new(0.0, -1.0, 0.0), (0.5, 0.3), 10)
        .unwrap();
    assert_near!(para.0, 1.0);
    assert_near!(para.1, 0.3);
}
