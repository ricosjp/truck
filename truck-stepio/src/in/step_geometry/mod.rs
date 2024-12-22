use crate::{self as truck_stepio};
use derive_more::*;
use serde::{Deserialize, Serialize};
use truck_derivers::{DisplayByStep, StepCurve, StepLength, StepSurface};

/// re-export structs in `truck-geometry` and `truck-polymesh`.
pub mod re_exports {
    pub use truck_geometry::prelude::*;
    pub use truck_polymesh::*;
}
pub use re_exports::*;

/// Errors that occur when converting STEP format
pub type StepConvertingError = Box<dyn std::error::Error>;

/// `ellipse`, realized in `truck`
pub type Ellipse<P, M> = Processor<TrimmedCurve<UnitCircle<P>>, M>;
/// `hyperbola`, realized in `truck`
pub type Hyperbola<P, M> = Processor<TrimmedCurve<UnitHyperbola<P>>, M>;
/// `parabola`, realized in `truck`
pub type Parabola<P, M> = Processor<TrimmedCurve<UnitParabola<P>>, M>;
/// `spherical_surface`, realized in `truck`
pub type SphericalSurface = Processor<Sphere, Matrix4>;
/// `cylindrical_surface`, realized in `truck`
pub type CylindricalSurface = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
/// `toroidal_surface`, realized in `truck`
pub type ToroidalSurface = Processor<Torus, Matrix4>;
/// `conical_surface`, realized in `truck`
pub type ConicalSurface = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
/// `surface_of_linear_extrusion`, realized in `truck`
pub type StepExtrudedCurve = ExtrudedCurve<Curve3D, Vector3>;
/// `surface_of_revolution`, realized in `truck`
pub type StepRevolutedCurve = Processor<RevolutedCurve<Curve3D>, Matrix4>;
/// `pcurve`, realized in `truck`
pub type PCurve = truck_geometry::prelude::PCurve<Box<Curve2D>, Box<Surface>>;

/// `conic` in 2D, realized in `truck`
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
    SelfSameGeometry,
    StepLength,
    DisplayByStep,
    StepCurve,
)]
pub enum Conic2D {
    Ellipse(Ellipse<Point2, Matrix3>),
    Hyperbola(Hyperbola<Point2, Matrix3>),
    Parabola(Parabola<Point2, Matrix3>),
}

/// `curve` in 2D, realized in `truck`
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
    SelfSameGeometry,
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

/// `conic` in 3D, realized in `truck`
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
    SelfSameGeometry,
    StepLength,
    DisplayByStep,
    StepCurve,
)]
pub enum Conic3D {
    Ellipse(Ellipse<Point3, Matrix4>),
    Hyperbola(Hyperbola<Point3, Matrix4>),
    Parabola(Parabola<Point3, Matrix4>),
}

/// `curve` in 3D, realized in `truck`
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
    SelfSameGeometry,
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

/// `elementary_surface`, realized in `truck`
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
    SelfSameGeometry,
    StepLength,
    DisplayByStep,
    StepSurface,
)]
pub enum ElementarySurface {
    Plane(Plane),
    Sphere(SphericalSurface),
    CylindricalSurface(CylindricalSurface),
    ToroidalSurface(ToroidalSurface),
    ConicalSurface(ConicalSurface),
}

/// `swept_surface`, realized in `truck`
#[derive(
    Clone,
    Debug,
    From,
    PartialEq,
    Serialize,
    Deserialize,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    SearchNearestParameterD2,
    Invertible,
    TransformedM4,
    SelfSameGeometry,
    StepLength,
    DisplayByStep,
    StepSurface,
)]
pub enum SweptCurve {
    ExtrudedCurve(StepExtrudedCurve),
    RevolutedCurve(StepRevolutedCurve),
}

/// `surface`, realized in `truck`
#[derive(
    Clone,
    Debug,
    From,
    PartialEq,
    Serialize,
    Deserialize,
    ParametricSurface3D,
    ParameterDivision2D,
    SearchParameterD2,
    SearchNearestParameterD2,
    Invertible,
    TransformedM4,
    SelfSameGeometry,
    StepLength,
    StepSurface,
)]
pub enum Surface {
    ElementarySurface(ElementarySurface),
    SweptCurve(SweptCurve),
    BSplineSurface(BSplineSurface<Point3>),
    NurbsSurface(NurbsSurface<Vector4>),
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

/// `spherical_surface`, realized in `truck`
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, StepSurface)]
pub struct Sphere(pub truck_geometry::prelude::Sphere);

impl truck_stepio::out::StepSurface for Processor<Sphere, Matrix4> {
    #[inline(always)]
    fn same_sense(&self) -> bool { self.orientation() }
}

mod sphere;

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

// implementation for trait `truck_modeling::builder`.
mod geom_impls;
