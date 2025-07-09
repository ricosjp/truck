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

fn der_routine<A: AsRef<[Vector3]>>(
    s0ders: &[A],
    s0tders: &mut [Vector3],
    s0uderders: &mut [Vector3],
    s0vderders: &mut [Vector3],
    cross0ders: &mut [Vector3],
    abs_cross0ders: &mut [f64],
    n0ders: &mut [Vector3],
    uv0ders: &mut [Vector2],
    s1ders: &[A],
    s1tders: &mut [Vector3],
    s1uderders: &mut [Vector3],
    s1vderders: &mut [Vector3],
    cross1ders: &mut [Vector3],
    abs_cross1ders: &mut [f64],
    n1ders: &mut [Vector3],
    uv1ders: &mut [Vector2],
    cders: &[Vector3],
    rders: &[f64],
    ders: &mut [Vector3],
    n: usize,
) {
    use pcurve::composition::*;
    let mut c = 1;
    let c_comp = (0..=n).fold(0.0, |sum, i| {
        let sum = sum + cders[i + 1].dot(cders[n - i] - ders[n - i]) * c as f64;
        c = c * (n - i) / (i + 1);
        sum
    });

    let mut c = 1;
    let n0_comp = rders[n]
        - (0..n).fold(0.0, |sum, i| {
            let sum = sum + ders[i + 1].dot(n0ders[n - 1 - i]) * c as f64;
            c = c * (n - 1 - i) / (i + 1);
            sum
        });

    let mut c = 1;
    let n1_comp = rders[n]
        - (0..n).fold(0.0, |sum, i| {
            let sum = sum + ders[i + 1].dot(n1ders[n - 1 - i]) * c as f64;
            c = c * (n - 1 - i) / (i + 1);
            sum
        });

    let mat = Matrix3::from_cols(cders[1], n0ders[0], n1ders[0]).transpose();
    ders[n] = mat.invert().unwrap() * Vector3::new(c_comp, n0_comp, n1_comp);

    let s0der_n_prime = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(s0ders, uv0ders, idx) * multiplicity(idx) as f64
        })
    });

    let mut c = 1;
    let mut lhs_u = (0..=n).fold(0.0, |sum, i| {
        let sum = sum + s0uderders[i].dot(s0tders[n - i] - ders[n - i]) * c as f64;
        c = c * (n - i) / (i + 1);
        sum
    });
    let s0uder_n_prime = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(&s0ders[1..], uv0ders, idx) * multiplicity(idx) as f64
        })
    });
    lhs_u += s0uderders[0].dot(s0der_n_prime - ders[n]) + s0uder_n_prime.dot(s0tders[0] - ders[0]);

    let mut c = 1;
    let mut lhs_v = (0..=n).fold(0.0, |sum, i| {
        let sum = sum + s0vderders[i].dot(s0tders[n - i] - ders[n - i]) * c as f64;
        c = c * (n - i) / (i + 1);
        sum
    });
    let s0vders = s0ders
        .iter()
        .map(|vec| &vec.as_ref()[1..])
        .collect::<Vec<_>>();
    let s0vder_n_prime = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(&s0vders, uv0ders, idx) * multiplicity(idx) as f64
        })
    });
    lhs_v += s0vderders[0].dot(s0der_n_prime - ders[n]) + s0vder_n_prime.dot(s0tders[0] - ders[0]);

    let mat = Matrix2::new(
        s0ders[1].as_ref()[0].dot(s0ders[n].as_ref()[0])
            + s0ders[n + 1].as_ref()[0].dot(s0ders[0].as_ref()[0] - ders[0]),
        s0ders[0].as_ref()[1].dot(s0ders[n].as_ref()[0])
            + s0ders[1].as_ref()[n].dot(s0ders[0].as_ref()[0] - ders[0]),
        s0ders[1].as_ref()[0].dot(s0ders[0].as_ref()[n])
            + s0ders[n].as_ref()[1].dot(s0ders[0].as_ref()[0] - ders[0]),
        s0ders[0].as_ref()[1].dot(s0ders[0].as_ref()[n])
            + s0ders[0].as_ref()[n + 1].dot(s0ders[0].as_ref()[0] - ders[0]),
    );
    uv0ders[n] = -mat.invert().unwrap() * Vector2::new(lhs_u, lhs_v);
    s0tders[n] = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(s0ders, uv0ders, idx) * multiplicity(idx) as f64
        })
    });
    s0uderders[n] = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(&s0ders[1..], uv0ders, idx) * multiplicity(idx) as f64
        })
    });
    s0vderders[n] = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(&s0vders, uv0ders, idx) * multiplicity(idx) as f64
        })
    });

    let mut c = 1;
    cross0ders[n] = (0..=n).fold(Vector3::zero(), |mut sum, i| {
        sum += s0uderders[i].cross(s0vderders[n - i]) * c as f64;
        c = c * (n - i) / (i + 1);
        sum
    });
    let mut c = 1;
    let sum0 = (0..n).fold(0.0, |mut sum, i| {
        let x = cross0ders[i + 1].dot(cross0ders[n - i - 1]);
        let y = abs_cross0ders[i + 1] * abs_cross0ders[n - i - 1];
        sum += (y - x) * c as f64;
        c = c * (n - i - 1) / (i + 1);
        sum
    });
    abs_cross0ders[n] = sum0 / abs_cross0ders[0];
    let homog0 = cross0ders[..=n]
        .iter()
        .zip(&abs_cross0ders[..=n])
        .map(|(v, w)| v.extend(*w))
        .collect::<Vec<_>>();
    n0ders[n] = rat_der(&homog0);

    let s1der_n_prime = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(s1ders, uv1ders, idx) * multiplicity(idx) as f64
        })
    });

    let mut c = 1;
    let mut lhs_u = (0..=n).fold(0.0, |sum, i| {
        let sum = sum + s1uderders[i].dot(s1tders[n - i] - ders[n - i]) * c as f64;
        c = c * (n - i) / (i + 1);
        sum
    });
    let s1uder_n_prime = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(&s1ders[1..], uv1ders, idx) * multiplicity(idx) as f64
        })
    });
    lhs_u += s1uderders[0].dot(s1der_n_prime - ders[n]) + s1uder_n_prime.dot(s1tders[0] - ders[0]);

    let mut c = 1;
    let mut lhs_v = (0..=n).fold(0.0, |sum, i| {
        let sum = sum + s1vderders[i].dot(s1tders[n - i] - ders[n - i]) * c as f64;
        c = c * (n - i) / (i + 1);
        sum
    });
    let s1vders = s1ders
        .iter()
        .map(|vec| &vec.as_ref()[1..])
        .collect::<Vec<_>>();
    let s1vder_n_prime = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(&s1vders, uv1ders, idx) * multiplicity(idx) as f64
        })
    });
    lhs_v += s1vderders[0].dot(s1der_n_prime - ders[n]) + s1vder_n_prime.dot(s1tders[0] - ders[0]);

    let mat = Matrix2::new(
        s1ders[1].as_ref()[0].dot(s1ders[n].as_ref()[0])
            + s1ders[n + 1].as_ref()[0].dot(s1ders[0].as_ref()[0] - ders[0]),
        s1ders[0].as_ref()[1].dot(s1ders[n].as_ref()[0])
            + s1ders[1].as_ref()[n].dot(s1ders[0].as_ref()[0] - ders[0]),
        s1ders[1].as_ref()[0].dot(s1ders[0].as_ref()[n])
            + s1ders[n].as_ref()[1].dot(s1ders[0].as_ref()[0] - ders[0]),
        s1ders[0].as_ref()[1].dot(s1ders[0].as_ref()[n])
            + s1ders[0].as_ref()[n + 1].dot(s1ders[0].as_ref()[0] - ders[0]),
    );
    uv1ders[n] = -mat.invert().unwrap() * Vector2::new(lhs_u, lhs_v);
    s1tders[n] = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(s1ders, uv1ders, idx) * multiplicity(idx) as f64
        })
    });
    s1uderders[n] = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(&s1ders[1..], uv1ders, idx) * multiplicity(idx) as f64
        })
    });
    s1vderders[n] = (1..=n).fold(Vector3::zero(), |sum, len| {
        let iter = CompositionIter::<32>::try_new(n, len).unwrap();
        iter.fold(sum, |sum, idx| {
            let idx = &idx[..len];
            sum + tensor(&s1vders, uv1ders, idx) * multiplicity(idx) as f64
        })
    });

    let mut c = 1;
    cross1ders[n] = (0..=n).fold(Vector3::zero(), |mut sum, i| {
        sum += s1uderders[i].cross(s1vderders[n - i]) * c as f64;
        c = c * (n - i) / (i + 1);
        sum
    });
    let mut c = 1;
    let sum0 = (0..n).fold(0.0, |mut sum, i| {
        let x = cross1ders[i + 1].dot(cross1ders[n - i - 1]);
        let y = abs_cross1ders[i + 1] * abs_cross1ders[n - i - 1];
        sum += (y - x) * c as f64;
        c = c * (n - i - 1) / (i + 1);
        sum
    });
    abs_cross1ders[n] = sum0 / abs_cross1ders[0];
    let homog0 = cross1ders[..=n]
        .iter()
        .zip(&abs_cross1ders[..=n])
        .map(|(v, w)| v.extend(*w))
        .collect::<Vec<_>>();
    n1ders[n] = rat_der(&homog0);
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

fn rot_der_n(orders: [usize; 4], axis: Vector3, angle: f64) -> Matrix3 {
    let (s, c) = (angle + PI / 2.0 * orders[3] as f64).sin_cos();
    let k = -c + if orders[3] == 0 { 1.0 } else { 0.0 };
    match orders {
        [0, 0, 0, _] => Matrix3::new(
            k * axis.x * axis.x + c,
            k * axis.x * axis.y + s * axis.z,
            k * axis.x * axis.z - s * axis.y,
            k * axis.x * axis.y - s * axis.z,
            k * axis.y * axis.y + c,
            k * axis.y * axis.z + s * axis.x,
            k * axis.x * axis.z + s * axis.y,
            k * axis.y * axis.z - s * axis.x,
            k * axis.z * axis.z + c,
        ),
        [1, 0, 0, _] => Matrix3::new(
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
        [0, 1, 0, _] => Matrix3::new(
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
        [0, 0, 1, _] => Matrix3::new(
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
        [2, 0, 0, _] => Matrix3::new(2.0 * k, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        [0, 2, 0, _] => Matrix3::new(0.0, 0.0, 0.0, 0.0, 2.0 * k, 0.0, 0.0, 0.0, 0.0),
        [0, 0, 2, _] => Matrix3::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0 * k),
        [0, 1, 1, _] => Matrix3::new(0.0, 0.0, 0.0, 0.0, 0.0, k, 0.0, k, 0.0),
        [1, 0, 1, _] => Matrix3::new(0.0, 0.0, k, 0.0, 0.0, 0.0, k, 0.0, 0.0),
        [1, 1, 0, _] => Matrix3::new(0.0, k, 0.0, k, 0.0, 0.0, 0.0, 0.0, 0.0),
        _ => Matrix3::zero(),
    }
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
    #[test]
    fn rot_der_n_test(
        (u, v) in (0.0f64..2.0 * PI, -1.0f64..=1.0),
        angle in 0.0f64..2.0 * PI,
        orders in prop::array::uniform4(0usize..=4usize),
        index in 0..4usize,
    ) {
        const EPS: f64 = 1.0e-4;
        let (r, z) = ((1.0 - v * v).sqrt(), v);
        let (s, c) = u.sin_cos();
        let axis = Vector3::new(r * c, r * s, z);

        let mut orders0 = orders;
        orders0[index] += 1;
        let mat0 = rot_der_n(orders0, axis, angle);

        let (mut axis_p, mut angle_p) = (axis, angle);
        if index < 3 {
            axis_p[index] += EPS;
        } else {
            angle_p += EPS;
        }
        let (mut axis_m, mut angle_m) = (axis, angle);
        if index < 3 {
            axis_m[index] -= EPS;
        } else {
            angle_m -= EPS;
        }
        let mat1 =
            (rot_der_n(orders, axis_p, angle_p) - rot_der_n(orders, axis_m, angle_m)) / (2.0 * EPS);

        prop_assert!((0..3).all(|i| (mat0[i] - mat1[i]).magnitude() < EPS));
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
