#[macro_export]
macro_rules! impl_curve {
    ($mod: tt, $impl_curve_mod: ident) => {
        mod $impl_curve_mod {
            use super::$mod;
            use std::convert::{TryFrom, TryInto};
            use std::result::Result;
            use $crate::alias::*;
            $crate::sub_impl_curve!($mod, Point2, Vector2, Matrix3, Vector3);
            $crate::sub_impl_curve!($mod, Point3, Vector3, Matrix4, Vector4);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sub_impl_curve {
    ($mod: tt, $point: ty, $vector: ty, $matrix: ty, $homogeneous: ty) => {
        impl From<&$mod::Line> for Line<$point> {
            fn from(line: &$mod::Line) -> Self {
                let p = <$point>::from(&line.pnt);
                let q = p + <$vector>::from(&line.dir);
                Self(p, q)
            }
        }
        impl TryFrom<&$mod::Circle> for Ellipse<$point, $matrix> {
            type Error = ExpressParseError;
            fn try_from(circle: &$mod::Circle) -> Result<Self, ExpressParseError> {
                let radius: f64 = **circle.radius;
                let transform =
                    <$matrix>::try_from(&circle.conic.position)? * <$matrix>::from_scale(radius);
                Ok(Processor::new(UnitCircle::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::Ellipse> for Ellipse<$point, $matrix> {
            type Error = ExpressParseError;
            fn try_from(circle: &$mod::Ellipse) -> Result<Self, ExpressParseError> {
                let radius0: f64 = **circle.semi_axis_1;
                let radius1: f64 = **circle.semi_axis_2;
                let transform = <$matrix>::try_from(&circle.conic.position)?
                    * <$matrix>::from(Matrix3::from_nonuniform_scale(radius0, radius1));
                Ok(Processor::new(UnitCircle::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::Hyperbola> for Hyperbola<$point, $matrix> {
            type Error = ExpressParseError;
            fn try_from(circle: &$mod::Hyperbola) -> Result<Self, ExpressParseError> {
                let radius0: f64 = **circle.semi_axis;
                let radius1: f64 = **circle.semi_imag_axis;
                let transform = <$matrix>::try_from(&circle.conic.position)?
                    * <$matrix>::from(Matrix3::from_nonuniform_scale(radius0, radius1));
                Ok(Processor::new(UnitHyperbola::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::Parabola> for Parabola<$point, $matrix> {
            type Error = ExpressParseError;
            fn try_from(circle: &$mod::Parabola) -> Result<Self, ExpressParseError> {
                let f: f64 = *circle.focal_dist;
                let transform =
                    <$matrix>::try_from(&circle.conic.position)? * <$matrix>::from_scale(f);
                Ok(Processor::new(UnitParabola::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::ConicAny> for Conic<$point, $matrix> {
            type Error = ExpressParseError;
            fn try_from(conic: &$mod::ConicAny) -> Result<Self, ExpressParseError> {
                use $mod::ConicAny::*;
                match conic {
                    Conic(_) => Err("not enough data!".to_string()),
                    Circle(c) => Ok(Self::Ellipse((&**c).try_into()?)),
                    Ellipse(c) => Ok(Self::Ellipse((&**c).try_into()?)),
                    Hyperbola(c) => Ok(Self::Hyperbola((&**c).try_into()?)),
                    Parabola(c) => Ok(Self::Parabola((&**c).try_into()?)),
                }
            }
        }
        impl From<&$mod::Polyline> for PolylineCurve<$point> {
            fn from(poly: &$mod::Polyline) -> Self {
                Self(poly.points.iter().map(|pt| <$point>::from(pt)).collect())
            }
        }
        impl From<&$mod::BSplineCurveWithKnots> for BSplineCurve<$point> {
            fn from(curve: &$mod::BSplineCurveWithKnots) -> Self {
                let knots = curve.knots.iter().map(|a| **a).collect();
                let multi = curve
                    .knot_multiplicities
                    .iter()
                    .map(|n| *n as usize)
                    .collect();
                let knots = KnotVec::from_single_multi(knots, multi).unwrap();
                let ctrpts = curve
                    .b_spline_curve
                    .control_points_list
                    .iter()
                    .map(Into::into)
                    .collect();
                Self::new(knots, ctrpts)
            }
        }
        impl From<&$mod::BezierCurve> for BSplineCurve<$point> {
            fn from(curve: &$mod::BezierCurve) -> Self {
                let curve = &curve.b_spline_curve;
                let degree = curve.degree as usize;
                let knots = KnotVec::bezier_knot(degree);
                let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
                Self::new(knots, ctrpts)
            }
        }
        impl From<&$mod::QuasiUniformCurve> for BSplineCurve<$point> {
            fn from(curve: &$mod::QuasiUniformCurve) -> Self {
                let curve = &curve.b_spline_curve;
                let num_ctrl = curve.control_points_list.len();
                let degree = curve.degree as usize;
                let division = num_ctrl + 2 - degree;
                let mut knots = KnotVec::uniform_knot(degree, division);
                knots.transform(division as f64, 0.0);
                let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
                Self::new(knots, ctrpts)
            }
        }
        impl From<&$mod::RationalBSplineCurve> for NURBSCurve<$homogeneous> {
            fn from(curve: &$mod::RationalBSplineCurve) -> Self {
                let bcurve = &curve.b_spline_curve;
                let degree = bcurve.degree as usize;
                let knots = KnotVec::bezier_knot(degree);
                let ctrpts = bcurve
                    .control_points_list
                    .iter()
                    .zip(&curve.weights_data)
                    .map(|(pt, w)| <$homogeneous>::from_point_weight(pt.into(), *w))
                    .collect();
                Self::new(BSplineCurve::new(knots, ctrpts))
            }
        }
        impl From<&$mod::UniformCurve> for BSplineCurve<$point> {
            fn from(curve: &$mod::UniformCurve) -> Self {
                let curve = &curve.b_spline_curve;
                let num_ctrl = curve.control_points_list.len();
                let degree = curve.degree as usize;
                let knots = KnotVec::try_from(
                    (0..degree + num_ctrl + 1)
                        .map(|i| i as f64 - degree as f64)
                        .collect::<Vec<_>>(),
                );
                let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
                Self::new(knots.unwrap(), ctrpts)
            }
        }
        impl TryFrom<&$mod::BSplineCurveAny> for NURBSCurve<$homogeneous> {
            type Error = ExpressParseError;
            fn try_from(curve: &$mod::BSplineCurveAny) -> Result<Self, ExpressParseError> {
                use $mod::BSplineCurveAny as BSCA;
                match curve {
                    BSCA::BSplineCurve(_) => Err("not enough data!".to_string()),
                    BSCA::BSplineCurveWithKnots(x) => Ok(NURBSCurve::new(BSplineCurve::lift_up(
                        BSplineCurve::<$point>::from(&**x),
                    ))),
                    BSCA::UniformCurve(x) => Ok(NURBSCurve::new(BSplineCurve::lift_up(
                        BSplineCurve::<$point>::from(&**x),
                    ))),
                    BSCA::QuasiUniformCurve(x) => Ok(NURBSCurve::new(BSplineCurve::lift_up(
                        BSplineCurve::<$point>::from(&**x),
                    ))),
                    BSCA::BezierCurve(x) => Ok(NURBSCurve::new(BSplineCurve::lift_up(
                        BSplineCurve::<$point>::from(&**x),
                    ))),
                    BSCA::RationalBSplineCurve(x) => Ok(NURBSCurve::from(&**x)),
                }
            }
        }
        impl TryFrom<&$mod::CurveAny> for Curve<$point, $homogeneous, $matrix> {
            type Error = ExpressParseError;
            fn try_from(curve: &$mod::CurveAny) -> Result<Self, ExpressParseError> {
                use $mod::CurveAny::*;
                match curve {
                    Curve(_) => Err("not enough data!".to_string()),
                    Line(x) => Ok(Self::Line((&**x).into())),
                    Conic(x) => Ok(Self::Conic((&**x).try_into()?)),
                    BoundedCurve(_x) => unimplemented!(),
                }
            }
        }
    };
}
