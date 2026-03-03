use monstertruck_geometry::prelude::*;
use monstertruck_solid::{FilletIntersectionCurve, ParameterCurveLinear};

use crate::{Curve, Surface};

impl TryFrom<Surface> for NurbsSurface<Vector4> {
    type Error = ();
    fn try_from(surface: Surface) -> Result<Self, ()> {
        match surface {
            Surface::Plane(plane) => Ok(NurbsSurface::from(BsplineSurface::from(plane))),
            Surface::BsplineSurface(bsp) => Ok(NurbsSurface::from(bsp)),
            Surface::NurbsSurface(ns) => Ok(ns),
            Surface::RevolutedCurve(_) | Surface::TSplineSurface(_) => Err(()),
        }
    }
}
// From<NurbsSurface<Vector4>> for Surface — provided by derive_more::From

impl TryFrom<Curve> for NurbsCurve<Vector4> {
    type Error = ();
    fn try_from(curve: Curve) -> Result<Self, ()> {
        match curve {
            Curve::Line(line) => Ok(NurbsCurve::from(BsplineCurve::from(line))),
            Curve::BsplineCurve(bsp) => Ok(NurbsCurve::from(bsp)),
            Curve::NurbsCurve(nc) => Ok(nc),
            Curve::IntersectionCurve(ic) => {
                let range = ic.range_tuple();
                Ok(sample_to_nurbs(range, |t| ic.subs(t), 16))
            }
        }
    }
}
// From<NurbsCurve<Vector4>> for Curve — provided by derive_more::From

impl From<ParameterCurveLinear> for Curve {
    fn from(c: ParameterCurveLinear) -> Self {
        let range = c.range_tuple();
        Curve::NurbsCurve(sample_to_nurbs(range, |t| c.subs(t), 16))
    }
}

impl From<FilletIntersectionCurve> for Curve {
    fn from(c: FilletIntersectionCurve) -> Self {
        let range = c.range_tuple();
        Curve::NurbsCurve(sample_to_nurbs(range, |t| c.subs(t), 16))
    }
}

/// Sample a parametric curve into a degree-1 NURBS polyline approximation.
fn sample_to_nurbs(
    range: (f64, f64),
    subs: impl Fn(f64) -> Point3,
    n: usize,
) -> NurbsCurve<Vector4> {
    let (t0, t1) = range;
    let pts: Vec<Point3> = (0..=n)
        .map(|i| subs(t0 + (t1 - t0) * (i as f64) / (n as f64)))
        .collect();
    let knots: Vec<f64> = (0..=n).map(|i| i as f64 / n as f64).collect();
    let knot_vec = KnotVector::from(
        std::iter::once(0.0)
            .chain(knots.iter().copied())
            .chain(std::iter::once(1.0))
            .collect::<Vec<_>>(),
    );
    let bsp = BsplineCurve::new(knot_vec, pts);
    NurbsCurve::from(bsp)
}
