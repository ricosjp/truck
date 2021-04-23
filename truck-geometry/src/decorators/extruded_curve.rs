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
