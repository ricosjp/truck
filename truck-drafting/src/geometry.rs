use super::*;
use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize};
pub use truck_geometry::{decorators::*, nurbs::*, specifieds::*};

/// 2-dimensional curves.
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    From,
    TryInto,
    ParametricCurve,
    BoundedCurve,
    ParameterDivision1D,
    Cut,
    Invertible,
    SearchNearestParameterD1,
    SearchParameterD1,
)]
pub enum Curve {
    /// line segment support geometry
    Line(Line<Point2>),
    /// 2-dimensional B-spline curve
    BSplineCurve(BSplineCurve<Point2>),
    /// 2-dimensional NURBS curve in homogeneous coordinates
    NurbsCurve(NurbsCurve<Vector3>),
    /// transformed circle arc
    CircleArc(Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3>),
}

macro_rules! derive_curve_method {
    ($curve: expr, $method: expr, $($arg: ident),*) => {
        match $curve {
            Curve::Line(got) => $method(got, $($arg), *),
            Curve::BSplineCurve(got) => $method(got, $($arg), *),
            Curve::NurbsCurve(got) => $method(got, $($arg), *),
            Curve::CircleArc(got) => $method(got, $($arg), *),
        }
    };
}

macro_rules! derive_curve_self_method {
    ($curve: expr, $method: expr, $($arg: ident),*) => {
        match $curve {
            Curve::Line(got) => Curve::Line($method(got, $($arg), *)),
            Curve::BSplineCurve(got) => Curve::BSplineCurve($method(got, $($arg), *)),
            Curve::NurbsCurve(got) => Curve::NurbsCurve($method(got, $($arg), *)),
            Curve::CircleArc(got) => Curve::CircleArc($method(got, $($arg), *)),
        }
    };
}

impl Transformed<Matrix3> for Curve {
    fn transform_by(&mut self, trans: Matrix3) {
        derive_curve_method!(self, Transformed::transform_by, trans);
    }

    fn transformed(&self, trans: Matrix3) -> Self {
        derive_curve_self_method!(self, Transformed::transformed, trans)
    }
}

impl ToSameGeometry<Curve> for Line<Point2> {
    #[inline]
    fn to_same_geometry(&self) -> Curve {
        Curve::from(*self)
    }
}

impl ToSameGeometry<Curve> for BSplineCurve<Point2> {
    #[inline]
    fn to_same_geometry(&self) -> Curve {
        Curve::from(self.clone())
    }
}

impl ToSameGeometry<Curve> for NurbsCurve<Vector3> {
    #[inline]
    fn to_same_geometry(&self) -> Curve {
        Curve::from(self.clone())
    }
}

impl ToSameGeometry<Curve> for Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3> {
    #[inline]
    fn to_same_geometry(&self) -> Curve {
        Curve::from(self.clone())
    }
}
