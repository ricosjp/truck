pub use truck_geometry::*;
pub use truck_polymesh::*;

pub type ExpressParseError = String;

pub trait Empty {
    fn empty() -> Self;
}

pub type Ellipse<P, M> = Processor<TrimmedCurve<UnitCircle<P>>, M>;
pub type Hyperbola<P, M> = Processor<TrimmedCurve<UnitHyperbola<P>>, M>;
pub type Parabola<P, M> = Processor<TrimmedCurve<UnitParabola<P>>, M>;
pub type RevolutedLine = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
pub type ToroidalSurface = Processor<RevolutedCurve<Ellipse<Point3, Matrix4>>, Matrix4>;
pub type StepExtrudedCurve = ExtrudedCurve<Curve3D, Vector3>;
pub type StepRevolutedCurve = Processor<RevolutedCurve<Curve3D>, Matrix4>;

#[derive(
    Clone,
    Copy,
    Debug,
    ParametricCurve,
    BoundedCurve,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
)]
pub enum Conic2D {
    Ellipse(Ellipse<Point2, Matrix3>),
    Hyperbola(Hyperbola<Point2, Matrix3>),
    Parabola(Parabola<Point2, Matrix3>),
}
#[derive(
    Clone, Debug, ParametricCurve, BoundedCurve, Invertible, ParameterDivision1D, SearchParameterD1,
)]
pub enum Curve2D {
    Line(Line<Point2>),
    Polyline(PolylineCurve<Point2>),
    Conic(Conic2D),
    BSplineCurve(BSplineCurve<Point2>),
    NURBSCurve(NURBSCurve<Vector3>),
}
#[derive(
    Clone,
    Copy,
    Debug,
    ParametricCurve,
    BoundedCurve,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
)]
pub enum Conic3D {
    Ellipse(Ellipse<Point3, Matrix4>),
    Hyperbola(Hyperbola<Point3, Matrix4>),
    Parabola(Parabola<Point3, Matrix4>),
}
#[derive(
    Clone, Debug, ParametricCurve, BoundedCurve, Invertible, ParameterDivision1D, SearchParameterD1,
)]
pub enum Curve3D {
    Line(Line<Point3>),
    Polyline(PolylineCurve<Point3>),
    Conic(Conic3D),
    BSplineCurve(BSplineCurve<Point3>),
    NURBSCurve(NURBSCurve<Vector4>),
}

#[derive(
    Clone,
    Copy,
    Debug,
    ParametricSurface,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    Invertible,
)]
pub enum ElementarySurface {
    Plane(Plane),
    RevolutedLine(RevolutedLine),
    Sphere(Processor<Sphere, Matrix3>),
    ToroidalSurface(ToroidalSurface),
}
#[derive(
    Clone,
    Debug,
    ParametricSurface,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    Invertible,
)]
pub enum SweptCurve {
    ExtrudedCurve(StepExtrudedCurve),
    RevolutedCurve(StepRevolutedCurve),
}
#[derive(
    Clone,
    Debug,
    ParametricSurface,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    Invertible,
)]
pub enum Surface {
    ElementarySurface(ElementarySurface),
    SweptCurve(SweptCurve),
    BSplineSurface(BSplineSurface<Point3>),
    NURBSSurface(NURBSSurface<Vector4>),
}
