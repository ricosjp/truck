use super::{rbf_surface::*, *};

impl<S0, S1> ApproxFilletSurface<S0, S1> {
    /// Returns the first surface.
    pub const fn surface0(&self) -> &S0 { &self.surface0 }
    /// Returns the second surface.
    pub const fn surface1(&self) -> &S1 { &self.surface1 }
    /// Returns the knot vector for the parameter `v`.
    pub const fn vknot_vec(&self) -> &KnotVec { &self.knot_vec }
    /// Returns side curve on the first surface.
    pub fn side_pcurve0(&self) -> PCurve<BSplineCurve<Point2>, S0>
    where S0: Clone {
        let bsp = BSplineCurve::new(self.knot_vec.clone(), self.side_control_points0.clone());
        PCurve::new(bsp, self.surface0.clone())
    }
    /// Returns side curve on the second surface.
    pub fn side_pcurve1(&self) -> PCurve<BSplineCurve<Point2>, S1>
    where S1: Clone {
        let bsp = BSplineCurve::new(self.knot_vec.clone(), self.side_control_points1.clone());
        PCurve::new(bsp, self.surface1.clone())
    }
    fn vdegree(&self) -> usize { self.knot_vec.len() - self.weights.len() - 1 }
}

fn pmul<P: EuclideanSpace>((b, p): (&P::Scalar, &P)) -> P::Diff { p.to_vec() * *b }
fn tmul<S: Copy, T: Copy + Mul<S>>((b, v): (&S, &T)) -> <T as Mul<S>>::Output { *v * *b }

type SurfaceTriple<'a, S> = (S, &'a Vec<Point2>, &'a Vec<Vector2>);
fn u_control_points<S: ParametricSurface3D>(
    basis: &[f64],
    dbasis: &[f64],
    (surface, side_control_points, tangent_vecs): SurfaceTriple<'_, S>,
    weight: f64,
) -> (Vector4, Vector4) {
    let uv: Vector2 = basis.iter().zip(side_control_points).map(pmul).sum();
    let sders = surface.ders(1, uv.x, uv.y);
    let (p, uder, vder) = (sders[0][0], sders[1][0], sders[0][1]);
    let duv: Vector2 = dbasis.iter().zip(side_control_points).map(pmul).sum();
    let cder = (uder * duv.x + vder * duv.y).normalize();
    let axis = cder.cross(uder.cross(vder).normalize());
    let tuv: Vector2 = basis.iter().zip(tangent_vecs).map(tmul).sum();
    let wq = weight * p + (axis * tuv.x + cder * tuv.y) / 3.0;
    (p.extend(1.0), wq.extend(weight))
}

const fn bezier_3rd_basis(n: usize, u: f64) -> [f64; 4] {
    let _1subu = 1.0 - u;
    match n {
        0 => [
            _1subu * _1subu * _1subu,
            3.0 * _1subu * _1subu * u,
            3.0 * _1subu * u * u,
            u * u * u,
        ],
        1 => [
            -3.0 * _1subu * _1subu,
            3.0 * _1subu * (1.0 - 3.0 * u),
            3.0 * u * (2.0 - 3.0 * u),
            3.0 * u * u,
        ],
        2 => [
            6.0 * _1subu,
            -6.0 * (2.0 - 3.0 * u),
            6.0 * (1.0 - 3.0 * u),
            6.0 * u,
        ],
        3 => [-6.0, 18.0, -18.0, 6.0],
        _ => [0.0; 4],
    }
}

mod subders {
    use super::*;
    fn v_axis_ders(p_ders: &CurveDers<Vector3>) -> CurveDers<Vector3> {
        let p_derders = p_ders.der();
        let homog_ders = p_derders.element_wise_ders(&p_derders.abs_ders(), Vector3::extend);
        homog_ders.rat_ders()
    }
    fn n_ders(s_ders: &SurfaceDers<Vector3>, uv_ders: &CurveDers<Vector2>) -> CurveDers<Vector3> {
        let uders = s_ders.uder().composite_ders(&uv_ders);
        let vders = s_ders.vder().composite_ders(&uv_ders);
        let lnders = uders.combinatorial_ders(&vders, Vector3::cross);
        let homog = lnders.element_wise_ders(&lnders.abs_ders(), Vector3::extend);
        homog.rat_ders()
    }
    fn wq_ders(
        w_ders: &CurveDers<f64>,
        p_ders: &CurveDers<Vector3>,
        b_ders: &CurveDers<Vector2>,
        n_ders: &CurveDers<Vector3>,
    ) -> CurveDers<Vector3> {
        use std::ops::Add;
        let v_axis_ders = v_axis_ders(&p_ders);
        let u_axis_ders = v_axis_ders.combinatorial_ders(&n_ders, Vector3::cross);
        let wp_ders = w_ders.combinatorial_ders(p_ders, |w, p| w * p);
        let aders = b_ders.combinatorial_ders(&u_axis_ders, |v, p| v[0] * p) / 3.0;
        let bders = b_ders.combinatorial_ders(&v_axis_ders, |v, p| v[1] * p) / 3.0;
        wp_ders
            .element_wise_ders(&aders, Add::add)
            .element_wise_ders(&bders, Add::add)
    }
    fn lift_p_ders(p_ders: &CurveDers<Vector3>) -> CurveDers<Vector4> {
        let mut w_ders = CurveDers::<f64>::new(p_ders.max_order());
        w_ders[0] = 1.0;
        p_ders.element_wise_ders(&w_ders, Vector3::extend)
    }
    pub fn control_points_ders(
        s_ders: &SurfaceDers<Vector3>,
        uv_ders: &CurveDers<Vector2>,
        b_ders: &CurveDers<Vector2>,
        w_ders: &CurveDers<f64>,
    ) -> (CurveDers<Vector4>, CurveDers<Vector4>) {
        let p_ders = s_ders.composite_ders(uv_ders);
        let lift_p_ders = lift_p_ders(&p_ders);
        let wq_ders = wq_ders(w_ders, &p_ders, b_ders, &n_ders(s_ders, uv_ders));
        let lift_q_ders = wq_ders.element_wise_ders(&w_ders, |x, y| x.extend(y));
        (lift_p_ders, lift_q_ders)
    }
}

impl<S0, S1> ParametricSurface for ApproxFilletSurface<S0, S1>
where
    S0: ParametricSurface3D,
    S1: ParametricSurface3D,
{
    type Point = Point3;
    type Vector = Vector3;
    fn ders(&self, max_order: usize, u: f64, v: f64) -> SurfaceDers<Vector3> {
        let degree = self.vdegree();
        let [mut uv0_ders, mut uv1_ders, mut b0_ders, mut b1_ders] =
            [CurveDers::<Vector2>::new(max_order + 1); 4];
        let mut w_ders = CurveDers::<f64>::new(max_order + 1);
        (0..=max_order + 1).for_each(|order| {
            let basis = self.knot_vec.bspline_basis_functions(degree, order, v);
            uv0_ders[order] = basis.iter().zip(&self.side_control_points0).map(pmul).sum();
            uv1_ders[order] = basis.iter().zip(&self.side_control_points1).map(pmul).sum();
            b0_ders[order] = basis.iter().zip(&self.tangent_vecs0).map(tmul).sum();
            b1_ders[order] = basis.iter().zip(&self.tangent_vecs1).map(tmul).sum();
            w_ders[order] = basis.iter().zip(&self.weights).map(tmul).sum();
        });
        let Vector2 { x: u0, y: v0 } = uv0_ders[0];
        let s0_ders = self.surface0.ders(max_order + 1, u0, v0);
        let Vector2 { x: u1, y: v1 } = uv1_ders[0];
        let s1_ders = self.surface1.ders(max_order + 1, u1, v1);

        let (lift_p0_ders, lift_q0_ders) =
            subders::control_points_ders(&s0_ders, &uv0_ders, &b0_ders, &w_ders);
        let (lift_p1_ders, lift_q1_ders) =
            subders::control_points_ders(&s1_ders, &uv1_ders, &b1_ders, &w_ders);

        let mut homog_ders = SurfaceDers::<Vector4>::new(max_order);
        homog_ders.slice_iter_mut().enumerate().for_each(|(m, o)| {
            o.iter_mut().enumerate().for_each(|(n, o)| {
                let basis = bezier_3rd_basis(m, u);
                *o = lift_p0_ders[n] * basis[0]
                    + lift_q0_ders[n] * basis[1]
                    + lift_q1_ders[n] * basis[2]
                    + lift_p1_ders[n] * basis[3];
            });
        });
        homog_ders.rat_ders()
    }
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        self.ders(m + n, u, v)[m][n]
    }
    fn subs(&self, u: f64, v: f64) -> Point3 {
        let Self {
            knot_vec,
            surface0,
            surface1,
            side_control_points0,
            side_control_points1,
            tangent_vecs0,
            tangent_vecs1,
            weights,
        } = self;
        let degree = self.vdegree();
        let basis = knot_vec.bspline_basis_functions(degree, 0, v);
        let dbasis = knot_vec.bspline_basis_functions(degree, 1, v);
        let weight: f64 = basis.iter().zip(weights).map(|(&b, &w)| b * w).sum();
        let striple0 = (surface0, side_control_points0, tangent_vecs0);
        let striple1 = (surface1, side_control_points1, tangent_vecs1);
        let (pt0, pt1) = u_control_points(&basis, &dbasis, striple0, weight);
        let (pt3, pt2) = u_control_points(&basis, &dbasis, striple1, weight);
        let b = bezier_3rd_basis(0, u);
        Point3::from_homogeneous(b[0] * pt0 + b[1] * pt1 + b[2] * pt2 + b[3] * pt3)
    }
    fn uder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(1, 0, u, v) }
    fn vder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(0, 1, u, v) }
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(2, 0, u, v) }
    fn uvder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(1, 1, u, v) }
    fn vvder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(0, 2, u, v) }
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        use std::ops::Bound::*;
        let (a, b) = (self.knot_vec[0], *self.knot_vec.last().unwrap());
        ((Included(0.0), Included(1.0)), (Included(a), Included(b)))
    }
}

impl<S0, S1> ParametricSurface3D for ApproxFilletSurface<S0, S1>
where
    S0: ParametricSurface3D,
    S1: ParametricSurface3D,
{
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        let ders = self.ders(1, u, v);
        ders[1][0].cross(ders[0][1]).normalize()
    }
}

impl<S0, S1> ParameterDivision2D for ApproxFilletSurface<S0, S1>
where
    S0: ParametricSurface3D,
    S1: ParametricSurface3D,
{
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        algo::surface::parameter_division(self, range, tol)
    }
}

impl<S0, S1> ApproxFilletSurface<S0, S1>
where
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
{
    /// approx fillet by `ApproxFilletSurface`.
    pub fn approx_rolling_ball_fillet<C, R>(
        fillet_surface: &RbfSurface<C, S0, S1, R>,
        edge_parameter_range: (f64, f64),
        tol: f64,
    ) -> Option<Self>
    where
        C: ParametricCurve3D,
        R: RadiusFunction,
    {
        let (v0, v1) = edge_parameter_range;
        let v_5 = (v0 + v1) / 2.0;
        let cc0 = fillet_surface.contact_circle(v0)?;
        let cc_5 = fillet_surface.contact_circle(v_5)?;
        let cc1 = fillet_surface.contact_circle(v1)?;
        let mut ccs = vec![(v0, cc0), (v_5, cc_5), (v1, cc1)];

        for _i in 0..16 {
            let mut vec = Vec::with_capacity(ccs.len() + 3);
            vec.extend([v0, v0, v0]);
            vec.extend({
                let n = ccs.len() - 1;
                ccs[1..n].windows(2).map(move |v| (v[0].0 + v[1].0) / 2.0)
            });
            vec.extend([v1, v1, v1]);
            let knot_vec = KnotVec::try_from(vec).unwrap();

            let make_uv0 = move |&(v, cc): &(f64, ContactCircle)| (v, cc.contact_point0().uv);
            let uv0s = ccs.iter().map(make_uv0).collect::<Vec<_>>();
            let parameter_curve0 = BSplineCurve::interpole(knot_vec.clone(), uv0s);
            let pcurve0 = PCurve::new(&parameter_curve0, fillet_surface.surface0());

            let make_uv1 = move |&(v, cc): &(f64, ContactCircle)| (v, cc.contact_point1().uv);
            let uv1s = ccs.iter().map(make_uv1).collect::<Vec<_>>();
            let parameter_curve1 = BSplineCurve::interpole(knot_vec.clone(), uv1s);
            let pcurve1 = PCurve::new(&parameter_curve1, fillet_surface.surface1());

            let mut raw_tangent_vecs0 = Vec::new();
            let mut raw_tangent_vecs1 = Vec::new();
            let mut raw_weights = Vec::new();
            for &(v, cc) in &ccs {
                let mut nurbs: NurbsCurve<Vector4> = cc.to_same_geometry();
                nurbs.elevate_degree();
                raw_weights.push((v, Point1::new(nurbs.control_point(1).w)));

                let der0 = nurbs.der(0.0);
                let cder0 = pcurve0.der(v).normalize();
                let uv0 = cc.contact_point0().uv;
                let n0 = fillet_surface.surface0.normal(uv0.x, uv0.y);
                let handle0 = cder0.cross(n0);
                let mat0 = Matrix3::from_cols(handle0, cder0, n0);
                let vec0 = mat0.invert().unwrap() * der0;
                raw_tangent_vecs0.push((v, vec0.truncate()));

                let der1 = -nurbs.der(1.0);
                let cder1 = pcurve1.der(v).normalize();
                let uv1 = cc.contact_point1().uv;
                let n1 = fillet_surface.surface1.normal(uv1.x, uv1.y);
                let handle1 = cder1.cross(n1);
                let mat1 = Matrix3::from_cols(handle1, cder1, n1);
                let vec1 = mat1.invert().unwrap() * der1;
                raw_tangent_vecs1.push((v, vec1.truncate()));
            }

            let tangent_curve0 = BSplineCurve::interpole(knot_vec.clone(), raw_tangent_vecs0);
            let tangent_curve1 = BSplineCurve::interpole(knot_vec.clone(), raw_tangent_vecs1);
            let weights_curve = BSplineCurve::interpole(knot_vec.clone(), raw_weights);
            let weights_points = weights_curve.destruct().1;
            let weights = weights_points.into_iter().map(|x| x.x).collect();

            let approx = ApproxFilletSurface {
                knot_vec,
                surface0: fillet_surface.surface0(),
                side_control_points0: parameter_curve0.destruct().1,
                tangent_vecs0: tangent_curve0.destruct().1,
                surface1: fillet_surface.surface1(),
                side_control_points1: parameter_curve1.destruct().1,
                tangent_vecs1: tangent_curve1.destruct().1,
                weights,
            };
            let added_ccs = ccs
                .windows(2)
                .filter_map(|v| {
                    let v = (v[0].0 + v[1].0) / 2.0;
                    let cc = fillet_surface.contact_circle(v).unwrap();
                    let is_far = |t: f64| approx.subs(t, v).distance2(cc.subs(t)) < tol * tol;
                    match [0.0, 0.5, 1.0].into_iter().all(is_far) {
                        true => None,
                        false => Some((v, cc)),
                    }
                })
                .collect::<Vec<_>>();
            if added_ccs.is_empty() {
                return Some(Self {
                    knot_vec: approx.knot_vec,
                    surface0: S0::clone(approx.surface0),
                    side_control_points0: approx.side_control_points0,
                    tangent_vecs0: approx.tangent_vecs0,
                    surface1: S1::clone(approx.surface1),
                    side_control_points1: approx.side_control_points1,
                    tangent_vecs1: approx.tangent_vecs1,
                    weights: approx.weights,
                });
            }
            ccs.extend(added_ccs);
            ccs.sort_by(|(x, _), (y, _)| x.partial_cmp(y).unwrap());
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{prelude::*, property_test};

    #[property_test]
    fn plane_cylinder(#[strategy = 0.0..=1.0] u: f64, #[strategy = 0.0..=1.0] v: f64) {
        let surface = ApproxFilletSurface {
            knot_vec: KnotVec::bezier_knot(1),
            surface0: Plane::xy(),
            side_control_points0: vec![(-1.0, 0.0).into(), (-1.0, 1.0).into()],
            tangent_vecs0: vec![(f64::sqrt(2.0), 0.0).into(); 2],
            surface1: Plane::yz(),
            side_control_points1: vec![(0.0, -1.0).into(), (1.0, -1.0).into()],
            tangent_vecs1: vec![(-f64::sqrt(2.0), 0.0).into(); 2],
            weights: vec![(1.0 + f64::sqrt(2.0)) / 3.0; 2],
        };
        let w = 1.0 / f64::sqrt(2.0);
        let nurbs_surface = NurbsSurface::new(BSplineSurface::<Vector4>::new(
            (KnotVec::bezier_knot(2), KnotVec::bezier_knot(1)),
            vec![
                vec![(-1.0, 0.0, 0.0, 1.0).into(), (-1.0, 1.0, 0.0, 1.0).into()],
                vec![(0.0, 0.0, 0.0, w).into(), (0.0, w, 0.0, w).into()],
                vec![(0.0, 0.0, -1.0, 1.0).into(), (0.0, 1.0, -1.0, 1.0).into()],
            ],
        ));

        prop_assert_near!(surface.subs(u, v), nurbs_surface.subs(u, v));
        prop_assert_near!(surface.ders(3, u, v), nurbs_surface.ders(3, u, v));
    }

    #[property_test]
    fn test_ders(#[strategy = 0.0..=1.0] u: f64, #[strategy = 0.0..=1.0] v: f64) {
        #[rustfmt::skip]
        let surface0 = &BSplineSurface::<Point3>::new(
            (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
            vec![
                vec![(-1.0, 0.0, 0.0).into(), (-1.0, 0.5, 0.0).into(), (-1.0, 1.0, 0.0).into()],
                vec![(-0.5, 0.0, 0.0).into(), (-0.5, 0.5, 1.0).into(), (-0.5, 1.0, 0.0).into()],
                vec![(0.0, 0.0, 0.0).into(), (0.0, 0.5, 0.0).into(), (0.0, 1.0, 0.0).into()],
            ]
        );
        #[rustfmt::skip]
        let surface1 = &BSplineSurface::<Point3>::new(
            (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
            vec![
                vec![(0.0, 0.0, -1.0).into(), (0.0, 0.0, -0.5).into(), (0.0, 0.0, 0.0).into()],
                vec![(0.0, 0.5, -1.0).into(), (1.0, 0.5, -0.5).into(), (0.0, 0.5, 0.0).into()],
                vec![(0.0, 1.0, -1.0).into(), (0.0, 1.0, -0.5).into(), (0.0, 1.0, 0.0).into()],
            ]
        );

        let surface = ApproxFilletSurface {
            knot_vec: KnotVec::bezier_knot(2),
            surface0,
            side_control_points0: vec![(0.8, 0.0).into(), (0.5, 0.5).into(), (0.8, 1.0).into()],
            tangent_vecs0: vec![(0.2, -0.1).into(), (0.4, 0.0).into(), (0.2, 0.1).into()],
            surface1,
            side_control_points1: vec![(0.0, 0.8).into(), (0.5, 0.5).into(), (1.0, 0.8).into()],
            tangent_vecs1: vec![(-0.2, -0.1).into(), (-0.4, 0.0).into(), (-0.2, 0.1).into()],
            weights: vec![1.0, 2.0, 1.0],
        };

        let pt = surface.subs(u, v);
        let ders = surface.ders(3, u, v);
        assert_near!(pt.to_vec(), ders[0][0]);

        const EPS: f64 = 1.0e-4;

        let upders = surface.ders(2, u + EPS, v);
        let umders = surface.ders(2, u - EPS, v);
        let calc_uders = upders.element_wise_ders(&umders, |x, y| x - y) / (2.0 * EPS);
        let res_uders = ders.uder();

        let iter = res_uders
            .slice_iter()
            .flatten()
            .zip(calc_uders.slice_iter().flatten());
        for (a, b) in iter {
            prop_assert!((a - b).magnitude() < 10.0 * EPS);
        }

        let vpders = surface.ders(2, u, v + EPS);
        let vmders = surface.ders(2, u, v - EPS);
        let calc_vders = vpders.element_wise_ders(&vmders, |x, y| x - y) / (2.0 * EPS);
        let res_vders = ders.vder();

        let iter = res_vders
            .slice_iter()
            .flatten()
            .zip(calc_vders.slice_iter().flatten());
        for (a, b) in iter {
            prop_assert!((a - b).magnitude() < 10.0 * EPS);
        }

        let pt0 = surface.subs(0.0, v);
        let (u0, v0) = surface0.search_parameter(pt0, (0.5, 0.5), 100).unwrap();
        assert_near!(surface.normal(0.0, v), surface0.normal(u0, v0));

        let pt1 = surface.subs(1.0, v);
        let (u1, v1) = surface1.search_parameter(pt1, (0.5, 0.5), 100).unwrap();
        assert_near!(surface.normal(1.0, v), surface1.normal(u1, v1));
    }
}
