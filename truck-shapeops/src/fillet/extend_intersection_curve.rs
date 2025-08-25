use truck_geometry::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct CurveConnection<C, Prev, Next> {
    curve: C,
    prev: Prev,
    next: Next,
}

impl<C, Prev, Next> ParametricCurve for CurveConnection<C, Prev, Next>
where
    C: BoundedCurve,
    C::Vector: VectorSpace<Scalar = f64>,
    Prev: ParametricCurve<Point = C::Point, Vector = C::Vector>,
    Next: ParametricCurve<Point = C::Point, Vector = C::Vector>,
{
    type Point = C::Point;
    type Vector = C::Vector;
    fn subs(&self, t: f64) -> Self::Point {
        let (t0, t1) = self.curve.range_tuple();
        if t < t0 {
            self.prev.subs(t0 - t)
        } else if t1 < t {
            self.next.subs(t - t1)
        } else {
            self.curve.subs(t)
        }
    }
    fn der(&self, t: f64) -> Self::Vector {
        let (t0, t1) = self.curve.range_tuple();
        if t < t0 {
            self.prev.der(t0 - t) * (-1.0)
        } else if t1 < t {
            self.next.der(t - t1)
        } else {
            self.curve.der(t)
        }
    }
    fn der2(&self, t: f64) -> Self::Vector {
        let (t0, t1) = self.curve.range_tuple();
        if t < t0 {
            self.prev.der2(t0 - t)
        } else if t1 < t {
            self.next.der2(t - t1)
        } else {
            self.curve.der2(t)
        }
    }
    fn der_n(&self, n: usize, t: f64) -> Self::Vector {
        let (t0, t1) = self.curve.range_tuple();
        if t < t0 {
            self.prev.der_n(n, t0 - t) * f64::powi(-1.0, n as i32)
        } else if t1 < t {
            self.next.der_n(n, t - t1)
        } else {
            self.curve.der_n(n, t)
        }
    }
    fn ders(&self, n: usize, t: f64) -> CurveDers<Self::Vector> {
        let (t0, t1) = self.curve.range_tuple();
        if t < t0 {
            let mut ders = self.prev.ders(n, t0 - t);
            let mut s = 1.0;
            ders.iter_mut().for_each(|der| {
                *der = *der * s;
                s = -s;
            });
            ders
        } else if t1 < t {
            self.next.ders(n, t - t1)
        } else {
            self.curve.ders(n, t)
        }
    }
}

pub(crate) fn extend_intersection_curve<C, S0, S1>(
    curve: C,
    surface0: S0,
    surface1: S1,
) -> CurveConnection<
    C,
    IntersectionCurve<Line<Point3>, S0, S1>,
    IntersectionCurve<Line<Point3>, S0, S1>,
>
where
    C: ParametricCurve3D + BoundedCurve,
    S0: Clone,
    S1: Clone,
{
    let (t0, t1) = curve.range_tuple();
    let ders0 = curve.ders(1, t0);
    let (p0, v0) = (Point3::from_vec(ders0[0]), ders0[1]);
    let line0 = Line(p0, p0 - v0);
    let ders1 = curve.ders(1, t1);
    let (p1, v1) = (Point3::from_vec(ders1[0]), ders1[1]);
    let line1 = Line(p1, p1 + v1);
    CurveConnection {
        curve,
        prev: IntersectionCurve::new(surface0.clone(), surface1.clone(), line0),
        next: IntersectionCurve::new(surface0, surface1, line1),
    }
}

#[test]
fn test_extend_intersection_curve() {
    let plane0 = Plane::xy();
    let plane1 = Plane::yz();
    let line = Line(Point3::origin(), Point3::new(0.0, 1.0, 0.0));
    let curve = extend_intersection_curve(line, plane0, plane1);
    assert_near!(curve.subs(1.2), Point3::new(0.0, 1.2, 0.0));
    assert_near!(curve.subs(-4.0), Point3::new(0.0, -4.0, 0.0));
    assert_near!(curve.subs(0.6), Point3::new(0.0, 0.6, 0.0));
}
