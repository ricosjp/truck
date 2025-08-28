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
    /// Calculate contact circle corresponding to the parameter `t`, i.e.
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

    /// Calculate higher-order derivatives of the center curve and the contact curves.
    pub fn center_contact_ders(&self, cc: ContactCircle, n: usize) -> CenterContactDers {
        let rders = self.radius.ders(n + 1, cc.t);
        self.sub_center_contacts_ders(cc, &rders, n)
    }

    pub(super) fn sub_center_contacts_ders(
        &self,
        cc: ContactCircle,
        rders: &CurveDers<f64>,
        n: usize,
    ) -> CenterContactDers {
        let Point2 { x: u0, y: v0 } = cc.contact_point0.uv;
        let mut s0info = SurfaceInfo::new(&self.surface0, (u0, v0), n + 1);
        let Point2 { x: u1, y: v1 } = cc.contact_point1.uv;
        let mut s1info = SurfaceInfo::new(&self.surface1, (u1, v1), n + 1);

        let cders = self.edge_curve.ders(n + 1, cc.t);

        let mut ders = CurveDers::new(n);
        ders[0] = cc.center.to_vec();

        (1..=n).for_each(|m| der_routine(&mut s0info, &mut s1info, &cders, rders, &mut ders, m));

        CenterContactDers {
            center_ders: ders,
            contact0_ders: s0info.tders,
            contact1_ders: s1info.tders,
            uv0_ders: s0info.uvders,
            uv1_ders: s1info.uvders,
        }
    }

    pub(super) fn vder_info(&self, cc: ContactCircle, n: usize) -> VderInfo {
        let rders = self.radius.ders(n + 1, cc.t);
        let cc_ders = self.sub_center_contacts_ders(cc, &rders, n);
        cc_ders.vder_info(&rders)
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

/// Derivations of centers and contact points
#[derive(Clone, Copy, Debug)]
pub struct CenterContactDers {
    pub center_ders: CurveDers<Vector3>,
    pub contact0_ders: CurveDers<Vector3>,
    pub contact1_ders: CurveDers<Vector3>,
    pub uv0_ders: CurveDers<Vector2>,
    pub uv1_ders: CurveDers<Vector2>,
}

impl CenterContactDers {
    fn cross_ders(&self) -> CurveDers<Vector3> {
        let sub = |x, y| x - y;
        let cp0 = self.contact0_ders.element_wise_ders(&self.center_ders, sub);
        let cp1 = self.contact1_ders.element_wise_ders(&self.center_ders, sub);
        cp0.combinatorial_ders(&cp1, CROSS)
    }
    fn axis_ders(
        &self,
        cross_ders: &CurveDers<Vector3>,
        abs_cross_ders: &CurveDers<f64>,
    ) -> CurveDers<Vector3> {
        cross_ders
            .element_wise_ders(abs_cross_ders, Vector3::extend)
            .rat_ders()
    }
    fn angle_ders(
        &self,
        abs_cross_ders: &CurveDers<f64>,
        rders: &CurveDers<f64>,
    ) -> CurveDers<f64> {
        let n = rders.max_order() - 1;
        let mut r_rat_ders = CurveDers::<f64>::new(n);
        r_rat_ders[0] = rders[1] / rders[0];
        (1..=n).for_each(|m| {
            let sum = rders.combinatorial_der(&r_rat_ders, MUL, m);
            r_rat_ders[m] = (rders[m + 1] - sum) / rders[0];
        });

        let cp0 = self
            .contact0_ders
            .element_wise_ders(&self.center_ders, |x, y| x - y);
        let cp1 = self
            .contact1_ders
            .element_wise_ders(&self.center_ders, |x, y| x - y);
        let dot_ders = cp0.combinatorial_ders(&cp1, DOT);

        let mut angle_ders = CurveDers::new(n);
        angle_ders[0] = f64::acos(f64::min(cp0[0].dot(cp1[0]) / (rders[0] * rders[0]), 1.0));
        (1..=n).for_each(|m| {
            let sum0 = angle_ders
                .der()
                .combinatorial_der(abs_cross_ders, MUL, m - 1);
            let sum1 = r_rat_ders.combinatorial_der(&dot_ders, MUL, m - 1);
            angle_ders[m] = (2.0 * sum1 - dot_ders[m] - sum0) / abs_cross_ders[0]
        });
        angle_ders
    }
    pub(super) fn vder_info(self, rders: &CurveDers<f64>) -> VderInfo {
        let cross_ders = self.cross_ders();
        let abs_cross_ders = cross_ders.abs_ders();
        VderInfo {
            axis_ders: self.axis_ders(&cross_ders, &abs_cross_ders),
            angle_ders: self.angle_ders(&abs_cross_ders, rders),
            cc_ders: self,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct VderInfo {
    cc_ders: CenterContactDers,
    axis_ders: CurveDers<Vector3>,
    angle_ders: CurveDers<f64>,
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
    ders: SurfaceDers<Vector3>,
    tders: CurveDers<Vector3>,
    uderders: CurveDers<Vector3>,
    vderders: CurveDers<Vector3>,
    crossders: CurveDers<Vector3>,
    abs_crossders: CurveDers<f64>,
    nders: CurveDers<Vector3>,
    uvders: CurveDers<Vector2>,
}

impl SurfaceInfo {
    fn new(surface: &impl ParametricSurface3D, (u, v): (f64, f64), n: usize) -> Self {
        let ders = surface.ders(n, u, v);

        let mut tders = CurveDers::new(n);
        tders[0] = ders[0][0];

        let mut uderders = CurveDers::new(n);
        uderders[0] = ders[1][0];

        let mut vderders = CurveDers::new(n);
        vderders[0] = ders[0][1];

        let mut crossders = CurveDers::new(n);
        crossders[0] = uderders[0].cross(vderders[0]);

        let mut abs_crossders = CurveDers::new(n);
        abs_crossders[0] = crossders[0].magnitude();

        let mut nders = CurveDers::new(n);
        nders[0] = surface.normal(u, v);

        let mut uvders = CurveDers::new(n);
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

    fn routine(&mut self, ders: &CurveDers<Vector3>, n: usize) {
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
        let sder_n_prime = sders.composite_der(uvders, n);

        let ders_sub = tders.element_wise_ders(ders, |x, y| x - y);

        let mut lhs_u = uderders.combinatorial_der(&ders_sub, DOT, n);
        let uder_n_prime = sders.uder().composite_der(uvders, n);
        lhs_u += uderders[0].dot(sder_n_prime) + uder_n_prime.dot(ders_sub[0]);

        let mut lhs_v = vderders.combinatorial_der(&ders_sub, DOT, n);
        let vder_n_prime = sders.vder().composite_der(uvders, n);
        lhs_v += vderders[0].dot(sder_n_prime) + vder_n_prime.dot(ders_sub[0]);

        let cp = tders[0] - ders[0];
        let uu = sders[1][0].magnitude2() + sders[2][0].dot(cp);
        let uv = sders[1][0].dot(sders[0][1]) + sders[1][1].dot(cp);
        let vv = sders[0][1].magnitude2() + sders[0][2].dot(cp);
        let mat = Matrix2::new(uu, uv, uv, vv);
        uvders[n] = -mat.invert().unwrap() * Vector2::new(lhs_u, lhs_v);
        tders[n] = sders.composite_der(uvders, n);
        uderders[n] = sders.uder().composite_der(uvders, n);
        vderders[n] = sders.vder().composite_der(uvders, n);

        crossders[n] = uderders.combinatorial_der(vderders, CROSS, n);
        let sum = crossders.der().combinatorial_der(crossders, DOT, n - 1)
            - abs_crossders
                .der()
                .combinatorial_der(abs_crossders, MUL, n - 1);
        abs_crossders[n] = sum / abs_crossders[0];
        let homog = crossders.element_wise_ders(abs_crossders, Vector3::extend);
        nders[n] = homog.rat_ders()[n];
    }
}

fn der_routine(
    s0info: &mut SurfaceInfo,
    s1info: &mut SurfaceInfo,
    cders: &CurveDers<Vector3>,
    rders: &CurveDers<f64>,
    ders: &mut CurveDers<Vector3>,
    n: usize,
) {
    let (n0ders, n1ders) = (&s0info.nders, &s1info.nders);
    let mat = Matrix3::from_cols(cders[1], n0ders[0], n1ders[0]).transpose();
    let (der_cders, der_ders) = (cders.der(), ders.der());
    let sub = cders.element_wise_ders(ders, |x, y| x - y);
    let b = Vector3::new(
        der_cders.combinatorial_der(&sub, DOT, n),
        rders[n] - der_ders.combinatorial_der(n0ders, DOT, n - 1),
        rders[n] - der_ders.combinatorial_der(n1ders, DOT, n - 1),
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
            let eders = edge_curve.ders(2, t);
            let (p, der, der2) = (Point3::from_vec(eders[0]), eders[1], eders[2]);
            let rders = radius.ders(1, t);
            let (r, r_der) = (rders[0], rders[1]);
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
    S0: ParametricSurface3D
        + SearchNearestParameter<D2, Point = Point3>
        + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D
        + SearchNearestParameter<D2, Point = Point3>
        + SearchParameter<D2, Point = Point3>,
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

        let (n0, n1) = (surface0.normal(u0, v0), surface1.normal(u1, v1));
        let sign = -f64::signum(n0.cross(n1).dot(edge_curve.der(t)));

        let (p0, p1) = (0..=trials).find_map(|_i| {
            let eders = edge_curve.ders(2, t);
            let (p, der, der2) = (Point3::from_vec(eders[0]), eders[1], eders[2]);
            let rders = radius.ders(1, t);
            let (r, r_der) = (rders[0] * sign, rders[1] * sign);
            let ders0 = surface0.ders(1, u0, v0);
            let (p0, uder0, vder0) = (Point3::from_vec(ders0[0][0]), ders0[1][0], ders0[0][1]);
            let n0 = uder0.cross(vder0).normalize();
            let ders1 = surface1.ders(1, u1, v1);
            let (p1, uder1, vder1) = (Point3::from_vec(ders1[0][0]), ders1[1][0], ders1[0][1]);
            let n1 = uder1.cross(vder1).normalize();

            let (pp0, pp1, pc) = (p0 - point, p1 - point, c - point);

            if pp0.so_small() {
                let cc0 = self.contact_curve0();
                t = cc0.search_parameter(point, t, trials)?;
                c = p0 + sign * radius.subs(t) * n0;
                (u1, v1) = surface1.search_nearest_parameter(c, (u1, v1), trials)?;
                return Some((p0, surface1.subs(u1, v1)));
            }
            if pp1.so_small() {
                let cc1 = self.contact_curve1();
                t = cc1.search_parameter(point, t, trials)?;
                c = p1 + sign * radius.subs(t) * n1;
                (u0, v0) = surface0.search_nearest_parameter(c, (u0, v0), trials)?;
                return Some((surface0.subs(u0, v0), p1));
            }
            let center_contact0 = (p0 + r * n0).near(&c);
            let center_contact1 = (p1 + r * n1).near(&c);
            let same_plane = pp0.cross(pp1).dot(pc).so_small2();
            if center_contact0 && center_contact1 && same_plane {
                return Some((p0, p1));
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
            debug_assert!(duv0.0.z.so_small() && duv0.1.z.so_small(), "{duv0:?}");

            let duv1 = {
                let mat = Matrix3::from_cols(
                    uder1 + r * surface1.normal_uder(u1, v1),
                    vder1 + r * surface1.normal_vder(u1, v1),
                    n1,
                );
                mat.invert().unwrap() * (c_next - PointVector3(p1 + r * n1, r_der * n1))
            };
            debug_assert!(duv1.0.z.so_small() && duv1.1.z.so_small(), "{duv1:?}");

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

impl<C, S0, S1, R> RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    /// Returns the cross point of contact curve and other edge.
    pub fn search_contact_curve0_cross_point_with_adjacent_edge<C0>(
        &self,
        edge_hint: f64,
        adjacent_curve: C0,
        adjacent_hint: f64,
        trials: usize,
    ) -> Option<(ContactPoint, ContactPoint, f64, f64)>
    where
        C0: ParametricCurve3D,
    {
        let Self {
            edge_curve,
            surface0,
            surface1,
            radius,
        } = self;
        let (mut t, mut s, mut w) = (edge_hint, adjacent_hint, 0.0);
        let mut c = edge_curve.subs(t);
        let ((mut u0, mut v0), (mut u1, mut v1)) = (
            surface0.search_parameter(c, None, trials)?,
            surface1.search_parameter(c, None, trials)?,
        );

        let (n0, n1) = (surface0.normal(u0, v0), surface1.normal(u1, v1));
        let sign = -f64::signum(n0.cross(n1).dot(edge_curve.der(t)));

        for _i in 0..=trials {
            let cders = edge_curve.ders(2, t);
            let (p, der, der2) = (Point3::from_vec(cders[0]), cders[1], cders[2]);

            let rders = radius.ders(1, t);
            let (r, r_der) = (rders[0] * sign, rders[1] * sign);

            let ders0 = surface0.ders(1, u0, v0);
            let (p0, uder0, vder0) = (Point3::from_vec(ders0[0][0]), ders0[1][0], ders0[0][1]);
            let n0 = uder0.cross(vder0).normalize();

            let ders1 = surface1.ders(1, u1, v1);
            let (p1, uder1, vder1) = (Point3::from_vec(ders1[0][0]), ders1[1][0], ders1[0][1]);
            let n1 = uder1.cross(vder1).normalize();

            let aders = adjacent_curve.ders(1, s);
            let (e, eder) = (Point3::from_vec(aders[0]), aders[1]);

            let pc = c - p;

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
            debug_assert!(duv0.0.z.so_small() && duv0.1.z.so_small(), "{duv0:?}");

            let duv1 = {
                let mat = Matrix3::from_cols(
                    uder1 + r * surface1.normal_uder(u1, v1),
                    vder1 + r * surface1.normal_vder(u1, v1),
                    n1,
                );
                mat.invert().unwrap() * (c_next - PointVector3(p1 + r * n1, r_der * n1))
            };
            debug_assert!(duv1.0.z.so_small() && duv1.1.z.so_small(), "{duv1:?}");

            let dp0 = Matrix3::from_cols(uder0, vder0, n0) * duv0;

            let mat = Matrix3::from_cols(dp0.1, n0, -eder);
            let vec = (e - p0) - w * n0 - dp0.0;

            let Vector3 {
                x: dt,
                y: dw,
                z: ds,
            } = mat.invert().unwrap() * vec;

            if p0.near(&e) && dt.so_small2() && ds.so_small2() && dw.so_small2() {
                let contact_point0 = ContactPoint {
                    point: p0,
                    uv: (u0, v0).into(),
                };
                let contact_point1 = ContactPoint {
                    point: p1,
                    uv: (u1, v1).into(),
                };
                return Some((contact_point0, contact_point1, t, s));
            }

            (t, s, w) = (t + dt, s + ds, w + dw);
            c = c_next.0 + c_next.1 * dt;
            let (duv0, duv1) = (duv0.0 + duv0.1 * dt, duv1.0 + duv1.1 * dt);
            (u0, v0) = (u0 + duv0.x, v0 + duv0.y);
            (u1, v1) = (u1 + duv1.x, v1 + duv1.y);
        }

        None
    }
    /// Returns the cross point of contact curve and other edge.
    pub fn search_contact_curve1_cross_point_with_adjacent_edge<C0>(
        &self,
        edge_hint: f64,
        adjacent_curve: C0,
        adjacent_hint: f64,
        trials: usize,
    ) -> Option<(ContactPoint, ContactPoint, f64, f64)>
    where
        C0: ParametricCurve3D,
    {
        let Self {
            edge_curve,
            surface0,
            surface1,
            radius,
        } = self;
        let (mut t, mut s, mut w) = (edge_hint, adjacent_hint, 0.0);
        let mut c = edge_curve.subs(t);
        let ((mut u0, mut v0), (mut u1, mut v1)) = (
            surface0.search_parameter(c, None, trials)?,
            surface1.search_parameter(c, None, trials)?,
        );

        let (n0, n1) = (surface0.normal(u0, v0), surface1.normal(u1, v1));
        let sign = -f64::signum(n0.cross(n1).dot(edge_curve.der(t)));

        for _i in 0..=trials {
            let cders = edge_curve.ders(2, t);
            let (p, der, der2) = (Point3::from_vec(cders[0]), cders[1], cders[2]);

            let rders = radius.ders(1, t);
            let (r, r_der) = (rders[0] * sign, rders[1] * sign);

            let ders0 = surface0.ders(1, u0, v0);
            let (p0, uder0, vder0) = (Point3::from_vec(ders0[0][0]), ders0[1][0], ders0[0][1]);
            let n0 = uder0.cross(vder0).normalize();

            let ders1 = surface1.ders(1, u1, v1);
            let (p1, uder1, vder1) = (Point3::from_vec(ders1[0][0]), ders1[1][0], ders1[0][1]);
            let n1 = uder1.cross(vder1).normalize();

            let aders = adjacent_curve.ders(1, s);
            let (e, eder) = (Point3::from_vec(aders[0]), aders[1]);

            let pc = c - p;

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
            debug_assert!(duv0.0.z.so_small() && duv0.1.z.so_small(), "{duv0:?}");

            let duv1 = {
                let mat = Matrix3::from_cols(
                    uder1 + r * surface1.normal_uder(u1, v1),
                    vder1 + r * surface1.normal_vder(u1, v1),
                    n1,
                );
                mat.invert().unwrap() * (c_next - PointVector3(p1 + r * n1, r_der * n1))
            };
            debug_assert!(duv1.0.z.so_small() && duv1.1.z.so_small(), "{duv1:?}");

            let dp1 = Matrix3::from_cols(uder1, vder1, n1) * duv1;

            let mat = Matrix3::from_cols(dp1.1, n1, -eder);
            let vec = (e - p1) - w * n1 - dp1.0;

            let Vector3 {
                x: dt,
                y: dw,
                z: ds,
            } = mat.invert().unwrap() * vec;

            if p1.near(&e) && dt.so_small2() && ds.so_small2() && dw.so_small2() {
                let contact_point0 = ContactPoint {
                    point: p0,
                    uv: (u0, v0).into(),
                };
                let contact_point1 = ContactPoint {
                    point: p1,
                    uv: (u1, v1).into(),
                };
                return Some((contact_point0, contact_point1, t, s));
            }

            (t, s, w) = (t + dt, s + ds, w + dw);
            c = c_next.0 + c_next.1 * dt;
            let (duv0, duv1) = (duv0.0 + duv0.1 * dt, duv1.0 + duv1.1 * dt);
            (u0, v0) = (u0 + duv0.x, v0 + duv0.y);
            (u1, v1) = (u1 + duv1.x, v1 + duv1.y);
        }

        None
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

        let rders = fillet.radius.ders(5, t);

        let eps = 1.0e-4;
        let cc_plus = fillet.contact_circle(t + eps).unwrap();
        let rders_plus = fillet.radius.ders(4, t + eps);

        let cc_minus = fillet.contact_circle(t - eps).unwrap();
        let rders_minus = fillet.radius.ders(4, t - eps);

        let cc_ders = fillet.sub_center_contacts_ders(cc, &rders, 4);
        let cc_ders_plus = fillet.sub_center_contacts_ders(cc_plus, &rders_plus, 3);
        let cc_ders_minus = fillet.sub_center_contacts_ders(cc_minus, &rders_minus, 3);

        let cross_ders = cc_ders.cross_ders();
        let cross_ders_plus = cc_ders_plus.cross_ders();
        let cross_ders_minus = cc_ders_minus.cross_ders();

        let abs_cross_ders = cross_ders.abs_ders();
        let abs_cross_ders_plus = cross_ders_plus.abs_ders();
        let abs_cross_ders_minus = cross_ders_minus.abs_ders();

        let axis_ders = cc_ders.axis_ders(&cross_ders, &abs_cross_ders);
        let axis_ders_plus = cc_ders_plus.axis_ders(&cross_ders_plus, &abs_cross_ders_plus);
        let axis_ders_minus = cc_ders_minus.axis_ders(&cross_ders_minus, &abs_cross_ders_minus);

        let angle_ders = cc_ders.angle_ders(&abs_cross_ders, &rders);
        let angle_ders_plus = cc_ders_plus.angle_ders(&abs_cross_ders_plus, &rders_plus);
        let angle_ders_minus = cc_ders_minus.angle_ders(&abs_cross_ders_minus, &rders_minus);

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
