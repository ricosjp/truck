use super::*;
use derive_more::From;
use serde::{Deserialize, Serialize};
#[doc(hidden)]
pub use truck_geometry::{algo, inv_or_zero};
pub use truck_geometry::{decorators::*, nurbs::*, specifieds::*};
use truck_geotrait::{Invertible, ParametricSurface};
pub use truck_polymesh::PolylineCurve;

/// 3-dimensional curve
#[derive(Clone, Debug, Serialize, Deserialize, From)]
pub enum Curve {
    /// 3-dimensional B-spline curve
    BSplineCurve(BSplineCurve<Point3>),
    /// 3-dimensional NURBS curve
    NURBSCurve(NURBSCurve<Vector4>),
    /// intersection curve
    IntersectionCurve(IntersectionCurve<PolylineCurve<Point3>, Surface>),
}

macro_rules! derive_curve_method {
    ($curve: expr, $method: expr, $($ver: ident),*) => {
        match $curve {
            Curve::BSplineCurve(got) => $method(got, $($ver), *),
            Curve::NURBSCurve(got) => $method(got, $($ver), *),
            Curve::IntersectionCurve(got) => $method(got, $($ver), *),
        }
    };
}

macro_rules! derive_curve_self_method {
    ($curve: expr, $method: expr, $($ver: ident),*) => {
        match $curve {
            Curve::BSplineCurve(got) => Curve::BSplineCurve($method(got, $($ver), *)),
            Curve::NURBSCurve(got) => Curve::NURBSCurve($method(got, $($ver), *)),
            Curve::IntersectionCurve(got) => Curve::IntersectionCurve($method(got, $($ver), *)),
        }
    };
}

impl ParametricCurve for Curve {
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, t: f64) -> Self::Point { derive_curve_method!(self, ParametricCurve::subs, t) }
    fn der(&self, t: f64) -> Self::Vector { derive_curve_method!(self, ParametricCurve::der, t) }
    fn der2(&self, t: f64) -> Self::Vector { derive_curve_method!(self, ParametricCurve::der2, t) }
}

impl BoundedCurve for Curve {
    fn parameter_range(&self) -> (f64, f64) {
        derive_curve_method!(self, BoundedCurve::parameter_range,)
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
    fn transformed(&self, trans: Matrix4) -> Self {
        derive_curve_self_method!(self, Transformed::transformed, trans)
    }
}

impl ParameterDivision1D for Curve {
    type Point = Point3;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Point3>) {
        derive_curve_method!(self, ParameterDivision1D::parameter_division, range, tol)
    }
}

impl Cut for Curve {
    fn cut(&mut self, t: f64) -> Self { derive_curve_self_method!(self, Cut::cut, t) }
}

impl SearchNearestParameter for Curve {
    type Point = Point3;
    type Parameter = f64;
    fn search_nearest_parameter(
        &self,
        point: Point3,
        hint: Option<f64>,
        trials: usize,
    ) -> Option<f64> {
        derive_curve_method!(
            self,
            SearchNearestParameter::search_nearest_parameter,
            point,
            hint,
            trials
        )
    }
}

impl SearchParameter for Curve {
    type Point = Point3;
    type Parameter = f64;
    fn search_parameter(&self, point: Point3, hint: Option<f64>, trials: usize) -> Option<f64> {
        derive_curve_method!(self, SearchParameter::search_parameter, point, hint, trials)
    }
}

impl Curve {
    /// Into non-ratinalized 4-dimensinal B-spline curve
    pub fn lift_up(self) -> BSplineCurve<Vector4> {
        match self {
            Curve::BSplineCurve(curve) => BSplineCurve::new(
                curve.knot_vec().clone(),
                curve
                    .control_points()
                    .iter()
                    .map(|pt| pt.to_vec().extend(1.0))
                    .collect(),
            ),
            Curve::NURBSCurve(curve) => curve.into_non_rationalized(),
            Curve::IntersectionCurve(_) => {
                unimplemented!("intersection curve cannot connect by homotopy")
            }
        }
    }
}

/// 3-dimensional surfaces
#[derive(Clone, Debug, Serialize, Deserialize, From)]
pub enum Surface {
    /// Plane
    Plane(Plane),
    /// 3-dimensional B-spline surface
    BSplineSurface(BSplineSurface<Point3>),
    /// 3-dimensional NURBS Surface
    NURBSSurface(NURBSSurface<Vector4>),
    /// revoluted curve
    RevolutedCurve(Processor<RevolutedCurve<Curve>, Matrix4>),
}

macro_rules! derive_surface_method {
    ($surface: expr, $method: expr, $($ver: ident),*) => {
        match $surface {
            Self::Plane(got) => $method(got, $($ver), *),
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
    fn uuder(&self, u: f64, v: f64) -> Vector3 {
        derive_surface_method!(self, ParametricSurface::uuder, u, v)
    }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector3 {
        derive_surface_method!(self, ParametricSurface::uvder, u, v)
    }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector3 {
        derive_surface_method!(self, ParametricSurface::vvder, u, v)
    }
}

impl ParametricSurface3D for Surface {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        derive_surface_method!(self, ParametricSurface3D::normal, u, v)
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
    fn transformed(&self, trans: Matrix4) -> Self {
        derive_surface_self_method!(self, Transformed::transformed, trans)
    }
}

impl IncludeCurve<Curve> for Surface {
    #[inline(always)]
    fn include(&self, curve: &Curve) -> bool {
        match self {
            Surface::BSplineSurface(surface) => match curve {
                Curve::BSplineCurve(curve) => surface.include(curve),
                Curve::NURBSCurve(curve) => surface.include(curve),
                Curve::IntersectionCurve(_) => unimplemented!(),
            },
            Surface::NURBSSurface(surface) => match curve {
                Curve::BSplineCurve(curve) => surface.include(curve),
                Curve::NURBSCurve(curve) => surface.include(curve),
                Curve::IntersectionCurve(_) => unimplemented!(),
            },
            Surface::Plane(surface) => match curve {
                Curve::BSplineCurve(curve) => surface.include(curve),
                Curve::NURBSCurve(curve) => surface.include(curve),
                Curve::IntersectionCurve(_) => unimplemented!(),
            },
            Surface::RevolutedCurve(surface) => match surface.entity_curve() {
                Curve::BSplineCurve(entity_curve) => {
                    let surface = RevolutedCurve::by_revolution(
                        entity_curve,
                        surface.origin(),
                        surface.axis(),
                    );
                    match curve {
                        Curve::BSplineCurve(curve) => surface.include(curve),
                        Curve::NURBSCurve(curve) => surface.include(curve),
                        Curve::IntersectionCurve(_) => unimplemented!(),
                    }
                }
                Curve::NURBSCurve(entity_curve) => {
                    let surface = RevolutedCurve::by_revolution(
                        entity_curve,
                        surface.origin(),
                        surface.axis(),
                    );
                    match curve {
                        Curve::BSplineCurve(curve) => surface.include(curve),
                        Curve::NURBSCurve(curve) => surface.include(curve),
                        Curve::IntersectionCurve(_) => unimplemented!(),
                    }
                }
                Curve::IntersectionCurve(_) => unimplemented!(),
            },
        }
    }
}

impl SearchParameter for Surface {
    type Point = Point3;
    type Parameter = (f64, f64);
    fn search_parameter(
        &self,
        point: Point3,
        hint: Option<(f64, f64)>,
        trials: usize,
    ) -> Option<(f64, f64)> {
        derive_surface_method!(self, SearchParameter::search_parameter, point, hint, trials)
    }
}

impl SearchNearestParameter for Surface {
    type Point = Point3;
    type Parameter = (f64, f64);
    fn search_nearest_parameter(
        &self,
        point: Point3,
        hint: Option<(f64, f64)>,
        trials: usize,
    ) -> Option<(f64, f64)> {
        match self {
            Surface::Plane(plane) => plane.search_nearest_parameter(point, hint, trials),
            Surface::BSplineSurface(bspsurface) => {
                bspsurface.search_nearest_parameter(point, hint, trials)
            }
            Surface::NURBSSurface(surface) => surface.search_nearest_parameter(point, hint, trials),
            Surface::RevolutedCurve(rotted) => {
                let hint = match hint {
                    Some(hint) => hint,
                    None => algo::surface::presearch(rotted, point, rotted.parameter_range(), 100),
                };
                algo::surface::search_nearest_parameter(rotted, point, hint, trials)
            }
        }
    }
}

impl ParameterDivision2D for Surface {
    #[inline(always)]
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        derive_surface_method!(self, ParameterDivision2D::parameter_division, range, tol)
    }
}
