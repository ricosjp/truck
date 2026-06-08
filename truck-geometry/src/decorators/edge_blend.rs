use super::*;

impl<C0, S0, C1, S1, F0, F1> EdgeBlendSurface<C0, S0, F0, C1, S1, F1> {
    /// Constructor
    #[inline]
    pub fn new(
        pcurve0: PCurve<C0, S0>,
        magnitude0: F0,
        pcurve1: PCurve<C1, S1>,
        magnitude1: F1,
    ) -> Self {
        Self {
            pcurve0,
            magnitude0,
            pcurve1,
            magnitude1,
        }
    }
    /// Returns the first pcurve.
    #[inline]
    pub fn pcurve0(&self) -> &PCurve<C0, S0> { &self.pcurve0 }
    /// Returns the second pcurve.
    #[inline]
    pub fn pcurve1(&self) -> &PCurve<C1, S1> { &self.pcurve1 }
    /// Returns the first pcurve.
    #[inline]
    pub fn pcurve0_mut(&mut self) -> &mut PCurve<C0, S0> { &mut self.pcurve0 }
    /// Returns the second pcurve.
    #[inline]
    pub fn pcurve1_mut(&mut self) -> &mut PCurve<C1, S1> { &mut self.pcurve1 }
    /// Returns the first magnitude.
    #[inline]
    pub fn magnitude0(&self) -> &F0 { &self.magnitude0 }
    /// Returns the second magnitude.
    #[inline]
    pub fn magnitude1(&self) -> &F1 { &self.magnitude1 }
    /// Returns the first magnitude.
    #[inline]
    pub fn magnitude0_mut(&mut self) -> &mut F0 { &mut self.magnitude0 }
    /// Returns the second magnitude.
    #[inline]
    pub fn magnitude1_mut(&mut self) -> &mut F1 { &mut self.magnitude1 }
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

fn normalized_ders(ders: &CurveDers<Vector3>) -> CurveDers<Vector3> {
    ders.element_wise_ders(&ders.abs_ders(), Vector3::extend)
        .rat_ders()
}

fn pcurve_normal_ders<C, S>(
    pcurve: &PCurve<C, S>,
    max_order: usize,
    u: f64,
) -> (CurveDers<Vector3>, CurveDers<Vector3>)
where
    C: ParametricCurve2D,
    S: ParametricSurface3D,
{
    let cders = pcurve.curve().ders(max_order + 1, u);
    let Vector2 { x, y } = cders[0];
    let sders = pcurve.surface().ders(max_order + 1, x, y);
    let pders = sders.composite_ders(&cders);
    let uders = sders.uder().composite_ders(&cders);
    let vders = sders.vder().composite_ders(&cders);
    let normal_ders = uders.combinatorial_ders(&vders, Vector3::cross);
    (pders, normal_ders)
}

fn tangent_ders(
    pders: &CurveDers<Vector3>,
    normal_ders: &CurveDers<Vector3>,
    magnitude_ders: &CurveDers<f64>,
) -> CurveDers<Vector3> {
    let axis_ders = pders.der().combinatorial_ders(normal_ders, Vector3::cross);
    normalized_ders(&axis_ders)
        .combinatorial_ders(magnitude_ders, |axis, magnitude| axis * magnitude)
}

fn edge_control_point_ders<C, S, F>(
    pcurve: &PCurve<C, S>,
    magnitude: &F,
    max_order: usize,
    u: f64,
) -> (CurveDers<Vector3>, CurveDers<Vector3>)
where
    C: ParametricCurve2D,
    S: ParametricSurface3D,
    F: ScalarFunctionD1,
{
    let (pders, normal_ders) = pcurve_normal_ders(pcurve, max_order, u);
    let tangent_ders = tangent_ders(&pders, &normal_ders, &magnitude.ders(max_order, u)) / 3.0;
    (pders, tangent_ders)
}

impl<C0, S0, F0, C1, S1, F1> ParametricSurface for EdgeBlendSurface<C0, S0, F0, C1, S1, F1>
where
    C0: ParametricCurve2D,
    S0: ParametricSurface3D,
    F0: ScalarFunctionD1,
    C1: ParametricCurve2D,
    S1: ParametricSurface3D,
    F1: ScalarFunctionD1,
{
    type Point = Point3;
    type Vector = Vector3;
    fn ders(&self, max_order: usize, u: f64, v: f64) -> SurfaceDers<Self::Vector> {
        let (pders0, tangent_ders0) =
            edge_control_point_ders(&self.pcurve0, &self.magnitude0, max_order, u);
        let (pders1, tangent_ders1) =
            edge_control_point_ders(&self.pcurve1, &self.magnitude1, max_order, u);
        let mut ders = SurfaceDers::new(max_order);
        ders.slice_iter_mut().enumerate().for_each(|(m, ders)| {
            ders.iter_mut().enumerate().for_each(|(n, der)| {
                let basis = bezier_3rd_basis(n, v);
                let q0 = pders0[m];
                let q1 = pders0[m] + tangent_ders0[m];
                let q2 = pders1[m] - tangent_ders1[m];
                let q3 = pders1[m];
                *der = q0 * basis[0] + q1 * basis[1] + q2 * basis[2] + q3 * basis[3];
            });
        });
        ders
    }
    #[inline]
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        self.ders(m + n, u, v)[m][n]
    }
    #[inline]
    fn subs(&self, u: f64, v: f64) -> Self::Point { Point3::from_vec(self.ders(0, u, v)[0][0]) }
    #[inline]
    fn uder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(1, 0, u, v) }
    #[inline]
    fn vder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(0, 1, u, v) }
    #[inline]
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(2, 0, u, v) }
    #[inline]
    fn uvder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(1, 1, u, v) }
    #[inline]
    fn vvder(&self, u: f64, v: f64) -> Self::Vector { self.der_mn(0, 2, u, v) }
    #[inline]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        let range0 = self.pcurve0.parameter_range();
        let range1 = self.pcurve1.parameter_range();
        let range = range_common_part(&range0, &range1);
        (range, (Bound::Included(0.0), Bound::Included(1.0)))
    }
}

impl<C0, S0, F0, C1, S1, F1> ParametricSurface3D for EdgeBlendSurface<C0, S0, F0, C1, S1, F1>
where
    C0: ParametricCurve2D,
    S0: ParametricSurface3D,
    F0: ScalarFunctionD1,
    C1: ParametricCurve2D,
    S1: ParametricSurface3D,
    F1: ScalarFunctionD1,
{
}

impl<C0, S0, F0, C1, S1, F1> BoundedSurface for EdgeBlendSurface<C0, S0, F0, C1, S1, F1>
where
    C0: BoundedCurve + ParametricCurve2D,
    S0: ParametricSurface3D,
    F0: ScalarFunctionD1,
    C1: BoundedCurve + ParametricCurve2D,
    S1: ParametricSurface3D,
    F1: ScalarFunctionD1,
{
}

impl<C0, S0, F0, C1, S1, F1> ParameterDivision2D for EdgeBlendSurface<C0, S0, F0, C1, S1, F1>
where
    C0: ParametricCurve2D,
    S0: ParametricSurface3D,
    F0: ScalarFunctionD1,
    C1: ParametricCurve2D,
    S1: ParametricSurface3D,
    F1: ScalarFunctionD1,
{
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        algo::surface::parameter_division(self, range, tol)
    }
}

impl<C0, S0, F0, C1, S1, F1> SearchNearestParameter<D2> for EdgeBlendSurface<C0, S0, F0, C1, S1, F1>
where
    C0: BoundedCurve + ParametricCurve2D,
    S0: ParametricSurface3D,
    F0: ScalarFunctionD1,
    C1: BoundedCurve + ParametricCurve2D,
    S1: ParametricSurface3D,
    F1: ScalarFunctionD1,
{
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SPHint2D::Parameter(x, y) => (x, y),
            SPHint2D::Range(range0, range1) => {
                algo::surface::presearch(self, point, (range0, range1), PRESEARCH_DIVISION)
            }
            SPHint2D::None => {
                algo::surface::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::surface::search_nearest_parameter(self, point, hint, trials)
    }
}

impl<C0, S0, F0, C1, S1, F1> SearchParameter<D2> for EdgeBlendSurface<C0, S0, F0, C1, S1, F1>
where
    C0: BoundedCurve + ParametricCurve2D,
    S0: ParametricSurface3D,
    F0: ScalarFunctionD1,
    C1: BoundedCurve + ParametricCurve2D,
    S1: ParametricSurface3D,
    F1: ScalarFunctionD1,
{
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = match hint.into() {
            SPHint2D::Parameter(x, y) => (x, y),
            SPHint2D::Range(range0, range1) => {
                algo::surface::presearch(self, point, (range0, range1), PRESEARCH_DIVISION)
            }
            SPHint2D::None => {
                algo::surface::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::surface::search_parameter(self, point, hint, trials)
    }
}
