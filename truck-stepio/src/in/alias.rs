use crate::{self as truck_stepio};
use derive_more::*;
use serde::{Deserialize, Serialize};
use truck_derivers::{DisplayByStep, StepCurve, StepLength, StepSurface};
pub use truck_geometry::prelude::*;
pub use truck_polymesh::*;

pub type ExpressParseError = Box<dyn std::error::Error>;

pub type Ellipse<P, M> = Processor<TrimmedCurve<UnitCircle<P>>, M>;
pub type Hyperbola<P, M> = Processor<TrimmedCurve<UnitHyperbola<P>>, M>;
pub type Parabola<P, M> = Processor<TrimmedCurve<UnitParabola<P>>, M>;
pub type RevolutedLine = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
pub type SphericalSurface = Processor<Sphere, Matrix4>;
pub type CylindricalSurface = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
pub type ToroidalSurface = Processor<Torus, Matrix4>;
pub type ConicalSurface = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
pub type StepExtrudedCurve = ExtrudedCurve<Curve3D, Vector3>;
pub type StepRevolutedCurve = Processor<RevolutedCurve<Curve3D>, Matrix4>;
pub type PCurve = truck_geometry::prelude::PCurve<Box<Curve2D>, Box<Surface>>;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    From,
    Serialize,
    Deserialize,
    ParametricCurve,
    BoundedCurve,
    Cut,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
    SearchNearestParameterD1,
    TransformedM3,
    StepLength,
    DisplayByStep,
    StepCurve,
)]
pub enum Conic2D {
    Ellipse(Ellipse<Point2, Matrix3>),
    Hyperbola(Hyperbola<Point2, Matrix3>),
    Parabola(Parabola<Point2, Matrix3>),
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    From,
    Serialize,
    Deserialize,
    ParametricCurve,
    BoundedCurve,
    Cut,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
    SearchNearestParameterD1,
    TransformedM3,
    StepLength,
    DisplayByStep,
    StepCurve,
)]

pub enum Curve2D {
    Line(Line<Point2>),
    Polyline(PolylineCurve<Point2>),
    Conic(Conic2D),
    BSplineCurve(BSplineCurve<Point2>),
    NurbsCurve(NurbsCurve<Vector3>),
}
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    From,
    Serialize,
    Deserialize,
    ParametricCurve,
    BoundedCurve,
    Cut,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
    SearchNearestParameterD1,
    TransformedM4,
    StepLength,
    DisplayByStep,
    StepCurve,
)]
pub enum Conic3D {
    Ellipse(Ellipse<Point3, Matrix4>),
    Hyperbola(Hyperbola<Point3, Matrix4>),
    Parabola(Parabola<Point3, Matrix4>),
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    From,
    Serialize,
    Deserialize,
    ParametricCurve,
    BoundedCurve,
    Cut,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
    SearchNearestParameterD1,
    TransformedM4,
    StepLength,
    DisplayByStep,
    StepCurve,
)]
pub enum Curve3D {
    Line(Line<Point3>),
    Polyline(PolylineCurve<Point3>),
    Conic(Conic3D),
    BSplineCurve(BSplineCurve<Point3>),
    PCurve(PCurve),
    NurbsCurve(NurbsCurve<Vector4>),
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    SearchNearestParameterD2,
    Invertible,
    TransformedM4,
    StepLength,
    DisplayByStep,
    StepSurface,
)]
pub enum ElementarySurface {
    Plane(Plane),
    RevolutedLine(RevolutedLine),
    Sphere(SphericalSurface),
    CylindricalSurface(CylindricalSurface),
    ToroidalSurface(ToroidalSurface),
    ConicalSurface(ConicalSurface),
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    SearchNearestParameterD2,
    Invertible,
    TransformedM4,
    StepLength,
    DisplayByStep,
    StepSurface,
)]
pub enum SweptCurve {
    ExtrudedCurve(StepExtrudedCurve),
    RevolutedCurve(StepRevolutedCurve),
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    SearchNearestParameterD2,
    Invertible,
    TransformedM4,
    StepLength,
    StepSurface,
)]
pub enum Surface {
    ElementarySurface(Box<ElementarySurface>),
    SweptCurve(Box<SweptCurve>),
    BSplineSurface(Box<BSplineSurface<Point3>>),
    NurbsSurface(Box<NurbsSurface<Vector4>>),
}

impl truck_stepio::out::DisplayByStep for Surface {
    fn fmt(&self, idx: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Surface::*;
        match self {
            ElementarySurface(x) => x.fmt(idx, f),
            SweptCurve(x) => x.fmt(idx, f),
            BSplineSurface(x) => x.fmt(idx, f),
            NurbsSurface(x) => x.fmt(idx, f),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, StepSurface)]
pub struct Sphere(pub truck_geometry::prelude::Sphere);

impl truck_stepio::out::StepSurface for Processor<Sphere, Matrix4> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.orientation() }
}

mod sphere {
    use super::*;
    use std::f64::consts::PI;
    use std::ops::Bound;
    impl ParametricSurface for Sphere {
        type Point = Point3;
        type Vector = Vector3;
        #[inline]
        fn subs(&self, u: f64, v: f64) -> Point3 { self.0.subs(PI / 2.0 - v, u) }
        #[inline]
        fn uder(&self, u: f64, v: f64) -> Vector3 { self.0.vder(PI / 2.0 - v, u) }
        #[inline]
        fn vder(&self, u: f64, v: f64) -> Vector3 { -self.0.uder(PI / 2.0 - v, u) }
        #[inline]
        fn uuder(&self, u: f64, v: f64) -> Vector3 { self.0.vvder(PI / 2.0 - v, u) }
        #[inline]
        fn uvder(&self, u: f64, v: f64) -> Vector3 { -self.0.uvder(PI / 2.0 - v, u) }
        #[inline]
        fn vvder(&self, u: f64, v: f64) -> Vector3 { self.0.uuder(PI / 2.0 - v, u) }
        #[inline]
        fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
            (
                (Bound::Included(0.0), Bound::Excluded(2.0 * PI)),
                (Bound::Included(-PI / 2.0), Bound::Excluded(PI / 2.0)),
            )
        }
        #[inline]
        fn u_period(&self) -> Option<f64> { Some(2.0 * PI) }
    }
    impl ParametricSurface3D for Sphere {
        #[inline]
        fn normal(&self, u: f64, v: f64) -> Vector3 { self.0.normal(PI / 2.0 - v, u) }
    }
    impl SearchNearestParameter<D2> for Sphere {
        type Point = Point3;
        #[inline]
        fn search_nearest_parameter<H: Into<SPHint2D>>(
            &self,
            point: Self::Point,
            hint: H,
            trials: usize,
        ) -> Option<(f64, f64)> {
            self.0
                .search_nearest_parameter(point, hint, trials)
                .map(|(u, v)| (v, PI / 2.0 - u))
        }
    }
    impl SearchParameter<D2> for Sphere {
        type Point = Point3;
        #[inline]
        fn search_parameter<H: Into<SPHint2D>>(
            &self,
            point: Self::Point,
            hint: H,
            trials: usize,
        ) -> Option<(f64, f64)> {
            self.0
                .search_parameter(point, hint, trials)
                .map(|(u, v)| (v, PI / 2.0 - u))
        }
    }
    impl ParameterDivision2D for Sphere {
        #[inline]
        fn parameter_division(
            &self,
            ((u0, u1), (v0, v1)): ((f64, f64), (f64, f64)),
            tol: f64,
        ) -> (Vec<f64>, Vec<f64>) {
            let range = ((PI / 2.0 - v1, PI / 2.0 - v0), (u0, u1));
            let (udiv0, vdiv0) = self.0.parameter_division(range, tol);
            let vdiv = udiv0.into_iter().map(|u| PI / 2.0 - u).collect();
            (vdiv0, vdiv)
        }
    }

    #[cfg(test)]
    proptest::proptest! {
        #[test]
        fn surface(
            center in proptest::array::uniform3(-100.0f64..100.0f64),
            radius in 0.1f64..100.0f64,
            (u, v) in (0.0..=2.0 * PI, -PI / 2.0..=PI / 2.0),
        ) {
            const EPS: f64 = 1.0e-3;
            let sphere = Sphere(truck_geometry::prelude::Sphere::new(center.into(), radius));

            let uder0 = sphere.uder(u, v);
            let uder1 = (sphere.subs(u + EPS, v) - sphere.subs(u - EPS, v)) / (2.0 * EPS);
            assert!(
                (uder0 - uder1).magnitude2() < EPS,
                "uder failed: {uder0:?}, {uder1:?}"
            );

            let vder0 = sphere.vder(u, v);
            let vder1 = (sphere.subs(u, v + EPS) - sphere.subs(u, v - EPS)) / (2.0 * EPS);
            assert!(
                (vder0 - vder1).magnitude2() < EPS,
                "vder failed: {vder0:?}, {vder1:?}"
            );

            let uuder0 = sphere.uuder(u, v);
            let uuder1 = (sphere.uder(u + EPS, v) - sphere.uder(u - EPS, v)) / (2.0 * EPS);
            assert!(
                (uuder0 - uuder1).magnitude2() < EPS,
                "uuder failed: {uuder0:?}, {uuder1:?}"
            );

            let uvder0 = sphere.uvder(u, v);
            let uvder1 = (sphere.uder(u, v + EPS) - sphere.uder(u, v - EPS)) / (2.0 * EPS);
            assert!(
                (uvder0 - uvder1).magnitude2() < EPS,
                "uvder failed: {uvder0:?}, {uvder1:?}"
            );

            let vvder0 = sphere.vvder(u, v);
            let vvder1 = (sphere.vder(u, v + EPS) - sphere.vder(u, v - EPS)) / (2.0 * EPS);
            assert!(
                (vvder0 - vvder1).magnitude2() < EPS,
                "vvder failed: {vvder0:?}, {vvder1:?}"
            );

            let n0 = sphere.normal(u, v);
            let n1 = sphere.uder(u, v).cross(sphere.vder(u, v)).normalize();
            assert!(
                (n0 - n1).magnitude2() < EPS,
                "normal failed: {n0:?}, {n1:?}"
            );
        }
    }
}

impl truck_stepio::out::ConstStepLength for Processor<Sphere, Matrix4> {
    const LENGTH: usize = Processor::<truck_geometry::prelude::Sphere, Matrix4>::LENGTH;
}
impl truck_stepio::out::StepLength for Processor<Sphere, Matrix4> {
    fn step_length(&self) -> usize { <Self as truck_stepio::out::ConstStepLength>::LENGTH }
}
impl truck_stepio::out::DisplayByStep for Processor<Sphere, Matrix4> {
    fn fmt(&self, idx: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Processor::new(self.entity().0)
            .transformed(*self.transform())
            .fmt(idx, f)
    }
}

/// Implementation required to apply a closed surface division to a shape parsed from a STEP file.
mod from_pcurve {
    use super::{Curve2D, Curve3D, Surface};
    use truck_geometry::prelude::*;

    impl From<PCurve<Line<Point2>, Surface>> for Curve3D {
        fn from(value: PCurve<Line<Point2>, Surface>) -> Self {
            let (line, surface) = value.decompose();
            Curve3D::PCurve(PCurve::new(Curve2D::Line(line).into(), surface.into()))
        }
    }
}
