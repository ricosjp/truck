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
    fn subs(&self, u: f64, v: f64) -> Self::Point {
        let cders0 = self.pcurve0.curve().ders(1, u);
        let Vector2 { x: u0, y: v0 } = cders0[0];
        let ders0 = self.pcurve0.surface().ders(1, u0, v0);
        let pcder0 = ders0.composite_der(&cders0, 1);
        let normal0 = ders0[1][0].cross(ders0[0][1]);
        let tangent0 = pcder0.cross(normal0).normalize() * self.magnitude0.subs(u);

        let cders1 = self.pcurve1.curve().ders(1, u);
        let Vector2 { x: u1, y: v1 } = cders1[0];
        let ders1 = self.pcurve1.surface().ders(1, u1, v1);
        let pcder1 = ders1.composite_der(&cders1, 1);
        let normal1 = ders1[1][0].cross(ders1[0][1]);
        let tangent1 = pcder1.cross(normal1).normalize() * self.magnitude1.subs(u);

        let p0 = pcder0;
        let p1 = pcder0 + tangent0 / 3.0;
        let p2 = pcder1 - tangent1 / 3.0;
        let p3 = pcder1;
        Point3::from_vec(
            p0 * (1.0 - v).powi(3)
                + p1 * 3.0 * (1.0 - v) * (1.0 - v) * v
                + p2 * 3.0 * (1.0 - v) * v * v
                + p3 * v.powi(3),
        )
    }
}
