use super::*;
use algo::surface::{SsnpVector, SspVector};
use control_point::ControlPoint;
use std::ops::RangeBounds;

impl<C0, C1> HomotopySurface<C0, C1> {
    /// constructor
    #[inline(always)]
    pub fn new(curve0: C0, curve1: C1) -> Self { Self { curve0, curve1 } }
    /// Returns the first curve.
    #[inline(always)]
    pub fn first_curve(&self) -> &C0 { &self.curve0 }
    /// Returns the second curve.
    #[inline(always)]
    pub fn second_curve(&self) -> &C1 { &self.curve1 }
    /// Returns the first curve.
    #[inline(always)]
    pub fn first_curve_mut(&mut self) -> &mut C0 { &mut self.curve0 }
    /// Returns the second curve.
    #[inline(always)]
    pub fn second_curve_mut(&mut self) -> &mut C1 { &mut self.curve1 }
}

impl<C0, C1> ParametricSurface for HomotopySurface<C0, C1>
where
    C0: ParametricCurve,
    C1: ParametricCurve<Point = C0::Point, Vector = C0::Vector>,
    C0::Point: EuclideanSpace<Scalar = f64, Diff = C0::Vector>,
    C0::Vector: VectorSpace<Scalar = f64>,
{
    type Point = C0::Point;
    type Vector = C0::Vector;
    #[inline(always)]
    fn der_mn(&self, u: f64, v: f64, m: usize, n: usize) -> Self::Vector {
        match (m, n) {
            (_, 0) => {
                let v0 = self.curve0.der_n(u, m);
                let v1 = self.curve1.der_n(u, m);
                v0 + (v1 - v0) * v
            }
            (_, 1) => {
                let v0 = self.curve0.der_n(u, m);
                let v1 = self.curve1.der_n(u, m);
                v1 - v0
            }
            _ => Self::Vector::zero(),
        }
    }
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point {
        let p0 = self.curve0.subs(u);
        let p1 = self.curve1.subs(u);
        p0 + (p1 - p0) * v
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector {
        let v0 = self.curve0.der(u);
        let v1 = self.curve1.der(u);
        v0 + (v1 - v0) * v
    }
    #[inline(always)]
    fn vder(&self, u: f64, _: f64) -> Self::Vector { self.curve1.subs(u) - self.curve0.subs(u) }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Self::Vector {
        let v0 = self.curve0.der2(u);
        let v1 = self.curve1.der2(u);
        v0 + (v1 - v0) * v
    }
    #[inline(always)]
    fn uvder(&self, u: f64, _: f64) -> Self::Vector { self.curve1.der(u) - self.curve0.der(u) }
    #[inline(always)]
    fn vvder(&self, _: f64, _: f64) -> Self::Vector { Self::Vector::zero() }
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        let range0 = self.curve0.parameter_range();
        let range1 = self.curve1.parameter_range();
        let range = range_common_part(&range0, &range1);
        (range, (Bound::Included(0.0), Bound::Included(1.0)))
    }
}

impl<C0, C1> ParametricSurface3D for HomotopySurface<C0, C1>
where
    C0: ParametricCurve3D,
    C1: ParametricCurve3D,
{
}

impl<C0, C1> BoundedSurface for HomotopySurface<C0, C1>
where
    C0: BoundedCurve,
    C1: BoundedCurve,
    Self: ParametricSurface,
{
}

impl<C0, C1> ParameterDivision2D for HomotopySurface<C0, C1>
where
    C0: ParameterDivision1D,
    C1: ParameterDivision1D,
{
    fn parameter_division(
        &self,
        (urange, vrange): ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let (mut div, _) = self.curve0.parameter_division(urange, tol);
        let (div0, _) = self.curve1.parameter_division(urange, tol);
        div.extend(div0);
        div.sort_by(|x, y| x.partial_cmp(y).unwrap());
        div.dedup();
        (div, vec![vrange.0, vrange.1])
    }
}

impl<C0, C1> SearchNearestParameter<D2> for HomotopySurface<C0, C1>
where
    C0: BoundedCurve,
    C1: BoundedCurve<Point = C0::Point, Vector = C0::Vector>,
    C0::Point: EuclideanSpace<Scalar = f64, Diff = C0::Vector> + MetricSpace<Metric = f64>,
    C0::Vector: SsnpVector<Point = C0::Point>,
{
    type Point = C0::Point;
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

impl<C0, C1> SearchParameter<D2> for HomotopySurface<C0, C1>
where
    C0: BoundedCurve,
    C1: BoundedCurve<Point = C0::Point, Vector = C0::Vector>,
    C0::Point:
        EuclideanSpace<Scalar = f64, Diff = C0::Vector> + MetricSpace<Metric = f64> + Tolerance,
    C0::Vector: SspVector<Point = C0::Point>,
{
    type Point = C0::Point;
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

impl<P> From<HomotopySurface<BSplineCurve<P>, BSplineCurve<P>>> for BSplineSurface<P>
where P: ControlPoint<f64> + Tolerance
{
    fn from(value: HomotopySurface<BSplineCurve<P>, BSplineCurve<P>>) -> Self {
        let HomotopySurface {
            curve0: mut bspcurve0,
            curve1: mut bspcurve1,
        } = value;
        bspcurve0.syncro_degree(&mut bspcurve1);
        bspcurve0.syncro_knots(&mut bspcurve1);

        let uknot_vec = bspcurve0.knot_vec().clone();
        let vknot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
        let mut control_points = Vec::new();
        for i in 0..bspcurve0.control_points().len() {
            control_points.push(Vec::new());
            control_points[i].push(*bspcurve0.control_point(i));
            control_points[i].push(*bspcurve1.control_point(i));
        }
        BSplineSurface::new_unchecked((uknot_vec, vknot_vec), control_points)
    }
}

fn bound2opt<T>(x: Bound<T>) -> Option<T> {
    match x {
        Bound::Included(x) => Some(x),
        Bound::Excluded(x) => Some(x),
        Bound::Unbounded => None,
    }
}

fn range_common_part<R0, R1>(range0: &R0, range1: &R1) -> ParameterRange
where
    R0: RangeBounds<f64>,
    R1: RangeBounds<f64>, {
    use std::cmp::Ordering;
    let (t00, t01) = (range0.start_bound(), range0.end_bound());
    let (t10, t11) = (range1.start_bound(), range1.end_bound());
    let t0 = match (bound2opt(t00), bound2opt(t10)) {
        (Some(x), Some(y)) => match x.partial_cmp(y).unwrap() {
            Ordering::Greater => t00,
            Ordering::Less => t10,
            Ordering::Equal => match matches!(t00, Bound::Excluded(_)) {
                true => t00,
                false => t10,
            },
        },
        (_, None) => t00,
        (None, _) => t10,
    };
    let t1 = match (bound2opt(t01), bound2opt(t11)) {
        (Some(x), Some(y)) => match x.partial_cmp(y).unwrap() {
            Ordering::Less => t01,
            Ordering::Greater => t11,
            Ordering::Equal => match matches!(t01, Bound::Excluded(_)) {
                true => t01,
                false => t11,
            },
        },
        (_, None) => t01,
        (None, _) => t11,
    };
    (t0.cloned(), t1.cloned())
}

#[test]
fn test_range_common_part() {
    fn to_parameter_range<R: RangeBounds<f64>>(x: &R) -> ParameterRange {
        (x.start_bound().cloned(), x.end_bound().cloned())
    }
    fn compare<R0, R1, R2>(range0: R0, range1: R1, range2: R2)
    where
        R0: RangeBounds<f64>,
        R1: RangeBounds<f64>,
        R2: RangeBounds<f64>, {
        assert_eq!(
            range_common_part(&range0, &range1),
            to_parameter_range(&range2),
        );
        assert_eq!(
            range_common_part(&range1, &range0),
            to_parameter_range(&range2),
        );
    }
    compare(0.0..2.0, -1.0..1.0, 0.0..1.0);
    compare(0.0..=2.0, -1.0..2.0, 0.0..2.0);
    compare(..=2.0, 0.0.., 0.0..=2.0);
    compare(
        (Bound::Excluded(0.0), Bound::Included(1.0)),
        0.0..1.0,
        (Bound::Excluded(0.0), Bound::Excluded(1.0)),
    );
    compare(0.0..1.0, 2.0..3.0, 2.0..1.0)
}
