use truck_base::cgmath64::*;
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;

/// polyline base ntersection curve with parameter
#[derive(Debug, Clone, derive_more::Deref, derive_more::DerefMut)]
pub struct IntersectionCurveWithParameters<S0, S1> {
    #[deref]
    #[deref_mut]
    ic: IntersectionCurve<PolylineCurve<Point3>, S0, S1>,
    params0: PolylineCurve<Point2>,
    params1: PolylineCurve<Point2>,
}

impl<S0, S1> From<IntersectionCurveWithParameters<S0, S1>>
    for IntersectionCurve<PolylineCurve<Point3>, S0, S1>
{
    fn from(a: IntersectionCurveWithParameters<S0, S1>) -> Self { a.ic }
}

impl<S0, S1> IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    pub fn try_new(surface0: S0, surface1: S1, poly: PolylineCurve<Point3>) -> Option<Self> {
        let len = poly.len();
        let ic = IntersectionCurve::new(surface0, surface1, poly);
        let poly = ic.leader();
        let mut polyline = PolylineCurve(Vec::new());
        let mut params0 = PolylineCurve(Vec::new());
        let mut params1 = PolylineCurve(Vec::new());
        for i in 0..len - 1 {
            let (q, p0, p1) = ic.search_triple(i as f64, 100)?;
            polyline.push(q);
            params0.push(p0);
            params1.push(p1);
        }
        let (q, p0, p1) = match poly[0].near(&poly[poly.len() - 1]) {
            true => (polyline[0], params0[0], params1[0]),
            false => ic.search_triple((len - 1) as f64, 100)?,
        };
        polyline.push(q);
        params0.push(p0);
        params1.push(p1);
        Some(Self {
            ic,
            params0,
            params1,
        })
    }
}

impl<S0, S1> ParametricCurve for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, t: f64) -> Point3 { self.ic.subs(t) }
    #[inline(always)]
    fn der(&self, t: f64) -> Vector3 { self.ic.der(t) }
    #[inline(always)]
    fn der2(&self, t: f64) -> Vector3 { self.ic.der2(t) }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { self.ic.parameter_range() }
}

impl<S0, S1> BoundedCurve for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
}

impl<S0, S1> ParameterDivision1D for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    #[inline(always)]
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Point3>) {
        self.ic.parameter_division(range, tol)
    }
}

impl<S0, S1> Cut for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    #[inline(always)]
    fn cut(&mut self, t: f64) -> Self {
        Self {
            ic: self.ic.cut(t),
            params0: self.params0.cut(t),
            params1: self.params1.cut(t),
        }
    }
}

impl<S0: Clone, S1: Clone> Invertible for IntersectionCurveWithParameters<S0, S1> {
    fn invert(&mut self) {
        self.ic.invert();
        self.params0.invert();
        self.params1.invert();
    }
}

impl<S0, S1> SearchParameter<D1> for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        self.ic.search_parameter(point, hint, trials)
    }
}

impl<S0, S1> SearchNearestParameter<D1> for IntersectionCurveWithParameters<S0, S1>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        self.ic.search_nearest_parameter(point, hint, trials)
    }
}

pub fn intersection_curves<S0, S1>(
    surface0: S0,
    polygon0: &PolygonMesh,
    surface1: S1,
    polygon1: &PolygonMesh,
) -> Vec<(
    PolylineCurve<Point3>,
    Option<IntersectionCurveWithParameters<S0, S1>>,
)>
where
    S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    let interferences = polygon0.extract_interference(polygon1);
    let polylines = super::polyline_construction::construct_polylines(&interferences);
    polylines
        .into_iter()
        .map(|polyline| {
            let curve = IntersectionCurveWithParameters::try_new(
                surface0.clone(),
                surface1.clone(),
                polyline.clone(),
            );
            (polyline, curve)
        })
        .collect()
}

#[cfg(test)]
mod tests;
