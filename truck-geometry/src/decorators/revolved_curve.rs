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

fn presearch<C>(curve: &C, pt0: C::Point) -> f64
where
    C: ParametricCurve,
    C::Point: MetricSpace<Metric = f64> + Copy, {
    const N: usize = 50;
    let range = curve.parameter_range();
    let mut min_dist2 = std::f64::INFINITY;
    let mut res = range.0;
    for i in 0..=N {
        let t = i as f64 / N as f64;
        let t = range.0 * (1.0 - t) + range.1 * t;
        let pt = curve.subs(t);
        let dist2 = pt.distance2(pt0);
        if dist2 < min_dist2 {
            res = t;
            min_dist2 = dist2;
        }
    }
    res
}

impl IncludeCurve<BSplineCurve<Vector3>> for RevolutedCurve<BSplineCurve<Vector3>> {
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        let pts = self
            .curve
            .control_points()
            .iter()
            .map(|pt| Vector1::new(self.proj_point(Point3::from_vec(*pt)).0))
            .collect();
        let projed = &BSplineCurve::new(self.curve.knot_vec().clone(), pts);
        let knots = curve.knot_vec().to_single_multi().0;
        let first = curve.subs(knots[0]);
        let first = self.proj_point(Point3::from_vec(first)).0;
        let mut hint = presearch(projed, Point1::new(first));
        hint = match projed.search_nearest_parameter(Vector1::new(first), hint) {
            Some(got) => got,
            None => return false,
        };
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=curve.degree()).map(move |i| {
                    let s = i as f64 / projed.degree() as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                let pt = curve.subs(t);
                let (z, r0) = self.proj_point(Point3::from_vec(pt));
                match projed.search_nearest_parameter(Vector1::new(z), hint) {
                    Some(got) => {
                        hint = got;
                        let pt = self.curve.subs(hint);
                        let (w, r1) = self.proj_point(Point3::from_vec(pt));
                        z.near(&w) && r0.near(&r1)
                    }
                    None => false,
                }
            })
    }
}

impl<'a> IncludeCurve<BSplineCurve<Vector3>> for RevolutedCurve<&'a BSplineCurve<Vector3>> {
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        let pts = self
            .curve
            .control_points()
            .iter()
            .map(|pt| Vector1::new(self.proj_point(Point3::from_vec(*pt)).0))
            .collect();
        let projed = &BSplineCurve::new(self.curve.knot_vec().clone(), pts);
        let knots = curve.knot_vec().to_single_multi().0;
        let first = curve.subs(knots[0]);
        let first = self.proj_point(Point3::from_vec(first)).0;
        let mut hint = presearch(projed, Point1::new(first));
        hint = match projed.search_nearest_parameter(Vector1::new(first), hint) {
            Some(got) => got,
            None => return false,
        };
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=curve.degree()).map(move |i| {
                    let s = i as f64 / projed.degree() as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                let pt = curve.subs(t);
                let (z, r0) = self.proj_point(Point3::from_vec(pt));
                match projed.search_nearest_parameter(Vector1::new(z), hint) {
                    Some(got) => {
                        hint = got;
                        let pt = self.curve.subs(hint);
                        let (w, r1) = self.proj_point(Point3::from_vec(pt));
                        z.near(&w) && r0.near(&r1)
                    }
                    None => false,
                }
            })
    }
}

impl IncludeCurve<BSplineCurve<Vector3>> for RevolutedCurve<NURBSCurve<Vector4>> {
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        let pts = self
            .curve
            .control_points()
            .iter()
            .map(|pt| {
                let (pt, w) = (pt.to_point(), pt.weight());
                let z = self.proj_point(pt).0;
                Vector2::new(z * w, w)
            })
            .collect();
        let projed = &NURBSCurve::new(BSplineCurve::new(self.curve.knot_vec().clone(), pts));
        let knots = curve.knot_vec().to_single_multi().0;
        let first = curve.subs(knots[0]);
        let first = Point1::new(self.proj_point(Point3::from_vec(first)).0);
        let mut hint = presearch(projed, first);
        hint = match projed.search_nearest_parameter(first, hint) {
            Some(got) => got,
            None => return false,
        };
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=curve.degree()).map(move |i| {
                    let s = i as f64 / projed.degree() as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                let pt = curve.subs(t);
                let (z, r0) = self.proj_point(Point3::from_vec(pt));
                match projed.search_nearest_parameter(Point1::new(z), hint) {
                    Some(got) => {
                        hint = got;
                        let pt = self.curve.subs(hint);
                        let (w, r1) = self.proj_point(pt);
                        z.near(&w) && r0.near(&r1)
                    }
                    None => false,
                }
            })
    }
}

impl<'a> IncludeCurve<BSplineCurve<Vector3>> for RevolutedCurve<&'a NURBSCurve<Vector4>> {
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        let pts = self
            .curve
            .control_points()
            .iter()
            .map(|pt| {
                let (pt, w) = (pt.to_point(), pt.weight());
                let z = self.proj_point(pt).0;
                Vector2::new(z * w, w)
            })
            .collect();
        let projed = &NURBSCurve::new(BSplineCurve::new(self.curve.knot_vec().clone(), pts));
        let knots = curve.knot_vec().to_single_multi().0;
        let first = curve.subs(knots[0]);
        let first = Point1::new(self.proj_point(Point3::from_vec(first)).0);
        let mut hint = presearch(projed, first);
        hint = match projed.search_nearest_parameter(first, hint) {
            Some(got) => got,
            None => return false,
        };
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=curve.degree()).map(move |i| {
                    let s = i as f64 / projed.degree() as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                let pt = curve.subs(t);
                let (z, r0) = self.proj_point(Point3::from_vec(pt));
                match projed.search_nearest_parameter(Point1::new(z), hint) {
                    Some(got) => {
                        hint = got;
                        let pt = self.curve.subs(hint);
                        let (w, r1) = self.proj_point(pt);
                        z.near(&w) && r0.near(&r1)
                    }
                    None => false,
                }
            })
    }
}

impl IncludeCurve<NURBSCurve<Vector4>> for RevolutedCurve<BSplineCurve<Vector3>> {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let pts = self
            .curve
            .control_points()
            .iter()
            .map(|pt| Vector1::new(self.proj_point(Point3::from_vec(*pt)).0))
            .collect();
        let projed = &BSplineCurve::new(self.curve.knot_vec().clone(), pts);
        let knots = curve.knot_vec().to_single_multi().0;
        let first = curve.subs(knots[0]);
        let first = self.proj_point(first).0;
        let mut hint = presearch(projed, Point1::new(first));
        hint = match projed.search_nearest_parameter(Vector1::new(first), hint) {
            Some(got) => got,
            None => return false,
        };
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=curve.degree() * 2).map(move |i| {
                    let s = i as f64 / projed.degree() as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                let pt = curve.subs(t);
                let (z, r0) = self.proj_point(pt);
                match projed.search_nearest_parameter(Vector1::new(z), hint) {
                    Some(got) => {
                        hint = got;
                        let pt = self.curve.subs(hint);
                        let (w, r1) = self.proj_point(Point3::from_vec(pt));
                        z.near(&w) && r0.near(&r1)
                    }
                    None => false,
                }
            })
    }
}

impl<'a> IncludeCurve<NURBSCurve<Vector4>> for RevolutedCurve<&'a BSplineCurve<Vector3>> {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let pts = self
            .curve
            .control_points()
            .iter()
            .map(|pt| Vector1::new(self.proj_point(Point3::from_vec(*pt)).0))
            .collect();
        let projed = &BSplineCurve::new(self.curve.knot_vec().clone(), pts);
        let knots = curve.knot_vec().to_single_multi().0;
        let first = curve.subs(knots[0]);
        let first = self.proj_point(first).0;
        let mut hint = presearch(projed, Point1::new(first));
        hint = match projed.search_nearest_parameter(Vector1::new(first), hint) {
            Some(got) => got,
            None => return false,
        };
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=curve.degree() * 2).map(move |i| {
                    let s = i as f64 / projed.degree() as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                let pt = curve.subs(t);
                let (z, r0) = self.proj_point(pt);
                match projed.search_nearest_parameter(Vector1::new(z), hint) {
                    Some(got) => {
                        hint = got;
                        let pt = self.curve.subs(hint);
                        let (w, r1) = self.proj_point(Point3::from_vec(pt));
                        z.near(&w) && r0.near(&r1)
                    }
                    None => false,
                }
            })
    }
}

impl IncludeCurve<NURBSCurve<Vector4>> for RevolutedCurve<NURBSCurve<Vector4>> {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let pts = self
            .curve
            .control_points()
            .iter()
            .map(|pt| {
                let (pt, w) = (pt.to_point(), pt.weight());
                let z = self.proj_point(pt).0;
                Vector2::new(z * w, w)
            })
            .collect();
        let projed = &NURBSCurve::new(BSplineCurve::new(self.curve.knot_vec().clone(), pts));
        let knots = curve.knot_vec().to_single_multi().0;
        let first = curve.subs(knots[0]);
        let first = Point1::new(self.proj_point(first).0);
        let mut hint = presearch(projed, first);
        hint = match projed.search_nearest_parameter(first, hint) {
            Some(got) => got,
            None => return false,
        };
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=curve.degree() * 2).map(move |i| {
                    let s = i as f64 / projed.degree() as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                let pt = curve.subs(t);
                let (z, r0) = self.proj_point(pt);
                match projed.search_nearest_parameter(Point1::new(z), hint) {
                    Some(got) => {
                        hint = got;
                        let pt = self.curve.subs(hint);
                        let (w, r1) = self.proj_point(pt);
                        z.near(&w) && r0.near(&r1)
                    }
                    None => false,
                }
            })
    }
}

impl<'a> IncludeCurve<NURBSCurve<Vector4>> for RevolutedCurve<&'a NURBSCurve<Vector4>> {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let pts = self
            .curve
            .control_points()
            .iter()
            .map(|pt| {
                let (pt, w) = (pt.to_point(), pt.weight());
                let z = self.proj_point(pt).0;
                Vector2::new(z * w, w)
            })
            .collect();
        let projed = &NURBSCurve::new(BSplineCurve::new(self.curve.knot_vec().clone(), pts));
        let knots = curve.knot_vec().to_single_multi().0;
        let first = curve.subs(knots[0]);
        let first = Point1::new(self.proj_point(first).0);
        let mut hint = presearch(projed, first);
        hint = match projed.search_nearest_parameter(first, hint) {
            Some(got) => got,
            None => return false,
        };
        knots
            .windows(2)
            .flat_map(move |knot| {
                (1..=curve.degree() * 2).map(move |i| {
                    let s = i as f64 / projed.degree() as f64;
                    knot[0] * (1.0 - s) + knot[1] * s
                })
            })
            .all(move |t| {
                let pt = curve.subs(t);
                let (z, r0) = self.proj_point(pt);
                match projed.search_nearest_parameter(Point1::new(z), hint) {
                    Some(got) => {
                        hint = got;
                        let pt = self.curve.subs(hint);
                        let (w, r1) = self.proj_point(pt);
                        z.near(&w) && r0.near(&r1)
                    }
                    None => false,
                }
            })
    }
}

#[test]
fn revolve_test() {
    let pt0 = Vector3::new(0.0, 2.0, 1.0);
    let pt1 = Vector3::new(1.0, 0.0, 0.0);
    let vec = pt1 - pt0;
    let curve = BSplineCurve::new(KnotVec::bezier_knot(1), vec![pt0, pt1]);
    let surface = RevolutedCurve::by_revolution(curve, Point3::origin(), Vector3::unit_y());
    const N: usize = 100;
    for i in 0..=N {
        for j in 0..=N {
            let u = i as f64 / N as f64;
            let v = 2.0 * PI * j as f64 / N as f64;
            let uder = Matrix3::from_axis_angle(Vector3::unit_y(), Rad(v)) * vec;
            assert_near!(surface.uder(u, v), uder);
            let pt = pt0 * (1.0 - u) + pt1 * u;
            let vec = Vector3::new(pt[2], 0.0, -pt[0]);
            let vder = Matrix3::from_axis_angle(Vector3::unit_y(), Rad(v)) * vec;
            assert_near!(surface.vder(u, v), vder);
            let n = surface.normal(u, v);
            assert!(n.dot(uder).so_small2());
            assert!(n.dot(vder).so_small2());
        }
    }
}
