use crate::*;

impl Curve for BSplineCurve<Vector2> {
    type Point = Vector2;
    #[inline(always)]
    fn front(&self) -> Self::Point {
        let t = self.knot_vec()[0];
        self.subs(t)
    }
    #[inline(always)]
    fn back(&self) -> Self::Point {
        let knot_vec = self.knot_vec();
        let t = knot_vec[0] + knot_vec.range_length();
        self.subs(t)
    }
    #[inline(always)]
    fn is_arc_of(&self, longer: &Self, hint: f64) -> Option<f64> {
        self.is_arc_of(longer, hint)
    }
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut curve = self.clone();
        curve.invert();
        curve
    }
}

impl Curve for BSplineCurve<Vector3> {
    type Point = Vector3;
    #[inline(always)]
    fn front(&self) -> Self::Point {
        let t = self.knot_vec()[0];
        self.subs(t)
    }
    #[inline(always)]
    fn back(&self) -> Self::Point {
        let knot_vec = self.knot_vec();
        let t = knot_vec[0] + knot_vec.range_length();
        self.subs(t)
    }
    #[inline(always)]
    fn is_arc_of(&self, longer: &Self, hint: f64) -> Option<f64> {
        self.is_arc_of(longer, hint)
    }
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut curve = self.clone();
        curve.invert();
        curve
    }
}

impl Curve for BSplineCurve<Vector4> {
    type Point = Vector4;
    #[inline(always)]
    fn front(&self) -> Self::Point {
        let t = self.knot_vec()[0];
        self.subs(t)
    }
    #[inline(always)]
    fn back(&self) -> Self::Point {
        let knot_vec = self.knot_vec();
        let t = knot_vec[0] + knot_vec.range_length();
        self.subs(t)
    }
    #[inline(always)]
    fn is_arc_of(&self, longer: &Self, hint: f64) -> Option<f64> {
        self.is_rational_arc_of(longer, hint)
    }
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut curve = self.clone();
        curve.invert();
        curve
    }
}
