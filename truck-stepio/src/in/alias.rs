use serde::{Deserialize, Serialize};
pub use truck_geometry::*;
pub use truck_polymesh::*;

pub type ExpressParseError = Box<dyn std::error::Error>;

pub trait Empty {
    fn empty() -> Self;
}

pub type Ellipse<P, M> = Processor<TrimmedCurve<UnitCircle<P>>, M>;
pub type Hyperbola<P, M> = Processor<TrimmedCurve<UnitHyperbola<P>>, M>;
pub type Parabola<P, M> = Processor<TrimmedCurve<UnitParabola<P>>, M>;
pub type RevolutedLine = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
pub type ToroidalSurface = RevolutedCurve<Ellipse<Point3, Matrix4>>;
pub type StepExtrudedCurve = ExtrudedCurve<Curve3D, Vector3>;
pub type StepRevolutedCurve = RevolutedCurve<Curve3D>;
pub type PCurve = truck_geometry::PCurve<Box<Curve2D>, Box<Surface>>;

#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
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
    Clone,
    Debug,
    Serialize,
    Deserialize,
    ParametricCurve,
    BoundedCurve,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
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
    Serialize,
    Deserialize,
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
    Clone,
    Debug,
    Serialize,
    Deserialize,
    ParametricCurve,
    BoundedCurve,
    Invertible,
    ParameterDivision1D,
    SearchParameterD1,
)]
pub enum Curve3D {
    Line(Line<Point3>),
    Polyline(PolylineCurve<Point3>),
    Conic(Conic3D),
    BSplineCurve(BSplineCurve<Point3>),
    PCurve(PCurve),
    NURBSCurve(NURBSCurve<Vector4>),
}

fn proj_mat(m: Matrix4) -> Matrix3 {
    Matrix3::new(
        m[0][0], m[0][1], m[0][3], m[1][0], m[1][1], m[1][3], m[3][0], m[3][1], m[3][3],
    )
}

macro_rules! impl_conic_projection {
    ($self: tt, $(($kind: tt, $curve: tt)),*) => {
        match $self { $(Conic3D::$kind(c) => {
            let mat = proj_mat(*c.transform());
            let c = c.entity();
            let range = c.parameter_range();
            Conic2D::$kind(
                Processor::new(TrimmedCurve::new($curve::<Point2>::new(), range)).transformed(mat),
            )
        })* }
    };
}

impl Conic3D {
    pub fn projection(self) -> Conic2D {
        impl_conic_projection!(
            self,
            (Ellipse, UnitCircle),
            (Hyperbola, UnitHyperbola),
            (Parabola, UnitParabola)
        )
    }
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
    Invertible,
)]
pub enum ElementarySurface {
    Plane(Plane),
    RevolutedLine(RevolutedLine),
    Sphere(Processor<Sphere, Matrix3>),
    CylindricalSurface(RevolutedCurve<Line<Point3>>),
    ToroidalSurface(ToroidalSurface),
}
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
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
    Serialize,
    Deserialize,
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
