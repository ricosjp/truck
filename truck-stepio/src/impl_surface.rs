#[macro_export]
macro_rules! impl_surface {
    ($mod: tt, $mod_impl_surface: ident) => {
        mod $mod_impl_surface {
            use super::$mod;
            use std::convert::{TryFrom, TryInto};
            use std::result::Result;
            use $crate::alias::*;
            $crate::sub_impl_surface!($mod);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sub_impl_surface {
    ($mod: tt) => {
        impl From<&$mod::Plane> for Plane {
            fn from(plane: &$mod::Plane) -> Self {
                let mat = Matrix4::from(&plane.elementary_surface.position);
                let o = Point3::from_homogeneous(mat[3]);
                let p = o + mat[0].truncate();
                let q = o + mat[1].truncate();
                Self::new(o, p, q)
            }
        }
        impl From<&$mod::CylindricalSurface> for RevolutedLine {
            fn from(cs: &$mod::CylindricalSurface) -> Self {
                let mat = Matrix4::from(&cs.elementary_surface.position);
                let radius = **cs.radius;
                let o = Point3::from_homogeneous(mat[3]);
                let (x, z) = (mat[0].truncate(), mat[2].truncate());
                let p = o + radius * x;
                let q = p + z;
                Processor::new(RevolutedCurve::by_revolution(Line(p, q), o, z)).inverse()
            }
        }
        impl From<&$mod::ConicalSurface> for RevolutedLine {
            fn from(cs: &$mod::ConicalSurface) -> Self {
                let mat = Matrix4::from(&cs.elementary_surface.position);
                let (radius, angle) = (*cs.radius, *cs.semi_angle);
                let o = Point3::from_homogeneous(mat[3]);
                let (x, z) = (mat[0].truncate(), mat[2].truncate());
                let p = o + radius * x;
                let q = p + f64::sin(angle) * x + f64::cos(angle) * z;
                Processor::new(RevolutedCurve::by_revolution(Line(p, q), o, z)).inverse()
            }
        }
        impl From<&$mod::SphericalSurface> for Processor<Sphere, Matrix3> {
            fn from(ss: &$mod::SphericalSurface) -> Self {
                let mat = Matrix4::from(&ss.elementary_surface.position);
                let center = Point3::from_homogeneous(mat[3]);
                let radius = **ss.radius;
                let mat =
                    Matrix3::from_cols(mat[0].truncate(), mat[1].truncate(), mat[2].truncate());
                Processor::new(Sphere::new(center, radius)).transformed(mat)
            }
        }
        impl From<&$mod::ToroidalSurface> for ToroidalSurface {
            fn from(ts: &$mod::ToroidalSurface) -> Self {
                let mat = Matrix4::from(&ts.elementary_surface.position);
                let r0 = **ts.major_radius;
                let r1 = **ts.minor_radius;
                let circle_mat =
                    Matrix4::from_translation(Vector3::new(r0, 0.0, 0.0)) * Matrix4::from_scale(r1);
                let circle = Processor::new(UnitCircle::new()).transformed(circle_mat);
                let torus =
                    RevolutedCurve::by_revolution(circle, Point3::origin(), Vector3::unit_y());
                Processor::new(torus).transformed(mat)
            }
        }
        impl TryFrom<&$mod::SurfaceOfLinearExtrusion> for StepExtrudedCurve {
            type Error = ExpressParseError;
            fn try_from(sle: &$mod::SurfaceOfLinearExtrusion) -> Result<Self, ExpressParseError> {
                Ok(ExtrudedCurve::by_extrusion(
                    Curve::try_from(&sle.swept_surface.swept_curve)?,
                    Vector3::from(&sle.extrusion_axis),
                ))
            }
        }
        impl TryFrom<&$mod::SurfaceOfRevolution> for StepRevolutedCurve {
            type Error = ExpressParseError;
            fn try_from(sr: &$mod::SurfaceOfRevolution) -> Result<Self, ExpressParseError> {
                Ok(Processor::new(RevolutedCurve::by_revolution(
                    Curve::try_from(&sr.swept_surface.swept_curve)?,
                    Point3::from(&sr.axis_position.placement.location),
                    match &sr.axis_position.axis {
                        Some(x) => Vector3::from(x),
                        None => Vector3::unit_z(),
                    },
                )))
            }
        }
        impl TryFrom<&$mod::SweptSurfaceAny> for SweptCurve {
            type Error = ExpressParseError;
            fn try_from(ss: &$mod::SweptSurfaceAny) -> Result<Self, ExpressParseError> {
                use $mod::SweptSurfaceAny::*;
                match ss {
                    SweptSurface(_) => Err("not enough data!".to_string()),
                    SurfaceOfLinearExtrusion(x) => Ok(Self::ExtrudedCurve((&**x).try_into()?)),
                    SurfaceOfRevolution(x) => Ok(Self::RevolutedCurve((&**x).try_into()?)),
                }
            }
        }
        impl From<&$mod::BSplineSurfaceWithKnots> for BSplineSurface<Point3> {
            fn from(surface: &$mod::BSplineSurfaceWithKnots) -> Self {
                let uknots = surface.u_knots.iter().map(|a| **a).collect();
                let umulti = surface
                    .u_multiplicities
                    .iter()
                    .map(|n| *n as usize)
                    .collect();
                let uknots = KnotVec::from_single_multi(uknots, umulti).unwrap();
                let vknots = surface.v_knots.iter().map(|a| **a).collect();
                let vmulti = surface
                    .v_multiplicities
                    .iter()
                    .map(|n| *n as usize)
                    .collect();
                let vknots = KnotVec::from_single_multi(vknots, vmulti).unwrap();
                let ctrls = surface
                    .b_spline_surface
                    .control_points_list
                    .iter()
                    .map(|vec| vec.iter().map(|pt| Point3::from(pt)).collect())
                    .collect();
                Self::new((uknots, vknots), ctrls)
            }
        }
        impl From<&$mod::UniformSurface> for BSplineSurface<Point3> {
            fn from(surface: &$mod::UniformSurface) -> Self {
                let surface = &surface.b_spline_surface;
                let unum_ctrl = surface.control_points_list.len();
                let udegree = surface.u_degree as usize;
                let uknots = KnotVec::try_from(
                    (0..udegree + unum_ctrl + 1)
                        .map(|i| i as f64 - udegree as f64)
                        .collect::<Vec<_>>(),
                );
                let vnum_ctrl = surface.control_points_list[0].len();
                let vdegree = surface.v_degree as usize;
                let vknots = KnotVec::try_from(
                    (0..vdegree + vnum_ctrl + 1)
                        .map(|i| i as f64 - vdegree as f64)
                        .collect::<Vec<_>>(),
                );
                let ctrls = surface
                    .control_points_list
                    .iter()
                    .map(|vec| vec.iter().map(|pt| Point3::from(pt)).collect())
                    .collect();
                Self::new((uknots.unwrap(), vknots.unwrap()), ctrls)
            }
        }
        impl From<&$mod::QuasiUniformSurface> for BSplineSurface<Point3> {
            fn from(surface: &$mod::QuasiUniformSurface) -> Self {
                let surface = &surface.b_spline_surface;
                let unum_ctrl = surface.control_points_list.len();
                let udegree = surface.u_degree as usize;
                let udivision = unum_ctrl + 2 - udegree;
                let mut uknots = KnotVec::uniform_knot(udegree, udivision);
                uknots.transform(udivision as f64, 0.0);
                let vnum_ctrl = surface.control_points_list[0].len();
                let vdegree = surface.v_degree as usize;
                let vdivision = vnum_ctrl + 2 - vdegree;
                let mut vknots = KnotVec::uniform_knot(vdegree, vdivision);
                vknots.transform(vdivision as f64, 0.0);
                let ctrls = surface
                    .control_points_list
                    .iter()
                    .map(|vec| vec.iter().map(|pt| Point3::from(pt)).collect())
                    .collect();
                Self::new((uknots, vknots), ctrls)
            }
        }
        impl From<&$mod::BezierSurface> for BSplineSurface<Point3> {
            fn from(surface: &$mod::BezierSurface) -> Self {
                let surface = &surface.b_spline_surface;
                let udegree = surface.u_degree as usize;
                let uknots = KnotVec::bezier_knot(udegree);
                let vdegree = surface.v_degree as usize;
                let vknots = KnotVec::bezier_knot(vdegree);
                let ctrls = surface
                    .control_points_list
                    .iter()
                    .map(|vec| vec.iter().map(|pt| Point3::from(pt)).collect())
                    .collect();
                Self::new((uknots, vknots), ctrls)
            }
        }
        impl From<&$mod::RationalBSplineSurface> for NURBSSurface<Vector4> {
            fn from(surface: &$mod::RationalBSplineSurface) -> Self {
                let udegree = surface.b_spline_surface.u_degree as usize;
                let uknots = KnotVec::bezier_knot(udegree);
                let vdegree = surface.b_spline_surface.v_degree as usize;
                let vknots = KnotVec::bezier_knot(vdegree);
                let ctrls = surface
                    .b_spline_surface
                    .control_points_list
                    .iter()
                    .zip(&surface.weights_data)
                    .map(|(vec0, vec1)| {
                        vec0.iter()
                            .zip(vec1)
                            .map(|(pt, w)| Vector4::from_point_weight(Point3::from(pt), *w))
                            .collect()
                    })
                    .collect();
                NURBSSurface::new(BSplineSurface::new((uknots, vknots), ctrls))
            }
        }
        impl TryFrom<&$mod::BSplineSurfaceAny> for NURBSSurface<Vector4> {
            type Error = ExpressParseError;
            fn try_from(curve: &$mod::BSplineSurfaceAny) -> Result<Self, ExpressParseError> {
                use $mod::BSplineSurfaceAny as BSSA;
                match curve {
                    BSSA::BSplineSurface(_) => Err("not enough data!".to_string()),
                    BSSA::BSplineSurfaceWithKnots(x) => Ok(NURBSSurface::new(
                        BSplineSurface::lift_up(BSplineSurface::from(&**x)),
                    )),
                    BSSA::UniformSurface(x) => Ok(NURBSSurface::new(BSplineSurface::lift_up(
                        BSplineSurface::from(&**x),
                    ))),
                    BSSA::QuasiUniformSurface(x) => Ok(NURBSSurface::new(BSplineSurface::lift_up(
                        BSplineSurface::from(&**x),
                    ))),
                    BSSA::BezierSurface(x) => Ok(NURBSSurface::new(BSplineSurface::lift_up(
                        BSplineSurface::from(&**x),
                    ))),
                    BSSA::RationalBSplineSurface(x) => Ok(NURBSSurface::from(&**x)),
                }
            }
        }
    };
}
