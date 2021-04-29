use crate::*;

impl<C, S> PCurve<C, S> {
    /// Creates composited
    #[inline(always)]
    pub fn new(curve: C, surface: S) -> PCurve<C, S> { PCurve { curve, surface } }

    /// Returns the reference to the previous map
    #[inline(always)]
    pub fn curve(&self) -> &C { &self.curve }

    /// Returns the reference to the previous map
    #[inline(always)]
    pub fn surface(&self) -> &S { &self.surface }
}

impl<C, S> ParametricCurve for PCurve<C, S>
where
    C: ParametricCurve<Point = Point2, Vector = Vector2>,
    S: ParametricSurface,
    S::Vector: VectorSpace<Scalar = f64>,
{
    type Point = S::Point;
    type Vector = S::Vector;
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point {
        let pt = self.curve.subs(t);
        self.surface.subs(pt[0], pt[1])
    }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector {
        let pt = self.curve.subs(t);
        let der = self.curve.der(t);
        self.surface.uder(pt[0], pt[1]) * der[0] + self.surface.vder(pt[0], pt[1]) * der[1]
    }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector {
        let pt = self.curve.subs(t);
        let der = self.curve.der(t);
        let der2 = self.curve.der2(t);
        self.surface.uuder(pt[0], pt[1]) * der[0] * der[0]
            + self.surface.uvder(pt[0], pt[1]) * der[0] * der[1] * 2.0
            + self.surface.vvder(pt[0], pt[1]) * der[1] * der[1]
            + self.surface.uder(pt[0], pt[1]) * der2[0]
            + self.surface.vder(pt[0], pt[1]) * der2[1]
    }
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) { self.curve.parameter_range() }
}

impl<C, S> ParameterDivision1D for PCurve<C, S>
where
    C: ParametricCurve<Point = Point2, Vector = Vector2>,
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>,
    S::Vector: VectorSpace<Scalar = f64>,
{
    fn parameter_division(&self, tol: f64) -> Vec<f64> {
        algo::curve::parameter_division(self, self.parameter_range(), tol)
    }
}

#[test]
fn pcurve_test() {
    let curve = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector2::new(1.0, 1.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, 0.0),
        ],
    );
    let surface = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(1)),
        vec![
            vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)],
            vec![Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 1.0)],
            vec![Vector3::new(1.0, 0.0, 1.0), Vector3::new(1.0, 1.0, 1.0)],
        ]
    );
    let pcurve = PCurve::new(curve, surface);
    assert_eq!(pcurve.parameter_range(), (0.0, 1.0));

    const N: usize = 100;
    for i in 0..=N {
        let t = i as f64 / N as f64;
        assert_near!(
            pcurve.subs(t),
            Point3::new(
                (1.0 - t * t) * (1.0 - t * t),
                (1.0 - t) * (1.0 - t),
                1.0 - t * t * t * t,
            ),
        );
        assert_near!(
            pcurve.der(t),
            Vector3::new(
                4.0 * t * (t * t - 1.0),
                2.0 * (t - 1.0),
                -4.0 * t * t * t,
            ),
        );
        assert_near!(
            pcurve.der2(t),
            Vector3::new(
                4.0 * (3.0 * t * t - 1.0),
                2.0,
                -12.0 * t * t,
            ),
        );
    }
}
