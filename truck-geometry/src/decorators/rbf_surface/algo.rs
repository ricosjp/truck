use super::*;
const MUL: fn(f64, f64) -> f64 = f64::mul;
const DOT: fn(Vector3, Vector3) -> f64 = Vector3::dot;
const CROSS: fn(Vector3, Vector3) -> Vector3 = Vector3::cross;

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

    pub(super) fn center_contacts_ders(
        &self,
        cc: ContactCircle,
        rders: &[f64],
        n: usize,
    ) -> CenterContactDers {
        let Point2 { x: u0, y: v0 } = cc.contact_point0.uv;
        let mut s0info = SurfaceInfo::new(&self.surface0, (u0, v0), n + 1);
        let Point2 { x: u1, y: v1 } = cc.contact_point1.uv;
        let mut s1info = SurfaceInfo::new(&self.surface1, (u1, v1), n + 1);

        let mut cders = [Vector3::zero(); 32];
        self.edge_curve.ders(cc.t, &mut cders[..=n + 1]);

        let mut ders = [Vector3::zero(); 32];
        ders[0] = cc.center.to_vec();

        (1..=n).for_each(|m| der_routine(&mut s0info, &mut s1info, &cders, rders, &mut ders, m));

        CenterContactDers {
            center_ders: ders,
            contact0_ders: s0info.tders,
            contact1_ders: s1info.tders,
        }
    }

    pub(super) fn vder_info(&self, cc: ContactCircle, n: usize) -> VderInfo {
        let mut rders = [0.0; 32];
        self.radius.ders(cc.t, &mut rders[..=n]);
        let cc_ders = self.center_contacts_ders(cc, &rders, n);
        cc_ders.vder_info(&rders, n)
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

#[derive(Clone, Copy, Debug)]
pub(super) struct CenterContactDers {
    pub center_ders: [Vector3; 32],
    pub contact0_ders: [Vector3; 32],
    pub contact1_ders: [Vector3; 32],
}

impl CenterContactDers {
    fn cross_ders(&self, n: usize) -> [Vector3; 32] {
        let mut cross_ders = [Vector3::zero(); 32];
        let f0 = |i| self.contact0_ders[i] - self.center_ders[i];
        let f1 = |i| self.contact1_ders[i] - self.center_ders[i];
        cross_ders[..=n]
            .iter_mut()
            .enumerate()
            .for_each(|(i, o)| *o = comp_sum(i, f0, f1, CROSS));
        cross_ders
    }
    fn axis_ders(&self, cross_ders: &[Vector3], abs_cross_ders: &[f64], n: usize) -> [Vector3; 32] {
        let mut homog = [Vector4::zero(); 32];
        cross_ders[..=n]
            .iter()
            .zip(abs_cross_ders)
            .zip(&mut homog)
            .for_each(|((c, &a), o)| *o = Vector4::new(c.x, c.y, c.z, a));
        let mut res = [Vector3::zero(); 32];
        rat_ders(&homog, &mut res);
        res
    }
    fn angle_ders(&self, abs_cross_ders: &[f64], rders: &[f64], n: usize) -> [f64; 32] {
        let mut r_rat_ders = [0.0; 32];
        r_rat_ders[0] = rders[1] / rders[0];
        (1..=n).for_each(|m| {
            let sum = comp_sum(m, |i| rders[i], |i| r_rat_ders[i], MUL);
            r_rat_ders[m] = (rders[m + 1] - sum) / rders[0];
        });

        let cp0 = |i| self.contact0_ders[i] - self.center_ders[i];
        let cp1 = |i| self.contact1_ders[i] - self.center_ders[i];
        let mut dot_ders = [0.0; 32];
        dot_ders[..=n]
            .iter_mut()
            .enumerate()
            .for_each(|(m, o)| *o = comp_sum(m, cp0, cp1, DOT));

        let mut angle_ders = [0.0; 32];
        angle_ders[0] = f64::acos(f64::min(cp0(0).dot(cp1(0)) / (rders[0] * rders[0]), 1.0));
        (1..=n).for_each(|m| {
            let sum0 = comp_sum(m - 1, |i| abs_cross_ders[i], |i| angle_ders[i + 1], MUL);
            let sum1 = comp_sum(m - 1, |i| r_rat_ders[i], |i| dot_ders[i], MUL);
            let sum2 = comp_sum(m, cp0, cp1, DOT);
            angle_ders[m] = (2.0 * sum1 - sum2 - sum0) / abs_cross_ders[0]
        });
        angle_ders
    }
    pub(super) fn vder_info(self, rders: &[f64], n: usize) -> VderInfo {
        let cross_ders = self.cross_ders(n);
        let mut abs_cross_ders = [0.0; 32];
        abs_ders(&cross_ders[..=n], &mut abs_cross_ders);
        VderInfo {
            axis_ders: self.axis_ders(&cross_ders, &abs_cross_ders, n),
            angle_ders: self.angle_ders(&abs_cross_ders, rders, n),
            cc_ders: self,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct VderInfo {
    cc_ders: CenterContactDers,
    axis_ders: [Vector3; 32],
    angle_ders: [f64; 32],
}

impl VderInfo {
    pub(super) fn vder(&self, u: f64) -> Vector3 {
        let VderInfo {
            cc_ders,
            axis_ders,
            angle_ders,
        } = self;
        let cp0 = cc_ders.contact0_ders[0] - cc_ders.center_ders[0];
        let dp0 = cc_ders.contact0_ders[1];
        let dc = cc_ders.center_ders[1];
        let rot = Matrix3::from_axis_angle(axis_ders[0], Rad(angle_ders[0] * u));
        let dm = std::array::from_fn::<_, 4, _>(|index| {
            let mut orders = [0; 4];
            orders[index] = 1;
            rot_der_n(orders, axis_ders[0], angle_ders[0] * u)
        });
        let daxis = axis_ders[1];
        let dangle = angle_ders[1] * u;
        let drot = daxis.x * dm[0] + daxis.y * dm[1] + daxis.z * dm[2] + dangle * dm[3];
        dc + drot * cp0 + rot * (dp0 - dc)
    }
    pub(super) fn uvder(&self, u: f64) -> Vector3 {
        let VderInfo {
            cc_ders,
            axis_ders,
            angle_ders,
        } = self;

        let (axis, daxis) = (axis_ders[0], axis_ders[1]);
        let (angle, dangle) = (angle_ders[0], angle_ders[1]);
        let rot_u = angle * rot_der_n([0, 0, 0, 1], axis, angle * u);
        let rot_uv = dangle * rot_der_n([0, 0, 0, 1], axis, angle * u)
            + angle * dangle * u * rot_der_n([0, 0, 0, 2], axis, angle * u);
        let rot_axis = angle
            * (daxis.x * rot_der_n([1, 0, 0, 1], axis, angle * u)
                + daxis.y * rot_der_n([0, 1, 0, 1], axis, angle * u)
                + daxis.z * rot_der_n([0, 0, 1, 1], axis, angle * u));

        let cp0 = cc_ders.contact0_ders[0] - cc_ders.center_ders[0];
        let dcp0 = cc_ders.contact0_ders[1] - cc_ders.center_ders[1];
        (rot_axis + rot_uv) * cp0 + rot_u * dcp0
    }
    pub(super) fn vvder(&self, u: f64) -> Vector3 {
        let VderInfo {
            cc_ders,
            axis_ders,
            angle_ders,
        } = self;
        let ddc = cc_ders.center_ders[2];
        let cp0 = cc_ders.contact0_ders[0] - cc_ders.center_ders[0];
        let dcp0 = cc_ders.contact0_ders[1] - cc_ders.center_ders[1];
        let ddcp0 = cc_ders.contact0_ders[2] - cc_ders.center_ders[2];

        let (axis, angle) = (axis_ders[0], angle_ders[0] * u);
        let aa_ders = std::array::from_fn::<_, 3, _>(|i| axis_ders[i].extend(angle_ders[i] * u));
        let coef0 = (0..4).fold(Matrix3::zero(), |sum, i| {
            let mut order = [0; 4];
            order[i] = 1;
            sum + aa_ders[2][i] * rot_der_n(order, axis, angle)
        });
        let coef1 = (0..4).flat_map(|i| (0..4).map(move |j| (i, j))).fold(
            Matrix3::zero(),
            |sum, (i, j)| {
                let mut order = [0; 4];
                order[i] += 1;
                order[j] += 1;
                sum + aa_ders[1][i] * aa_ders[1][j] * rot_der_n(order, axis, angle)
            },
        );
        let coef2 = (0..4).fold(Matrix3::zero(), |sum, i| {
            let mut order = [0; 4];
            order[i] = 1;
            sum + aa_ders[1][i] * rot_der_n(order, axis, angle)
        });

        ddc + (coef0 + coef1) * cp0
            + 2.0 * coef2 * dcp0
            + Matrix3::from_axis_angle(axis, Rad(angle)) * ddcp0
    }
}

#[derive(Clone, Copy, Debug)]
struct SurfaceInfo {
    ders: [[Vector3; 32]; 32],
    tders: [Vector3; 32],
    uderders: [Vector3; 32],
    vderders: [Vector3; 32],
    crossders: [Vector3; 32],
    abs_crossders: [f64; 32],
    nders: [Vector3; 32],
    uvders: [Vector2; 32],
}

impl SurfaceInfo {
    fn new(surface: &impl ParametricSurface3D, (u, v): (f64, f64), n: usize) -> Self {
        let mut ders = [[Vector3::zero(); 32]; 32];
        let mut ders_mut = ders[..=n]
            .iter_mut()
            .enumerate()
            .map(|(i, slice)| &mut slice[..=n - i])
            .collect::<Vec<_>>();
        surface.ders(u, v, &mut ders_mut);

        let mut tders = [Vector3::zero(); 32];
        tders[0] = ders[0][0];

        let mut uderders = [Vector3::zero(); 32];
        uderders[0] = ders[1][0];

        let mut vderders = [Vector3::zero(); 32];
        vderders[0] = ders[0][1];

        let mut crossders = [Vector3::zero(); 32];
        crossders[0] = uderders[0].cross(vderders[0]);

        let mut abs_crossders = [0.0; 32];
        abs_crossders[0] = crossders[0].magnitude();

        let mut nders = [Vector3::zero(); 32];
        nders[0] = surface.normal(u, v);

        let mut uvders = [Vector2::zero(); 32];
        uvders[0] = Vector2::new(u, v);

        Self {
            ders,
            tders,
            uderders,
            vderders,
            crossders,
            abs_crossders,
            nders,
            uvders,
        }
    }

    fn routine(&mut self, ders: &[Vector3], n: usize) {
        let SurfaceInfo {
            ders: ref sders,
            uvders,
            tders,
            uderders,
            vderders,
            crossders,
            abs_crossders,
            nders,
        } = self;
        let sder_n_prime = pcurve::raw_der_n(sders, uvders, n);

        let mut lhs_u = comp_sum(n, |i| uderders[i], |i| tders[i] - ders[i], DOT);
        let uder_n_prime = pcurve::raw_der_n(&sders[1..], uvders, n);
        lhs_u += uderders[0].dot(sder_n_prime) + uder_n_prime.dot(tders[0] - ders[0]);

        let mut lhs_v = comp_sum(n, |i| vderders[i], |i| tders[i] - ders[i], DOT);
        let vders = sders
            .iter()
            .map(|vec| &vec.as_ref()[1..])
            .collect::<Vec<_>>();
        let vder_n_prime = pcurve::raw_der_n(&vders, uvders, n);
        lhs_v += vderders[0].dot(sder_n_prime) + vder_n_prime.dot(tders[0] - ders[0]);

        let cp = tders[0] - ders[0];
        let uu = sders[1].as_ref()[0].magnitude2() + sders[2].as_ref()[0].dot(cp);
        let uv = sders[1].as_ref()[0].dot(sders[0].as_ref()[1]) + sders[1].as_ref()[1].dot(cp);
        let vv = sders[0].as_ref()[1].magnitude2() + sders[0].as_ref()[2].dot(cp);
        let mat = Matrix2::new(uu, uv, uv, vv);
        uvders[n] = -mat.invert().unwrap() * Vector2::new(lhs_u, lhs_v);
        tders[n] = pcurve::raw_der_n(sders, uvders, n);
        uderders[n] = pcurve::raw_der_n(&sders[1..], uvders, n);
        vderders[n] = pcurve::raw_der_n(&vders, uvders, n);

        crossders[n] = comp_sum(n, |i| uderders[i], |i| vderders[i], CROSS);
        let sum = comp_sum(n - 1, |i| crossders[i + 1], |i| crossders[i], DOT)
            - comp_sum(n - 1, |i| abs_crossders[i + 1], |i| abs_crossders[i], MUL);
        abs_crossders[n] = sum / abs_crossders[0];
        let homog = crossders[..=n]
            .iter()
            .zip(&abs_crossders[..=n])
            .map(|(v, &w)| v.extend(w))
            .collect::<Vec<_>>();
        nders[n] = rat_der(&homog);
    }
}

fn der_routine(
    s0info: &mut SurfaceInfo,
    s1info: &mut SurfaceInfo,
    cders: &[Vector3],
    rders: &[f64],
    ders: &mut [Vector3],
    n: usize,
) {
    let (n0ders, n1ders) = (&s0info.nders, &s1info.nders);
    let mat = Matrix3::from_cols(cders[1], n0ders[0], n1ders[0]).transpose();
    let b = Vector3::new(
        comp_sum(n, |i| cders[i + 1], |i| cders[i] - ders[i], DOT),
        rders[n] - comp_sum(n - 1, |i| ders[i + 1], |i| n0ders[i], DOT),
        rders[n] - comp_sum(n - 1, |i| ders[i + 1], |i| n1ders[i], DOT),
    );
    ders[n] = mat.invert().unwrap() * b;

    s0info.routine(ders, n);
    s1info.routine(ders, n);
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

#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
proptest! {
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

#[test]
fn fillet_between_two_spheres_deralgo() {
    let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 1.0), 2.0);
    let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -1.0), 2.0);
    let edge_circle = Processor::with_transform(
        UnitCircle::<Point3>::new(),
        Matrix4::from_scale(f64::sqrt(3.0)),
    );

    #[derive(Clone, Copy, Debug)]
    struct Radius;
    impl RadiusFunction for Radius {
        fn der_n(&self, n: usize, t: f64) -> f64 {
            let o = if n == 0 { 1.0 } else { 0.0 };
            let x = match n % 4 {
                0 => f64::cos(t),
                1 => -f64::sin(t),
                2 => -f64::cos(t),
                _ => f64::sin(t),
            };
            o + 0.2 * x
        }
    }

    let fillet = RbfSurface::new(edge_circle, sphere0, sphere1, Radius);

    const N: usize = 20;
    for i in 1..N {
        let t = 2.0 * PI * i as f64 / N as f64;
        let cc = fillet.contact_circle(t).unwrap();

        let mut rders = [0.0; 6];
        fillet.radius.ders(t, &mut rders);

        let eps = 1.0e-4;
        let cc_plus = fillet.contact_circle(t + eps).unwrap();
        let mut rders_plus = [0.0; 5];
        fillet.radius.ders(t + eps, &mut rders_plus);

        let cc_minus = fillet.contact_circle(t - eps).unwrap();
        let mut rders_minus = [0.0; 5];
        fillet.radius.ders(t - eps, &mut rders_minus);

        let cc_ders = fillet.center_contacts_ders(cc, &rders, 4);
        let cc_ders_plus = fillet.center_contacts_ders(cc_plus, &rders_plus, 3);
        let cc_ders_minus = fillet.center_contacts_ders(cc_minus, &rders_minus, 3);

        let cross_ders = cc_ders.cross_ders(4);
        let cross_ders_plus = cc_ders_plus.cross_ders(3);
        let cross_ders_minus = cc_ders_minus.cross_ders(3);

        let mut abs_cross_ders = [0.0; 5];
        abs_ders(&cross_ders[..=4], &mut abs_cross_ders);
        let mut abs_cross_ders_plus = [0.0; 4];
        abs_ders(&cross_ders_plus[..=3], &mut abs_cross_ders_plus);
        let mut abs_cross_ders_minus = [0.0; 4];
        abs_ders(&cross_ders_minus[..=3], &mut abs_cross_ders_minus);

        let axis_ders = cc_ders.axis_ders(&cross_ders, &abs_cross_ders, 4);
        let axis_ders_plus = cc_ders_plus.axis_ders(&cross_ders_plus, &abs_cross_ders_plus, 3);
        let axis_ders_minus = cc_ders_minus.axis_ders(&cross_ders_minus, &abs_cross_ders_minus, 3);

        let angle_ders = cc_ders.angle_ders(&abs_cross_ders, &rders, 4);
        let angle_ders_plus = cc_ders_plus.angle_ders(&abs_cross_ders_plus, &rders_plus, 3);
        let angle_ders_minus = cc_ders_minus.angle_ders(&abs_cross_ders_minus, &rders_minus, 3);

        for m in 0..=2 {
            let center_der_approx =
                (cc_ders_plus.center_ders[m] - cc_ders_minus.center_ders[m]) / (2.0 * eps);
            assert!(
                (cc_ders.center_ders[m + 1] - center_der_approx).magnitude() < eps,
                "m = {m}: {:?} {:?}",
                cc_ders.center_ders[m + 1],
                center_der_approx,
            );

            let contact0_der_approx =
                (cc_ders_plus.contact0_ders[m] - cc_ders_minus.contact0_ders[m]) / (2.0 * eps);
            assert!(
                (cc_ders.contact0_ders[m + 1] - contact0_der_approx).magnitude() < eps,
                "m = {m}: {:?} {:?}",
                cc_ders.contact0_ders[m + 1],
                contact0_der_approx,
            );

            let contact1_der_approx =
                (cc_ders_plus.contact1_ders[m] - cc_ders_minus.contact1_ders[m]) / (2.0 * eps);
            assert!(
                (cc_ders.contact1_ders[m + 1] - contact1_der_approx).magnitude() < eps,
                "m = {m}: {:?} {:?}",
                cc_ders.contact1_ders[m + 1],
                contact1_der_approx,
            );

            let cross_der_approx = (cross_ders_plus[m] - cross_ders_minus[m]) / (2.0 * eps);
            assert!(
                (cross_ders[m + 1] - cross_der_approx).magnitude() < eps,
                "m = {m}: {:?} {:?}",
                cross_ders[m + 1],
                cross_der_approx,
            );

            let axis_der_approx = (axis_ders_plus[m] - axis_ders_minus[m]) / (2.0 * eps);
            assert!(
                (axis_ders[m + 1] - axis_der_approx).magnitude() < eps,
                "m = {m}: {:?} {:?}",
                axis_ders[m + 1],
                axis_der_approx,
            );

            let angle_der_approx = (angle_ders_plus[m] - angle_ders_minus[m]) / (2.0 * eps);
            assert!(
                (angle_ders[m + 1] - angle_der_approx).abs() < eps,
                "m = {m}: {:?} {:?}",
                angle_ders[m + 1],
                angle_der_approx,
            );
        }
    }
}
