use truck_geometry::prelude::*;
use truck_shapeops::{FilletableCurve, FilletableSurface, ParamCurveLinear};

use crate::{Curve, Surface};

impl FilletableSurface for Surface {
    fn to_nurbs_surface(&self) -> Option<NurbsSurface<Vector4>> {
        match self {
            Surface::Plane(plane) => {
                let bsp: BSplineSurface<Point3> = (*plane).into();
                Some(NurbsSurface::from(bsp))
            }
            Surface::BSplineSurface(bsp) => Some(NurbsSurface::from(bsp.clone())),
            Surface::NurbsSurface(ns) => Some(ns.clone()),
            Surface::RevolutedCurve(_) | Surface::TSplineSurface(_) => None,
        }
    }
    fn from_nurbs_surface(surface: NurbsSurface<Vector4>) -> Self { Surface::NurbsSurface(surface) }
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
    let knot_vec = KnotVec::from(
        std::iter::once(0.0)
            .chain(knots.iter().copied())
            .chain(std::iter::once(1.0))
            .collect::<Vec<_>>(),
    );
    let bsp = BSplineCurve::new(knot_vec, pts);
    NurbsCurve::from(bsp)
}

impl FilletableCurve for Curve {
    fn to_nurbs_curve(&self) -> Option<NurbsCurve<Vector4>> {
        match self {
            Curve::Line(line) => {
                let bsp: BSplineCurve<Point3> = (*line).into();
                Some(NurbsCurve::from(bsp))
            }
            Curve::BSplineCurve(bsp) => Some(NurbsCurve::from(bsp.clone())),
            Curve::NurbsCurve(nc) => Some(nc.clone()),
            Curve::IntersectionCurve(_) => None,
        }
    }
    fn from_nurbs_curve(c: NurbsCurve<Vector4>) -> Self { Curve::NurbsCurve(c) }
    fn from_pcurve(c: ParamCurveLinear) -> Self {
        let range = c.range_tuple();
        Curve::NurbsCurve(sample_to_nurbs(range, |t| c.subs(t), 16))
    }
    fn from_intersection_curve(
        c: IntersectionCurve<
            ParamCurveLinear,
            Box<NurbsSurface<Vector4>>,
            Box<NurbsSurface<Vector4>>,
        >,
    ) -> Self {
        let range = c.range_tuple();
        Curve::NurbsCurve(sample_to_nurbs(range, |t| c.subs(t), 16))
    }
}
