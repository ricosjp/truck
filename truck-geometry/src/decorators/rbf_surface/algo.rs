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
    /// use truck_geometry::{
    ///     prelude::*,
    ///     decorators::rbf_surface::ContactCircle,
    /// };
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

    let mat = Matrix2::new(
        h.dot(uuder) + uder.dot(uder),
        h.dot(uvder) + uder.dot(vder),
        h.dot(uvder) + uder.dot(vder),
        h.dot(vvder) + vder.dot(vder),
    );
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
