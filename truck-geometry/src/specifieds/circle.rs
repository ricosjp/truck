use super::*;
use std::f64::consts::PI;

impl<P> UnitCircle<P> {
    /// constructor
    #[inline]
    pub const fn new() -> Self { Self(std::marker::PhantomData) }
}

impl ParametricCurve for UnitCircle<Point2> {
    type Point = Point2;
    type Vector = Vector2;
    #[inline]
    fn der_n(&self, t: f64, n: usize) -> Vector2 {
        match n % 4 {
            0 => Vector2::new(f64::cos(t), f64::sin(t)),
            1 => Vector2::new(-f64::sin(t), f64::cos(t)),
            2 => Vector2::new(-f64::cos(t), -f64::sin(t)),
            _ => Vector2::new(f64::sin(t), -f64::cos(t)),
        }
    }
    #[inline]
    fn subs(&self, t: f64) -> Point2 { Point2::from_vec(self.der_n(t, 0)) }
    #[inline]
    fn der(&self, t: f64) -> Vector2 { self.der_n(t, 1) }
    #[inline]
    fn der2(&self, t: f64) -> Vector2 { self.der_n(t, 2) }
    #[inline]
    fn parameter_range(&self) -> ParameterRange {
        (Bound::Included(0.0), Bound::Excluded(2.0 * PI))
    }
}

impl BoundedCurve for UnitCircle<Point2> {}

impl ParametricCurve for UnitCircle<Point3> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline]
    fn der_n(&self, t: f64, n: usize) -> Vector3 {
        match n % 4 {
            0 => Vector3::new(f64::cos(t), f64::sin(t), 0.0),
            1 => Vector3::new(-f64::sin(t), f64::cos(t), 0.0),
            2 => Vector3::new(-f64::cos(t), -f64::sin(t), 0.0),
            _ => Vector3::new(f64::sin(t), -f64::cos(t), 0.0),
        }
    }
    #[inline]
    fn subs(&self, t: f64) -> Point3 { Point3::from_vec(self.der_n(t, 0)) }
    #[inline]
    fn der(&self, t: f64) -> Vector3 { self.der_n(t, 1) }
    #[inline]
    fn der2(&self, t: f64) -> Vector3 { self.der_n(t, 2) }
    #[inline]
    fn period(&self) -> Option<f64> { Some(2.0 * PI) }
    #[inline]
    fn parameter_range(&self) -> ParameterRange {
        (Bound::Included(0.0), Bound::Excluded(2.0 * PI))
    }
}

impl BoundedCurve for UnitCircle<Point3> {}

impl<P> ParameterDivision1D for UnitCircle<P>
where UnitCircle<P>: ParametricCurve<Point = P>
{
    type Point = P;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<P>) {
        nonpositive_tolerance!(tol);
        let tol = f64::min(tol, 0.8);
        let delta = 2.0 * f64::acos(1.0 - tol);
        let n = 1 + ((range.1 - range.0) / delta) as usize;
        let params = (0..=n)
            .map(|i| {
                let t = i as f64 / n as f64;
                range.0 * (1.0 - t) + range.1 * t
            })
            .collect::<Vec<_>>();
        let pts = params.iter().map(|t| self.subs(*t)).collect();
        (params, pts)
    }
}

impl SearchNearestParameter<D1> for UnitCircle<Point2> {
    type Point = Point2;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        pt: Point2,
        _: H,
        _: usize,
    ) -> Option<f64> {
        let v = pt.to_vec();
        if v.magnitude().so_small() {
            return None;
        }
        let v = v.normalize();
        let theta = f64::acos(f64::clamp(v.x, -1.0, 1.0));
        let theta = match v.y > 0.0 {
            true => theta,
            false => 2.0 * PI - theta,
        };
        Some(theta)
    }
}

impl SearchParameter<D1> for UnitCircle<Point2> {
    type Point = Point2;
    fn search_parameter<H: Into<SPHint1D>>(&self, pt: Point2, _: H, _: usize) -> Option<f64> {
        let v = pt.to_vec();
        if !v.magnitude().near(&1.0) {
            return None;
        }
        let v = v.normalize();
        let theta = f64::acos(f64::clamp(v.x, -1.0, 1.0));
        let theta = match v.y > 0.0 {
            true => theta,
            false => 2.0 * PI - theta,
        };
        Some(theta)
    }
}

impl SearchNearestParameter<D1> for UnitCircle<Point3> {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        pt: Point3,
        _: H,
        _: usize,
    ) -> Option<f64> {
        UnitCircle::<Point2>::new().search_nearest_parameter(Point2::new(pt.x, pt.y), None, 0)
    }
}

impl SearchParameter<D1> for UnitCircle<Point3> {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint1D>>(&self, pt: Point3, _: H, _: usize) -> Option<f64> {
        if !f64::abs(pt.z).so_small() {
            return None;
        }
        UnitCircle::<Point2>::new().search_parameter(Point2::new(pt.x, pt.y), None, 0)
    }
}

impl ToSameGeometry<NurbsCurve<Vector3>> for TrimmedCurve<UnitCircle<Point2>> {
    fn to_same_geometry(&self) -> NurbsCurve<Vector3> {
        let (t0, t1) = self.range_tuple();
        let angle = t1 - t0;
        let (cos2, sin2) = (f64::cos(angle / 2.0), f64::sin(angle / 2.0));
        let rot = Matrix3::from(Matrix2::from_angle(Rad(t0)));
        NurbsCurve::new(BSplineCurve::new_unchecked(
            KnotVec::bezier_knot(2),
            vec![
                rot * Vector3::new(1.0, 0.0, 1.0),
                rot * Vector3::new(cos2, sin2, cos2),
                rot * Vector3::new(f64::cos(angle), f64::sin(angle), 1.0),
            ],
        ))
    }
}

impl ToSameGeometry<NurbsCurve<Vector4>> for TrimmedCurve<UnitCircle<Point3>> {
    fn to_same_geometry(&self) -> NurbsCurve<Vector4> {
        let (t0, t1) = self.range_tuple();
        let bsp: NurbsCurve<Vector3> =
            TrimmedCurve::new(UnitCircle::<Point2>::new(), (t0, t1)).to_same_geometry();
        let (knot_vec, pts) = bsp.into_non_rationalized().destruct();
        let mut curve = NurbsCurve::new(BSplineCurve::new_unchecked(
            knot_vec,
            vec![
                Vector4::new(pts[0].x, pts[0].y, 0.0, pts[0].z),
                Vector4::new(pts[1].x, pts[1].y, 0.0, pts[1].z),
                Vector4::new(pts[2].x, pts[2].y, 0.0, pts[2].z),
            ],
        ));
        curve.add_knot(0.25);
        curve.add_knot(0.5);
        curve.add_knot(0.75);
        curve
    }
}
