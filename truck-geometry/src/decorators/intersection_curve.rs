use super::*;
use truck_base::newton::{self, CalcOutput};

fn subs_tuple<S: ParametricSurface>(
    surface: &S,
    (u, v): (f64, f64),
) -> (S::Point, S::Vector, S::Vector) {
    (surface.subs(u, v), surface.uder(u, v), surface.vder(u, v))
}

fn double_projection<S0, S1>(
    surface0: &S0,
    hint0: Option<(f64, f64)>,
    surface1: &S1,
    hint1: Option<(f64, f64)>,
    plane_point: Point3,
    plane_normal: Vector3,
    trials: usize,
) -> Option<(Point3, Point2, Point2)>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    let function = move |Vector4 { x, y, z, w }| {
        let (pt0, uder0, vder0) = subs_tuple(surface0, (x, y));
        let (pt1, uder1, vder1) = subs_tuple(surface1, (z, w));
        CalcOutput {
            value: (pt0 - pt1).extend(plane_normal.dot(pt0.midpoint(pt1) - plane_point)),
            derivation: Matrix4::from_cols(
                uder0.extend(plane_normal.dot(uder0) / 2.0),
                vder0.extend(plane_normal.dot(vder0) / 2.0),
                (-uder1).extend(plane_normal.dot(uder1) / 2.0),
                (-vder1).extend(plane_normal.dot(vder1) / 2.0),
            ),
        }
    };
    let (x, y) = hint0.or_else(|| surface0.search_nearest_parameter(plane_point, hint0, trials))?;
    let (z, w) = hint1.or_else(|| surface1.search_nearest_parameter(plane_point, hint1, trials))?;
    let Vector4 { x, y, z, w } = newton::solve(function, Vector4 { x, y, z, w }, trials).ok()?;
    let point = surface0.subs(x, y).midpoint(surface1.subs(z, w));
    Some((point, Point2::new(x, y), Point2::new(z, w)))
}

impl<C, S0, S1> IntersectionCurve<C, S0, S1> {
    /// Constructor
    #[inline(always)]
    pub fn new(surface0: S0, surface1: S1, leader: C) -> Self {
        Self {
            surface0,
            surface1,
            leader,
        }
    }
    /// This curve is a part of intersection of `self.surface0()` and `self.surface1()`.
    #[inline(always)]
    pub fn surface0(&self) -> &S0 { &self.surface0 }
    /// This curve is a part of intersection of `self.surface0()` and `self.surface1()`.
    #[inline(always)]
    pub fn surface1(&self) -> &S1 { &self.surface1 }
    /// Returns the polyline leading this curve.
    #[inline(always)]
    pub fn leader(&self) -> &C { &self.leader }
    /// This curve is a part of intersection of `self.surface0()` and `self.surface1()`.
    #[inline(always)]
    pub fn surface0_mut(&mut self) -> &mut S0 { &mut self.surface0 }
    /// This curve is a part of intersection of `self.surface0()` and `self.surface1()`.
    #[inline(always)]
    pub fn surface1_mut(&mut self) -> &mut S1 { &mut self.surface1 }
    /// Returns the curve leading this curve.
    #[inline(always)]
    pub fn leader_mut(&mut self) -> &mut C { &mut self.leader }
    /// destruct `self`.
    #[inline(always)]
    pub fn destruct(self) -> (S0, S1, C) { (self.surface0, self.surface1, self.leader) }
}

impl<C, S0, S1> IntersectionCurve<C, S0, S1>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    /// Search triple value of the point corresponding to the parameter `t`.
    /// - the coordinate on 3D space
    /// - the uv coordinate on `self.surface0()`
    /// - the uv coordinate on `self.surface1()`
    #[inline(always)]
    pub fn search_triple(&self, t: f64, trials: usize) -> Option<(Point3, Point2, Point2)> {
        double_projection(
            self.surface0(),
            None,
            self.surface1(),
            None,
            self.leader.subs(t),
            self.leader.der(t),
            trials,
        )
    }
    /// Search triple value of the point nearest to `point`.
    /// - the coordinate on 3D space
    /// - the uv coordinate on `self.surface0()`
    /// - the uv coordinate on `self.surface1()`
    pub fn search_nearest_point(
        &self,
        point: Point3,
        hint0: Option<(f64, f64)>,
        hint1: Option<(f64, f64)>,
        trials: usize,
    ) -> Option<(Point3, Point2, Point2)> {
        let (surface0, surface1) = (self.surface0(), self.surface1());
        let function = |Vector4 { x, y, z, w }| {
            let (pt0, uder0, vder0, uuder0, uvder0, vvder0) = subs_tuple_der2(surface0, (x, y));
            let (pt1, uder1, vder1, uuder1, uvder1, vvder1) = subs_tuple_der2(surface1, (z, w));
            let diff = pt0.midpoint(pt1) - point;
            let (n0, n1) = (uder0.cross(vder0), uder1.cross(vder1));
            let n = n0.cross(n1);
            let n_xder = (uuder0.cross(vder0) + uder0.cross(uvder0)).cross(n1);
            let n_yder = (uvder0.cross(vder0) + uder0.cross(vvder0)).cross(n1);
            let n_zder = n0.cross(uuder1.cross(vder1) + uder1.cross(uvder1));
            let n_wder = n0.cross(uvder1.cross(vder1) + uder1.cross(vvder1));
            CalcOutput {
                value: (pt0 - pt1).extend(n.dot(diff)),
                derivation: Matrix4::from_cols(
                    uder0.extend(n_xder.dot(diff) + n.dot(uder0) / 2.0),
                    vder0.extend(n_yder.dot(diff) + n.dot(vder0) / 2.0),
                    (-uder1).extend(n_zder.dot(diff) + n.dot(uder1) / 2.0),
                    (-vder1).extend(n_wder.dot(diff) + n.dot(vder1) / 2.0),
                ),
            }
        };
        let (x, y) = hint0.or_else(|| surface0.search_nearest_parameter(point, hint0, trials))?;
        let (z, w) = hint1.or_else(|| surface1.search_nearest_parameter(point, hint1, trials))?;
        let Vector4 { x, y, z, w } =
            newton::solve(function, Vector4 { x, y, z, w }, trials).ok()?;
        let point = surface0.subs(x, y).midpoint(surface1.subs(z, w));
        Some((point, Point2::new(x, y), Point2::new(z, w)))
    }
}

#[derive(Clone, Copy, Debug)]
struct DerRoutineImmutableArgs<'a, A> {
    s0ders: &'a [A],
    s0normal: Vector3,
    s1ders: &'a [A],
    s1normal: Vector3,
    leaders: &'a [Vector3],
}

fn curve_der_n(
    sum0: Vector3,
    s0normal: Vector3,
    sum1: Vector3,
    s1normal: Vector3,
    leaders: &[Vector3],
    cders: &[Vector3],
    n: usize,
) -> Vector3 {
    let mat = Matrix3::from_cols(s0normal, s1normal, leaders[1]).transpose();
    let (f0, f1) = (|i| leaders[i + 1], |i| leaders[i] - cders[i]);
    let suml = comp_sum(n, f0, f1, Vector3::dot);
    let b = Vector3::new(s0normal.dot(sum0), s1normal.dot(sum1), suml);
    mat.invert().unwrap() * b
}

fn uv_der_n(
    uder: Vector3,
    vder: Vector3,
    sum: Vector3,
    normal: Vector3,
    cder_n: Vector3,
) -> Vector2 {
    let mat = Matrix3::from_cols(uder, vder, normal);
    let b = cder_n - sum;
    let uv_der_n = mat.invert().unwrap() * b;
    debug_assert!(uv_der_n.z.abs() < 1.0e-4, "{}", uv_der_n.z.abs());
    Vector2::new(uv_der_n.x, uv_der_n.y)
}

fn der_routine<A: AsRef<[Vector3]>>(
    DerRoutineImmutableArgs {
        s0ders,
        s0normal,
        s1ders,
        s1normal,
        leaders,
    }: DerRoutineImmutableArgs<'_, A>,
    uv0ders: &mut [Vector2],
    uv1ders: &mut [Vector2],
    cders: &mut [Vector3],
    n: usize,
) {
    let sum0 = pcurve::raw_der_n(s0ders, uv0ders, n);
    let sum1 = pcurve::raw_der_n(s1ders, uv1ders, n);
    cders[n] = curve_der_n(sum0, s0normal, sum1, s1normal, leaders, cders, n);
    let (uder0, vder0) = (s0ders[1].as_ref()[0], s0ders[0].as_ref()[1]);
    uv0ders[n] = uv_der_n(uder0, vder0, sum0, s0normal, cders[n]);
    let (uder1, vder1) = (s1ders[1].as_ref()[0], s1ders[0].as_ref()[1]);
    uv1ders[n] = uv_der_n(uder1, vder1, sum1, s1normal, cders[n]);
}

impl<C, S0, S1> ParametricCurve for IntersectionCurve<C, S0, S1>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, t: f64) -> Point3 { self.search_triple(t, 100).unwrap().0 }
    fn der(&self, t: f64) -> Vector3 {
        let IntersectionCurve {
            surface0,
            surface1,
            leader,
        } = self;
        let [l, l_der, l_der2] = leader.ders_array(t);
        let (c, uv0, uv1) = self.search_triple(t, 100).unwrap();
        let (n0, n1) = (surface0.normal(uv0.x, uv0.y), surface1.normal(uv1.x, uv1.y));
        let n = n0.cross(n1);
        let k = (l_der.magnitude2() - (c - l).dot(l_der2)) / n.dot(l_der);
        n * k
    }
    #[inline(always)]
    fn der2(&self, t: f64) -> Vector3 { self.der_n(2, t) }
    #[inline(always)]
    fn der_n(&self, n: usize, t: f64) -> Vector3 {
        match n {
            0 => return self.subs(t).to_vec(),
            1 => return self.der(t),
            _ => {}
        }
        let mut cders = [Vector3::zero(); 32];
        self.ders(t, &mut cders[0..=n]);
        cders[n]
    }
    #[inline(always)]
    fn ders_vec(&self, n: usize, t: f64) -> Vec<Self::Vector> {
        let mut res = vec![Vector3::zero(); n + 1];
        self.ders(t, &mut res);
        res
    }
    #[inline(always)]
    fn ders_array<const LEN: usize>(&self, t: f64) -> [Self::Vector; LEN] {
        let mut res = [Vector3::zero(); LEN];
        self.ders(t, &mut res);
        res
    }
    fn ders(&self, t: f64, cders: &mut [Vector3]) {
        if cders.is_empty() {
            return;
        }
        let n = cders.len() - 1;
        let IntersectionCurve {
            surface0,
            surface1,
            leader,
        } = self;
        let (c, uv0, uv1) = self.search_triple(t, 100).unwrap();

        let s0ders = &mut [[Vector3::zero(); 32]; 32];
        let s0normal = surface_info(surface0, uv0, s0ders, n);
        let mut uv0ders = [Vector2::zero(); 32];
        uv0ders[0] = uv0.to_vec();

        let s1ders = &mut [[Vector3::zero(); 32]; 32];
        let s1normal = surface_info(surface1, uv1, s1ders, n);
        let mut uv1ders = [Vector2::zero(); 32];
        uv1ders[0] = uv1.to_vec();

        let leaders = &mut [Vector3::zero(); 32];
        leader.ders(t, &mut leaders[0..=n + 1]);

        cders.iter_mut().for_each(|c| *c = Vector3::zero());
        cders[0] = c.to_vec();

        let info = DerRoutineImmutableArgs {
            s0ders,
            s0normal,
            s1ders: s1ders,
            s1normal,
            leaders,
        };
        (1..=n).for_each(move |i| der_routine(info, &mut uv0ders, &mut uv1ders, cders, i));
    }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { self.leader.parameter_range() }
}

fn surface_info<S: ParametricSurface3D>(
    surface: S,
    uv: Point2,
    ders: &mut [[Vector3; 32]; 32],
    n: usize,
) -> Vector3 {
    let mut ders_mut = ders[0..=n]
        .iter_mut()
        .enumerate()
        .map(|(i, slice)| &mut slice[0..=n - i])
        .collect::<Vec<_>>();
    surface.ders(uv.x, uv.y, &mut ders_mut);
    surface.normal(uv.x, uv.y)
}

impl<C, S0, S1> BoundedCurve for IntersectionCurve<C, S0, S1>
where
    C: ParametricCurve3D + BoundedCurve,
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
}

impl<C, S0, S1> ParameterDivision1D for IntersectionCurve<C, S0, S1>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    #[inline(always)]
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Point3>) {
        algo::curve::parameter_division(self, range, tol)
    }
}

impl<C, S0, S1> Cut for IntersectionCurve<C, S0, S1>
where
    C: Cut<Point = Point3, Vector = Vector3>,
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    #[inline(always)]
    fn cut(&mut self, t: f64) -> Self {
        Self {
            surface0: self.surface0.clone(),
            surface1: self.surface1.clone(),
            leader: self.leader.cut(t),
        }
    }
}

impl<C: Invertible, S0: Clone, S1: Clone> Invertible for IntersectionCurve<C, S0, S1> {
    fn invert(&mut self) { self.leader.invert(); }
}

impl<C, S0, S1> SearchParameter<D1> for IntersectionCurve<C, S0, S1>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let t = self
            .leader()
            .search_nearest_parameter(point, hint, trials)?;
        let pt = self.subs(t);
        match pt.near(&point) {
            true => Some(t),
            false => None,
        }
    }
}

type DersSubsTuple<S> = (
    <S as ParametricSurface>::Point,
    <S as ParametricSurface>::Vector,
    <S as ParametricSurface>::Vector,
    <S as ParametricSurface>::Vector,
    <S as ParametricSurface>::Vector,
    <S as ParametricSurface>::Vector,
);

fn subs_tuple_der2<S: ParametricSurface>(surface: &S, (u, v): (f64, f64)) -> DersSubsTuple<S> {
    (
        surface.subs(u, v),
        surface.uder(u, v),
        surface.vder(u, v),
        surface.uuder(u, v),
        surface.uvder(u, v),
        surface.vvder(u, v),
    )
}

impl<C, S0, S1> SearchNearestParameter<D1> for IntersectionCurve<C, S0, S1>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let (near_point, _, _) = self.search_nearest_point(point, None, None, trials)?;
        self.leader()
            .search_nearest_parameter(near_point, hint, trials)
    }
}

impl<C, S0, S1> Transformed<Matrix4> for IntersectionCurve<C, S0, S1>
where
    C: Transformed<Matrix4>,
    S0: Transformed<Matrix4>,
    S1: Transformed<Matrix4>,
{
    fn transform_by(&mut self, trans: Matrix4) {
        self.surface0.transform_by(trans);
        self.surface1.transform_by(trans);
        self.leader.transform_by(trans);
    }
}

impl<C: BoundedCurve> IntersectionCurve<C, Plane, Plane> {
    /// Optimizes intersection curve of [`Plane`] into [`Line`].
    #[inline]
    pub fn optimize(&self) -> Line<C::Point> {
        let (s, t) = self.leader.range_tuple();
        Line(self.leader.subs(s), self.leader.subs(t))
    }
}

#[cfg(test)]
mod double_projection_tests {
    use super::*;
    use proptest::prelude::*;
    use std::f64::consts::PI;
    type PResult = std::result::Result<(), TestCaseError>;

    fn get_one_vector(u: [f64; 2]) -> Vector3 {
        let angle = 2.0 * PI * u[0];
        let w = f64::sqrt(1.0 - u[1] * u[1]);
        Vector3::new(w * f64::cos(angle), w * f64::sin(angle), u[1])
    }

    fn create_axis(n: Vector3) -> (Vector3, Vector3) {
        let idx = if n[0].abs() < n[1].abs() { 0 } else { 1 };
        let idx = if n[idx].abs() < n[2].abs() { idx } else { 2 };
        let mut e = Vector3::zero();
        e[idx] = 1.0;
        let x = n.cross(e).normalize();
        (x, n.cross(x))
    }

    fn exec_plane_case(c0: [f64; 3], n0: [f64; 2], c1: [f64; 3], n1: [f64; 2]) -> PResult {
        let c0 = Point3::from(c0);
        let n0 = get_one_vector(n0);
        let (x, y) = create_axis(n0);
        let plane0 = Plane::new(c0, c0 + x, c0 + y);
        let c1 = Point3::from(c1);
        let n1 = get_one_vector(n1);
        let (x, y) = create_axis(n1);
        let plane1 = Plane::new(c1, c1 + x, c1 + y);
        let n = n0.cross(n1).normalize();
        let mut o = None;
        for i in 0..10 {
            let t = i as f64;
            let p = Point3::origin() + t * n;
            let (q, p0, p1) = double_projection(&plane0, None, &plane1, None, p, n, 100)
                .unwrap_or_else(|| panic!("plane0: {plane0:?}\nplane1: {plane1:?}\n p: {p:?}"));
            prop_assert_near!(q, plane0.subs(p0.x, p0.y));
            prop_assert_near!(q, plane1.subs(p1.x, p1.y));
            if let Some(o) = o {
                prop_assert_near!(q.distance2(o), t * t);
            } else {
                o = Some(q);
            }
        }
        Ok(())
    }

    proptest! {
        #[test]
        fn plane_case(
            c0 in prop::array::uniform3(-1f64..=1f64),
            n0 in prop::array::uniform2(0f64..=1f64),
            c1 in prop::array::uniform3(-1f64..=1f64),
            n1 in prop::array::uniform2(0f64..=1f64),
        ) {
            exec_plane_case(c0, n0, c1, n1)?;
        }
    }

    fn exec_sphere_case(t: f64, r: f64) -> PResult {
        let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 1.0), f64::sqrt(2.0));
        let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -1.0), f64::sqrt(2.0));
        let p = Point3::new(r * f64::cos(t), r * f64::sin(t), 0.0);
        let n = Vector3::new(-f64::sin(t), f64::cos(t), 0.0);
        let (q, p0, p1) = double_projection(&sphere0, None, &sphere1, None, p, n, 100)
            .unwrap_or_else(|| panic!("p: {p:?}"));
        prop_assert_near!(q, sphere0.subs(p0.x, p0.y));
        prop_assert_near!(q, sphere1.subs(p1.x, p1.y));
        prop_assert_near!(q, Point3::new(f64::cos(t), f64::sin(t), 0.0));
        Ok(())
    }

    proptest! {
        #[test]
        fn sphere_case(t in 0f64..=(2.0 * PI), r in 0.5f64..=1.5f64) {
            exec_sphere_case(t, r)?;
        }
    }
}
