use super::*;

impl<C, S0, S1, R> RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    /// calculate contact circle corresponding to the parameter `t`, i.e.
    /// - the circle contact to the surfaces, `surface0` and `surface1`,
    /// - the center of the circle is on the plane with the origin = `edge_curve.subs(t)` and
    ///   the normal = `edge_curve.der(t)`.
    /// - the radius of the circle is `radius.subs(t)`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let line = Line(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    /// let plane0 = Plane::xy();
    /// let plane1 = Plane::zx();
    /// let r = 0.5;
    /// let t = 0.75;
    /// let rfs = RbfSurface::new(line, plane0, plane1, r);
    /// let cc = rfs.contact_circle(t).unwrap();
    ///
    /// assert_near!(cc.center(), Point3::new(t, r, r));
    /// assert_near!(cc.axis(), Vector3::new(-1.0, 0.0, 0.0));
    /// let cp0 = cc.contact_point0();
    /// assert_near!(cp0.point, Point3::new(t, r, 0.0));
    /// assert_near!(cp0.uv, Point2::new(t, r));
    /// let cp1 = cc.contact_point1();
    /// assert_near!(cp1.point, Point3::new(t, 0.0, r));
    /// assert_near!(cp1.uv, Point2::new(r, t));
    /// ```
    pub fn contact_circle(&self, t: f64) -> Option<ContactCircle> {
        let poc = (self.edge_curve.subs(t), self.edge_curve.der(t));
        let radius = self.radius.subs(t);
        ContactCircle::try_new(poc, t, &self.surface0, &self.surface1, radius)
    }

    /// the derivation of the orbit of the contact circle
    pub fn center_der(&self, cc: ContactCircle) -> Vector3 {
        let p = self.edge_curve.subs(cc.t);
        let der = self.edge_curve.der(cc.t);
        let der2 = self.edge_curve.der2(cc.t);

        let (u0, v0) = cc.contact_point0.uv.into();
        let n0 = self.surface0.normal(u0, v0);

        let (u1, v1) = cc.contact_point1.uv.into();
        let n1 = self.surface1.normal(u1, v1);

        let sign = f64::signum(n0.cross(n1).dot(der));

        let dr = self.radius.der(cc.t);

        let mat = Matrix3::from_cols(der, n0, n1).transpose();
        let vec = Vector3::new(
            der.magnitude2() + der2.dot(p - cc.center),
            -sign * dr,
            -sign * dr,
        );

        mat.invert().unwrap() * vec
    }

    /// the derivation of the orbit of the `contact_point0`
    #[inline]
    pub fn contact_point0_der(&self, cc: ContactCircle) -> Vector3 {
        sub_contact_point_der(
            &self.surface0,
            cc.contact_point0,
            cc.center,
            self.center_der(cc),
        )
    }

    /// the derivation of the orbit of the `contact_point1`
    #[inline]
    pub fn contact_point1_der(&self, cc: ContactCircle) -> Vector3 {
        sub_contact_point_der(
            &self.surface1,
            cc.contact_point1,
            cc.center,
            self.center_der(cc),
        )
    }

    /// the derivation of the axis of rotation.
    pub fn axis_der(&self, cc: ContactCircle) -> Vector3 {
        let c = cc.center;
        let p0 = cc.contact_point0.point;
        let p1 = cc.contact_point1.point;
        let dc = self.center_der(cc);
        let dp0 = self.contact_point0_der(cc);
        let dp1 = self.contact_point1_der(cc);

        let n = (p0 - c).cross(p1 - c);
        let dn = (dp0 - dc).cross(p1 - c) + (p0 - c).cross(dp1 - dc);
        let n_abs = n.magnitude();
        dn / n_abs - n * dn.dot(n) / n_abs.powi(3)
    }

    /// deriveation of angle
    pub fn angle_der(&self, cc: ContactCircle) -> f64 {
        let c = cc.center;
        let p0 = cc.contact_point0.point;
        let p1 = cc.contact_point1.point;
        let dc = self.center_der(cc);
        let dp0 = self.contact_point0_der(cc);
        let dp1 = self.contact_point1_der(cc);
        let r = p0.distance(c);
        let dr = self.radius.der(cc.t);

        let (cp0, cp1) = (p0 - c, p1 - c);
        let (dcp0, dcp1) = (dp0 - dc, dp1 - dc);
        (2.0 * dr / r * cp0.dot(cp1) - dcp0.dot(cp1) - cp0.dot(dcp1)) / cp0.cross(cp1).magnitude()
    }

    pub(super) fn vder(&self, u: f64, cc: ContactCircle) -> Vector3 {
        let cp0 = cc.contact_point0.point - cc.center;
        let rot = Matrix3::from_axis_angle(cc.axis, cc.angle * u);
        let dc = self.center_der(cc);
        let dp0 = self.contact_point0_der(cc);
        let daxis = self.axis_der(cc);
        let dangle = self.angle_der(cc) * u;
        let dm = rot_der(cc.axis, cc.angle.0 * u);
        let drot = daxis.x * dm[0] + daxis.y * dm[1] + daxis.z * dm[2] + dangle * dm[3];
        dc + drot * cp0 + rot * (dp0 - dc)
    }

    pub(super) fn u_parameter_division(
        &self,
        ((u0, u1), (v0, v1)): ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> Option<Vec<f64>> {
        const N: usize = 10;
        let n_f = (0..=N).try_fold(0.0, |n_max, i| {
            let v = v0 + (v1 - v0) * i as f64 / N as f64;
            let cc = self.contact_circle(v)?;
            let r = (cc.contact_point0.point - cc.center).magnitude();
            let n_f = cc.angle.0 / (2.0 * f64::acos(1.0 - tol / r));
            Some(f64::max(n_f, n_max))
        })?;
        let u_delta = u1 - u0;
        let n = f64::ceil(u_delta * n_f) as usize;
        let closure = move |i: usize| u0 + u_delta * i as f64 / n as f64;
        Some((0..=n).map(closure).collect())
    }
}

fn sub_contact_point_der(
    surface: &impl ParametricSurface3D,
    contact_point: ContactPoint,
    center: Point3,
    center_der: Vector3,
) -> Vector3 {
    let (u, v) = contact_point.uv.into();
    let uder = surface.uder(u, v);
    let vder = surface.vder(u, v);
    let uuder = surface.uuder(u, v);
    let uvder = surface.uvder(u, v);
    let vvder = surface.vvder(u, v);
    let h = contact_point.point - center;

    let uu = h.dot(uuder) + uder.dot(uder);
    let uv = h.dot(uvder) + uder.dot(vder);
    let vv = h.dot(vvder) + vder.dot(vder);
    let mat = Matrix2::new(uu, uv, uv, vv);
    let dc = center_der;
    let vec = Vector2::new(uder.dot(dc), vder.dot(dc));
    let duv = mat.invert().unwrap() * vec;
    duv.x * uder + duv.y * vder
}

fn rot_der(axis: Vector3, angle: f64) -> [Matrix3; 4] {
    let (s, c) = angle.sin_cos();
    let k = 1.0 - c;
    [
        Matrix3::new(
            2.0 * axis.x * k,
            axis.y * k,
            axis.z * k,
            axis.y * k,
            0.0,
            s,
            axis.z * k,
            -s,
            0.0,
        ),
        Matrix3::new(
            0.0,
            axis.x * k,
            -s,
            axis.x * k,
            2.0 * axis.y * k,
            axis.z * k,
            s,
            axis.z * k,
            0.0,
        ),
        Matrix3::new(
            0.0,
            s,
            axis.x * k,
            -s,
            0.0,
            axis.y * k,
            axis.x * k,
            axis.y * k,
            2.0 * axis.z * k,
        ),
        Matrix3::new(
            (axis.x * axis.x - 1.0) * s,
            axis.x * axis.y * s + axis.z * c,
            axis.z * axis.x * s - axis.y * c,
            axis.x * axis.y * s - axis.z * c,
            (axis.y * axis.y - 1.0) * s,
            axis.y * axis.z * s + axis.x * c,
            axis.z * axis.x * s + axis.y * c,
            axis.y * axis.z * s - axis.x * c,
            (axis.z * axis.z - 1.0) * s,
        ),
    ]
}

#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
proptest! {
    #[test]
    fn rot_der_test(
        (u, v) in (0.0f64..2.0 * PI, -1.0f64..=1.0),
        angle in 0.0f64..2.0 * PI
    ) {
        const EPS: f64 = 1.0e-4;
        let (r, z) = ((1.0 - v * v).sqrt(), v);
        let (s, c) = u.sin_cos();
        let axis = Vector3::new(r * c, r * s, z);

        let [dm0, dm1, dm2, dm3] = rot_der(axis, angle);
        let ans0 = (Matrix3::from_axis_angle(axis + EPS * Vector3::unit_x(), Rad(angle))
            - Matrix3::from_axis_angle(axis - EPS * Vector3::unit_x(), Rad(angle)))
            / (2.0 * EPS);
        let ans1 = (Matrix3::from_axis_angle(axis + EPS * Vector3::unit_y(), Rad(angle))
            - Matrix3::from_axis_angle(axis - EPS * Vector3::unit_y(), Rad(angle)))
            / (2.0 * EPS);
        let ans2 = (Matrix3::from_axis_angle(axis + EPS * Vector3::unit_z(), Rad(angle))
            - Matrix3::from_axis_angle(axis - EPS * Vector3::unit_z(), Rad(angle)))
            / (2.0 * EPS);
        let ans3 = (Matrix3::from_axis_angle(axis, Rad(angle + EPS))
            - Matrix3::from_axis_angle(axis, Rad(angle - EPS)))
            / (2.0 * EPS);
        (0..3).for_each(|i| {
            assert!((dm0[i] - ans0[i]).magnitude() < EPS);
            assert!((dm1[i] - ans1[i]).magnitude() < EPS);
            assert!((dm2[i] - ans2[i]).magnitude() < EPS);
            assert!((dm3[i] - ans3[i]).magnitude() < EPS);
        });
    }
}

pub(super) fn v_parameter_division_for_fillet<S>(
    surface: &S,
    udiv: &[f64],
    vdiv: &mut Vec<f64>,
    tol: f64,
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

            *ub = *ub || far;
            *vb = *vb || far;
        }
    }

    let mut new_vdiv = vec![vdiv[0]];
    for (v, vb) in vdiv.windows(2).zip(divide_flag1) {
        if vb {
            new_vdiv.push((v[0] + v[1]) / 2.0);
        }
        new_vdiv.push(v[1]);
    }

    if vdiv.len() != new_vdiv.len() {
        *vdiv = new_vdiv;
        v_parameter_division_for_fillet(surface, udiv, vdiv, tol);
    }
}

impl<C, S0, S1, R> RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    pub(super) fn search_contact_curve0_parameter(
        &self,
        point: Point3,
        hint: impl Into<SPHint1D>,
        trials: usize,
        orientation: bool,
    ) -> Option<f64> {
        use truck_base::newton::{self, CalcOutput};
        let RbfSurface {
            edge_curve,
            surface0,
            surface1,
            radius,
        } = &self;

        let t0 = edge_curve.search_nearest_parameter(point, hint, trials)?;
        let p0 = edge_curve.subs(t0);
        let ((u00, v00), (u10, v10)) = (
            surface0.search_parameter(p0, None, trials)?,
            surface1.search_parameter(p0, None, trials)?,
        );
        let (n0, n1) = (surface0.normal(u00, v00), surface1.normal(u10, v10));
        let sign = -f64::signum(n0.cross(n1).dot(edge_curve.der(t0)))
            * if orientation { 1.0 } else { -1.0 };

        let (u0, v0) = surface0.search_parameter(point, (u00, v00), trials)?;
        let n = sign * surface0.normal(u0, v0);

        let function = |t: f64| {
            let p = edge_curve.subs(t);
            let der = edge_curve.der(t);
            let der2 = edge_curve.der2(t);
            let r = radius.subs(t);
            let r_der = radius.der(t);
            let po = point + r * n - p;
            CalcOutput {
                value: der.dot(po),
                derivation: der2.dot(po) + der.dot(r_der * n - der),
            }
        };
        let t = newton::solve(function, t0, trials).ok()?;

        let r = radius.subs(t);
        let o = point + r * n;
        let (u1, v1) = surface1.search_nearest_parameter(o, (u10, v10), trials)?;
        let dist2 = surface1.subs(u1, v1).distance2(o);
        match dist2.near(&(r * r)) {
            true => Some(t),
            false => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PointVector3(Point3, Vector3);

impl Mul<PointVector3> for Matrix3 {
    type Output = PointVector3;
    #[inline(always)]
    fn mul(self, rhs: PointVector3) -> Self::Output {
        PointVector3(self.transform_point(rhs.0), self * rhs.1)
    }
}

#[derive(Clone, Copy, Debug)]
struct VectorVector3(Vector3, Vector3);

impl Mul<VectorVector3> for Matrix3 {
    type Output = VectorVector3;
    #[inline(always)]
    fn mul(self, rhs: VectorVector3) -> Self::Output { VectorVector3(self * rhs.0, self * rhs.1) }
}

impl std::ops::Sub<PointVector3> for PointVector3 {
    type Output = VectorVector3;
    #[inline(always)]
    fn sub(self, rhs: PointVector3) -> Self::Output {
        VectorVector3(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<C, S0, S1, R> SearchParameter<D2> for RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3> + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3> + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let Self {
            edge_curve,
            surface0,
            surface1,
            radius,
        } = self;
        let curve_hint = match hint.into() {
            SPHint2D::Parameter(_, v) => SPHint1D::Parameter(v),
            SPHint2D::Range(_, (v0, v1)) => SPHint1D::Range(v0, v1),
            SPHint2D::None => SPHint1D::None,
        };
        let mut t = edge_curve.search_nearest_parameter(point, curve_hint, trials)?;
        let mut c = edge_curve.subs(t);
        let ((mut u0, mut v0), (mut u1, mut v1)) = (
            surface0.search_nearest_parameter(point, None, trials)?,
            surface1.search_nearest_parameter(point, None, trials)?,
        );
        let (mut p0, mut p1) = (surface0.subs(u0, v0), surface1.subs(u1, v1));

        let (n0, n1) = (surface0.normal(u0, v0), surface1.normal(u1, v1));
        let sign = -f64::signum(n0.cross(n1).dot(edge_curve.der(t)));

        let _ = (0..=trials).find_map(|_i| {
            let (p, der, der2) = (edge_curve.subs(t), edge_curve.der(t), edge_curve.der2(t));
            let (r, r_der) = (sign * radius.subs(t), sign * radius.der(t));
            let (uder0, vder0) = (surface0.uder(u0, v0), surface0.vder(u0, v0));
            let (uder1, vder1) = (surface1.uder(u1, v1), surface1.vder(u1, v1));
            let (n0, n1) = (surface0.normal(u0, v0), surface1.normal(u1, v1));

            let (pp0, pp1, pc) = (p0 - point, p1 - point, c - point);

            if pp0.so_small() {
                let cc0 = self.contact_curve0();
                t = cc0.search_parameter(point, t, trials)?;
                c = p0 + sign * radius.subs(t) * n0;
                (u1, v1) = surface1.search_nearest_parameter(c, (u1, v1), trials)?;
                p1 = surface1.subs(u1, v1);
                return Some(())
            }
            if pp1.so_small() {
                let cc1 = self.contact_curve1();
                t = cc1.search_parameter(point, t, trials)?;
                c = p1 + sign * radius.subs(t) * n1;
                (u0, v0) = surface0.search_nearest_parameter(c, (u0, v0), trials)?;
                p0 = surface0.subs(u0, v0);
                return Some(())
            }
            let center_contact0 = (p0 + r * n0).near(&c);
            let center_contact1 = (p1 + r * n1).near(&c);
            let same_plane = pp0.cross(pp1).dot(pc).so_small2();
            if center_contact0 && center_contact1 && same_plane {
                return Some(());
            }

            let c_next = {
                let mat = Matrix3::from_cols(der, n0, n1).transpose();
                let c_next0 = Point3::new(
                    der.dot(p.to_vec()),
                    n0.dot(p0.to_vec()) + r,
                    n1.dot(p1.to_vec()) + r,
                );
                let c_next1 = Vector3::new(der.dot(der) - der2.dot(pc), r_der, r_der);
                mat.invert().unwrap() * PointVector3(c_next0, c_next1)
            };

            let duv0 = {
                let mat = Matrix3::from_cols(
                    uder0 + r * surface0.normal_uder(u0, v0),
                    vder0 + r * surface0.normal_vder(u0, v0),
                    n0,
                );
                mat.invert().unwrap() * (c_next - PointVector3(p0 + r * n0, r_der * n0))
            };
            debug_assert!(duv0.0.z.so_small() && duv0.1.z.so_small());

            let duv1 = {
                let mat = Matrix3::from_cols(
                    uder1 + r * surface1.normal_uder(u1, v1),
                    vder1 + r * surface1.normal_vder(u1, v1),
                    n1,
                );
                mat.invert().unwrap() * (c_next - PointVector3(p1 + r * n1, r_der * n1))
            };
            debug_assert!(duv1.0.z.so_small() && duv1.1.z.so_small());

            let dp0 = Matrix3::from_cols(uder0, vder0, n0) * duv0;
            let dp1 = Matrix3::from_cols(uder1, vder1, n1) * duv1;

            let x = Matrix3::from_cols(dp0.0, pp1, pc).determinant()
                + Matrix3::from_cols(pp0, dp1.0, pc).determinant()
                + Matrix3::from_cols(pp0, pp1, c_next.0 - point).determinant();
            let y = Matrix3::from_cols(dp0.1, pp1, pc).determinant()
                + Matrix3::from_cols(pp0, dp1.1, pc).determinant()
                + Matrix3::from_cols(pp0, pp1, c_next.1).determinant();
            let dt = -x / y;

            t += dt;
            c = c_next.0 + c_next.1 * dt;
            let duv0 = duv0.0 + duv0.1 * dt;
            (u0, v0) = (u0 + duv0.x, v0 + duv0.y);
            let duv1 = duv1.0 + duv1.1 * dt;
            (u1, v1) = (u1 + duv1.x, v1 + duv1.y);
            (p0, p1) = (surface0.subs(u0, v0), surface1.subs(u1, v1));
            None
        })?;

        let (cp0, cp1, cp) = (p0 - c, p1 - c, point - c);
        let theta = cp.angle(cp0);
        let rot = Matrix3::from_axis_angle(cp0.cross(cp1).normalize(), theta);
        match (rot * cp0).near(&cp) {
            true => Some((theta.0 / cp0.angle(cp1).0, t)),
            false => None,
        }
    }
}
