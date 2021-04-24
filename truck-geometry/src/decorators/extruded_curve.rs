use crate::*;

impl<C, V: Copy> ExtrudedCurve<C, V> {
    /// Creates a linear extruded curve by extrusion.
    #[inline(always)]
    pub fn by_extrusion(curve: C, vector: V) -> Self { Self { curve, vector } }

    /// Returns the curve before extruded.
    #[inline(always)]
    pub fn entity_curve(&self) -> &C { &self.curve }
    /// Into the curve before revoluted.
    #[inline(always)]
    pub fn into_entity_curve(self) -> C { self.curve }

    /// Returns the vector of extruded curve.
    #[inline(always)]
    pub fn extruding_vector(&self) -> V { self.vector }
}

impl<C> ParametricSurface for ExtrudedCurve<C, Vector3>
where C: ParametricCurve<Point = Point3, Vector = Vector3>
{
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> C::Point { self.curve.subs(u) + self.vector * v }
    #[inline(always)]
    fn uder(&self, u: f64, _: f64) -> C::Vector { self.curve.der(u) }
    #[inline(always)]
    fn vder(&self, _: f64, _: f64) -> C::Vector { self.vector }
    #[inline(always)]
    fn uuder(&self, u: f64, _: f64) -> C::Vector { self.curve.der2(u) }
    #[inline(always)]
    fn uvder(&self, _: f64, _: f64) -> C::Vector { C::Vector::zero() }
    #[inline(always)]
    fn vvder(&self, _: f64, _: f64) -> C::Vector { C::Vector::zero() }
    #[inline(always)]
    fn normal(&self, u: f64, _: f64) -> C::Vector {
        self.curve.der(u).cross(self.vector).normalize()
    }
}

impl<C> ParametricSurface for ExtrudedCurve<C, Vector2>
where C: ParametricCurve<Point = Point2, Vector = Vector2>
{
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> C::Point { self.curve.subs(u) + self.vector * v }
    #[inline(always)]
    fn uder(&self, u: f64, _: f64) -> C::Vector { self.curve.der(u) }
    #[inline(always)]
    fn vder(&self, _: f64, _: f64) -> C::Vector { self.vector }
    #[inline(always)]
    fn uuder(&self, u: f64, _: f64) -> C::Vector { self.curve.der2(u) }
    #[inline(always)]
    fn uvder(&self, _: f64, _: f64) -> C::Vector { C::Vector::zero() }
    #[inline(always)]
    fn vvder(&self, _: f64, _: f64) -> C::Vector { C::Vector::zero() }
    /// zero identity
    #[inline(always)]
    fn normal(&self, _: f64, _: f64) -> C::Vector { C::Vector::zero() }
}

impl<C, V> BoundedSurface for ExtrudedCurve<C, V>
where
    C: ParametricCurve,
    Self: ParametricSurface,
{
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) {
        (self.curve.parameter_range(), (0.0, 1.0))
    }
}

#[test]
fn extruded_curve_test() {
    let cpts = vec![
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
    ];
    let spts = vec![
        vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ],
        vec![
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 1.0, 1.0),
        ],
        vec![
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 1.0),
        ],
    ];
    let curve = BSplineCurve::new(KnotVec::bezier_knot(2), cpts);
    let surface0 = ExtrudedCurve::by_extrusion(curve, Vector3::unit_z());
    let surface1 = BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(1)), spts);
    assert_eq!(surface0.parameter_range(), surface1.parameter_range());
    
    const N: usize = 10;
    for i in 0..=N {
        for j in 0..=N {
            let u = i as f64 / N as f64;
            let v = j as f64 / N as f64;
            assert_near!(surface0.subs(u, v), ParametricSurface::subs(&surface1, u, v));
            assert_near!(surface0.uder(u, v), surface1.uder(u, v));
            assert_near!(surface0.vder(u, v), surface1.vder(u, v));
            assert_near!(surface0.uuder(u, v), surface1.uuder(u, v));
            assert_near!(surface0.uvder(u, v), surface1.uvder(u, v));
            assert_near!(surface0.vvder(u, v), surface1.vvder(u, v));
            assert_near!(surface0.normal(u, v), surface1.normal(u, v));
        }
    }
}
