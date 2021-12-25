pub use truck_geometry::*;
pub use truck_polymesh::*;

pub type Ellipse<P, M> = Processor<UnitCircle<P>, M>;
pub type Hyperbola<P, M> = Processor<UnitHyperbola<P>, M>;
pub type Parabola<P, M> = Processor<UnitParabola<P>, M>;
pub type RevolutedLine = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;

pub trait CurveFromExpress<P>: ParametricCurve<Point = P> + ParameterDivision1D<Point = P> {}
impl<P, C: ParametricCurve<Point = P> + ParameterDivision1D<Point = P>> CurveFromExpress<P> for C {}
pub trait SurfaceFromExpress: ParametricSurface3D + ParameterDivision2D {}
impl<S: ParametricSurface3D + ParameterDivision2D> SurfaceFromExpress for S {}
