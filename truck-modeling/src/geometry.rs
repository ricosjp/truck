use super::*;
use serde::{Serialize, Deserialize};
use truck_base::geom_traits::{Invertible, ParametricSurface};
pub use truck_geometry::{decorators::*, nurbs::*, specifieds::*};

/// 3-dimensional curve
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Curve {
    /// 3-dimensional B-spline curve
    BSplineCurve(BSplineCurve<Vector3>),
    /// 3-dimensional NURBS curve
    NURBSCurve(NURBSCurve<Vector4>),
}

macro_rules! derive_curve_method {
    ($curve: expr, $method: expr, $($ver: ident),*) => {
        match $curve {
            Curve::BSplineCurve(got) => $method(got, $($ver), *),
            Curve::NURBSCurve(got) => $method(got, $($ver), *),
        }
    };
}

macro_rules! derive_curve_self_method {
    ($curve: expr, $method: expr, $($ver: ident),*) => {
        match $curve {
            Curve::BSplineCurve(got) => Curve::BSplineCurve($method(got, $($ver), *)),
            Curve::NURBSCurve(got) => Curve::NURBSCurve($method(got, $($ver), *)),
        }
    };
}

impl ParametricCurve for Curve {
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, t: f64) -> Self::Point { derive_curve_method!(self, ParametricCurve::subs, t) }
    fn der(&self, t: f64) -> Self::Vector { derive_curve_method!(self, ParametricCurve::der, t) }
    fn der2(&self, t: f64) -> Self::Vector { derive_curve_method!(self, ParametricCurve::der2, t) }
    fn parameter_range(&self) -> (f64, f64) {
        derive_curve_method!(self, ParametricCurve::parameter_range,)
    }
}

impl Invertible for Curve {
    fn invert(&mut self) { derive_curve_method!(self, Invertible::invert,) }
    fn inverse(&self) -> Self { derive_curve_self_method!(self, Invertible::inverse,) }
}

impl Transformed<Matrix4> for Curve {
    fn transform_by(&mut self, trans: Matrix4) {
        derive_curve_method!(self, Transformed::transform_by, trans);
    }
    fn transformed(self, trans: Matrix4) -> Self {
        derive_curve_self_method!(self, Transformed::transformed, trans)
    }
}

impl Curve {
    #[inline(always)]
    pub(super) fn knot_vec(&self) -> &KnotVec {
        match self {
            Curve::BSplineCurve(curve) => curve.knot_vec(),
            Curve::NURBSCurve(curve) => curve.knot_vec(),
        }
    }
    #[inline(always)]
    pub(super) fn cut(&mut self, t: f64) -> Self {
        match self {
            Curve::BSplineCurve(curve) => Curve::BSplineCurve(curve.cut(t)),
            Curve::NURBSCurve(curve) => Curve::NURBSCurve(curve.cut(t)),
        }
    }
    /// Into non-ratinalized 4-dimensinal B-spline curve
    pub fn lift_up(self) -> BSplineCurve<Vector4> {
        match self {
            Curve::BSplineCurve(curve) => BSplineCurve::new(
                curve.knot_vec().clone(),
                curve
                    .control_points()
                    .iter()
                    .map(|pt| pt.extend(1.0))
                    .collect(),
            ),
            Curve::NURBSCurve(curve) => curve.into_non_rationalized(),
        }
    }
}

/// 3-dimensional surfaces
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Surface {
    /// Plane
    Plane(Plane),
    /// sphere
    Sphere(Processor<Sphere, Matrix4>),
    /// 3-dimensional B-spline surface
    BSplineSurface(BSplineSurface<Vector3>),
    /// 3-dimensional NURBS Surface
    NURBSSurface(NURBSSurface<Vector4>),
    /// revoluted curve
    RevolutedCurve(Processor<RevolutedCurve<Curve>, Matrix4>),
}

macro_rules! derive_surface_method {
    ($surface: expr, $method: expr, $($ver: ident),*) => {
        match $surface {
            Self::Plane(got) => $method(got, $($ver), *),
            Self::Sphere(got) => $method(got, $($ver), *),
            Self::BSplineSurface(got) => $method(got, $($ver), *),
            Self::NURBSSurface(got) => $method(got, $($ver), *),
            Self::RevolutedCurve(got) => $method(got, $($ver), *),
        }
    };
}

macro_rules! derive_surface_self_method {
    ($surface: expr, $method: expr, $($ver: ident),*) => {
        match $surface {
            Self::Plane(got) => Self::Plane($method(got, $($ver), *)),
            Self::Sphere(got) => Self::Sphere($method(got, $($ver), *)),
            Self::BSplineSurface(got) => Self::BSplineSurface($method(got, $($ver), *)),
            Self::NURBSSurface(got) => Self::NURBSSurface($method(got, $($ver), *)),
            Self::RevolutedCurve(got) => Self::RevolutedCurve($method(got, $($ver), *)),
        }
    };
}

impl ParametricSurface for Surface {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 {
        derive_surface_method!(self, ParametricSurface::subs, u, v)
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        derive_surface_method!(self, ParametricSurface::uder, u, v)
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        derive_surface_method!(self, ParametricSurface::vder, u, v)
    }
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        derive_surface_method!(self, ParametricSurface::normal, u, v)
    }
}

impl Invertible for Surface {
    fn invert(&mut self) { derive_surface_method!(self, Invertible::invert,) }
    fn inverse(&self) -> Self { derive_surface_self_method!(self, Invertible::inverse,) }
}

impl Transformed<Matrix4> for Surface {
    fn transform_by(&mut self, trans: Matrix4) {
        derive_surface_method!(self, Transformed::transform_by, trans);
    }
    fn transformed(self, trans: Matrix4) -> Self {
        derive_surface_self_method!(self, Transformed::transformed, trans)
    }
}
