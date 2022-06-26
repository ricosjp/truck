use super::*;
use derive_more::From;
use serde::{Deserialize, Serialize};
#[doc(hidden)]
pub use truck_geometry::{algo, inv_or_zero};
pub use truck_geometry::{decorators::*, nurbs::*, specifieds::*};
pub use truck_polymesh::PolylineCurve;

/// 3-dimensional curve
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    From,
    ParametricCurve,
    BoundedCurve,
    ParameterDivision1D,
    Cut,
    Invertible,
    SearchNearestParameterD1,
    SearchParameterD1,
)]
pub enum Curve {
    /// line
    Line(Line<Point3>),
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
            Curve::Line(got) => $method(got, $($ver), *),
            Curve::BSplineCurve(got) => $method(got, $($ver), *),
            Curve::NURBSCurve(got) => $method(got, $($ver), *),
            Curve::IntersectionCurve(got) => $method(got, $($ver), *),
        }
    };
}

macro_rules! derive_curve_self_method {
    ($curve: expr, $method: expr, $($ver: ident),*) => {
        match $curve {
            Curve::Line(got) => Curve::Line($method(got, $($ver), *)),
            Curve::BSplineCurve(got) => Curve::BSplineCurve($method(got, $($ver), *)),
            Curve::NURBSCurve(got) => Curve::NURBSCurve($method(got, $($ver), *)),
            Curve::IntersectionCurve(got) => Curve::IntersectionCurve($method(got, $($ver), *)),
        }
    };
}

impl Transformed<Matrix4> for Curve {
    fn transform_by(&mut self, trans: Matrix4) {
        derive_curve_method!(self, Transformed::transform_by, trans);
    }
    fn transformed(&self, trans: Matrix4) -> Self {
        derive_curve_self_method!(self, Transformed::transformed, trans)
    }
}

impl Curve {
    /// Into non-ratinalized 4-dimensinal B-spline curve
    pub fn lift_up(self) -> BSplineCurve<Vector4> {
        match self {
            Curve::Line(curve) => Curve::BSplineCurve(curve.to_bspline()).lift_up(),
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
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    From,
    ParametricSurface,
    ParameterDivision2D,
    Invertible,
    SearchParameterD2,
)]
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

impl ParametricSurface3D for Surface {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        derive_surface_method!(self, ParametricSurface3D::normal, u, v)
    }
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
                Curve::Line(curve) => surface.include(&curve.to_bspline()),
                Curve::BSplineCurve(curve) => surface.include(curve),
                Curve::NURBSCurve(curve) => surface.include(curve),
                Curve::IntersectionCurve(_) => unimplemented!(),
            },
            Surface::NURBSSurface(surface) => match curve {
                Curve::Line(curve) => surface.include(&curve.to_bspline()),
                Curve::BSplineCurve(curve) => surface.include(curve),
                Curve::NURBSCurve(curve) => surface.include(curve),
                Curve::IntersectionCurve(_) => unimplemented!(),
            },
            Surface::Plane(surface) => match curve {
                Curve::Line(curve) => surface.include(&curve.to_bspline()),
                Curve::BSplineCurve(curve) => surface.include(curve),
                Curve::NURBSCurve(curve) => surface.include(curve),
                Curve::IntersectionCurve(_) => unimplemented!(),
            },
            Surface::RevolutedCurve(surface) => match surface.entity_curve() {
                Curve::Line(curve) => self.include(&Curve::BSplineCurve(curve.to_bspline())),
                Curve::BSplineCurve(entity_curve) => {
                    let surface = RevolutedCurve::by_revolution(
                        entity_curve,
                        surface.origin(),
                        surface.axis(),
                    );
                    match curve {
                        Curve::Line(curve) => surface.include(&curve.to_bspline()),
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
                        Curve::Line(curve) => surface.include(&curve.to_bspline()),
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

impl SearchNearestParameter<D2> for Surface {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        match self {
            Surface::Plane(plane) => plane.search_nearest_parameter(point, hint, trials),
            Surface::BSplineSurface(bspsurface) => {
                bspsurface.search_nearest_parameter(point, hint, trials)
            }
            Surface::NURBSSurface(surface) => surface.search_nearest_parameter(point, hint, trials),
            Surface::RevolutedCurve(rotted) => {
                let hint = match hint.into() {
                    SPHint2D::Parameter(hint0, hint1) => (hint0, hint1),
                    SPHint2D::Range(x, y) => algo::surface::presearch(rotted, point, (x, y), 100),
                    SPHint2D::None => {
                        algo::surface::presearch(rotted, point, rotted.parameter_range(), 100)
                    }
                };
                algo::surface::search_nearest_parameter(rotted, point, hint, trials)
            }
        }
    }
}
