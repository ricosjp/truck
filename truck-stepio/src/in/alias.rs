use derive_more::*;
use serde::{Deserialize, Serialize};
pub use truck_geometry::prelude::*;
pub use truck_polymesh::*;

pub type ExpressParseError = Box<dyn std::error::Error>;

pub trait Empty {
    fn empty() -> Self;
}

pub type Ellipse<P, M> = Processor<TrimmedCurve<UnitCircle<P>>, M>;
pub type Hyperbola<P, M> = Processor<TrimmedCurve<UnitHyperbola<P>>, M>;
pub type Parabola<P, M> = Processor<TrimmedCurve<UnitParabola<P>>, M>;
pub type RevolutedLine = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
pub type SphericalSurface = Processor<Sphere, Matrix4>;
pub type CylindricalSurface = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
pub type ToroidalSurface =
    Processor<RevolutedCurve<Processor<UnitCircle<Point3>, Matrix4>>, Matrix4>;
pub type ConicalSurface = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
pub type StepExtrudedCurve = ExtrudedCurve<Curve3D, Vector3>;
pub type StepRevolutedCurve = Processor<RevolutedCurve<Curve3D>, Matrix4>;
pub type PCurve = truck_geometry::prelude::PCurve<Box<Curve2D>, Box<Surface>>;

#[derive(
    Clone,
    Copy,
    Debug,
    From,
    Serialize,
    Deserialize,
    ParametricCurve,
    BoundedCurve,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
    SearchNearestParameterD1,
)]
pub enum Conic2D {
    Ellipse(Ellipse<Point2, Matrix3>),
    Hyperbola(Hyperbola<Point2, Matrix3>),
    Parabola(Parabola<Point2, Matrix3>),
}
#[derive(
    Clone,
    Debug,
    From,
    Serialize,
    Deserialize,
    ParametricCurve,
    BoundedCurve,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
    SearchNearestParameterD1,
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
    From,
    Debug,
    Serialize,
    Deserialize,
    ParametricCurve,
    BoundedCurve,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
    SearchNearestParameterD1,
)]
pub enum Conic3D {
    Ellipse(Ellipse<Point3, Matrix4>),
    Hyperbola(Hyperbola<Point3, Matrix4>),
    Parabola(Parabola<Point3, Matrix4>),
}

#[derive(
    Clone,
    Debug,
    From,
    Serialize,
    Deserialize,
    ParametricCurve,
    BoundedCurve,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
    SearchNearestParameterD1,
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
    Serialize,
    Deserialize,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    SearchNearestParameterD2,
    Invertible,
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
    Serialize,
    Deserialize,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    SearchNearestParameterD2,
    Invertible,
)]
pub enum SweptCurve {
    ExtrudedCurve(StepExtrudedCurve),
    RevolutedCurve(StepRevolutedCurve),
}
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    SearchNearestParameterD2,
    Invertible,
)]
pub enum Surface {
    ElementarySurface(Box<ElementarySurface>),
    SweptCurve(Box<SweptCurve>),
    BSplineSurface(Box<BSplineSurface<Point3>>),
    NurbsSurface(Box<NurbsSurface<Vector4>>),
}
