use super::*;

impl<S0, S1> ApproxFilletSurface<S0, S1> {
    /// Returns the first surface.
    pub const fn surface0(&self) -> &S0 { &self.surface0 }
    /// Returns the second surface.
    pub const fn surface1(&self) -> &S1 { &self.surface1 }
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

fn side_control_points<S: ParametricSurface3D>(
    basis: &[f64],
    dbasis: &[f64],
    surface: S,
    side_control_points: &[Point2],
    tangent_vecs: &[Vector2],
) -> (Point3, Point3) {
    let iter = basis.iter().zip(side_control_points);
    let uv = iter.fold(Vector2::zero(), |sum, (&b, &p)| sum + b * p.to_vec());
    let pt0 = surface.subs(uv.x, uv.y);
    let uder = surface.uder(uv.x, uv.y);
    let vder = surface.vder(uv.x, uv.y);
    let diter = dbasis.iter().zip(side_control_points);
    let duv = diter.fold(Vector2::zero(), |sum, (&b, &p)| sum + b * p.to_vec());
    let cder = (uder * duv.x + vder * duv.y).normalize();
    let n = uder.cross(vder).normalize();
    let axis = cder.cross(n);
    let titer = basis.iter().zip(tangent_vecs);
    let tuv = titer.fold(Vector2::zero(), |sum, (&b, &v)| sum + b * v);
    let pt1 = pt0 + axis * tuv.x + cder * tuv.y;
    (pt0, pt1)
}

/*
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

impl<S0, S1> ParametricSurface for ApproxFilletSurface<S0, S1>
where
    S0: ParametricSurface3D,
    S1: ParametricSurface3D,
{
    type Point = Point3;
    type Vector = Vector3;
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        let degree = self.vdegree();
        let basis = (0..=n)
            .map(|i| self.knot_vec.bspline_basis_functions(degree, i, v))
            .collect::<Vec<_>>();
        let duv0 = (0..=n)
            .map(|i| {
                self.side_control_points0
                    .iter()
                    .zip(&basis[i])
                    .map(|(&p, &b)| b * p.to_vec())
                    .sum::<Vector2>()
            })
            .collect::<Vec<_>>();
        let mut sders0 = (0..=n)
            .map(|i| vec![Vector3::zero(); n - i + 1])
            .collect::<Vec<_>>();
        self.surface0.ders(duv0[0].x, duv0[0].y, &mut sders0);
        let dp0 = (0..=n)
            .map(|i| pcurve::raw_der_n(&sders0, &duv0, i))
            .collect::<Vec<_>>();
        let db0 = (0..=n)
            .map(|i| {
                self.tangent_vecs0
                    .iter()
                    .zip(&basis[i])
                    .map(|(&p, &b)| b * p)
                    .sum::<Vector2>()
            })
            .collect::<Vec<_>>();
        Vector3::zero()
    }
    fn subs(&self, u: f64, v: f64) -> Point3 {
        let degree = self.vdegree();
        let basis = self.knot_vec.bspline_basis_functions(degree, 0, v);
        let dbasis = self.knot_vec.bspline_basis_functions(degree, 1, v);
        let (pt0, pt1) = side_control_points(
            &basis,
            &dbasis,
            &self.surface0,
            &self.side_control_points0,
            &self.tangent_vecs0,
        );
        let (pt3, pt2) = side_control_points(
            &basis,
            &dbasis,
            &self.surface1,
            &self.side_control_points1,
            &self.tangent_vecs1,
        );
        let witer = basis.iter().zip(&self.weights);
        let w = witer.fold(0.0, |sum, (&b, &w)| sum + b * w);
        let b = [
            (1.0 - u).powi(3),
            3.0 * (1.0 - u).powi(2) * u,
            3.0 * (1.0 - u) * u.powi(2),
            u.powi(3),
        ];
        let pts = [
            pt0.to_homogeneous(),
            w * pt1.to_homogeneous(),
            w * pt2.to_homogeneous(),
            pt3.to_homogeneous(),
        ];
        let viter = b.into_iter().zip(pts);
        let vec = viter.fold(Vector4::zero(), |sum, (b, v)| sum + b * v);
        Point3::from_homogeneous(vec)
    }
}
    */
