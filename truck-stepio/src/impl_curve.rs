#[macro_export]
macro_rules! impl_curve {
    ($mod: tt, $impl_curve_mod: ident) => {
        mod $impl_curve_mod {
            use super::$mod;
            use std::convert::TryFrom;
            use std::result::Result;
            use $crate::alias::*;
            $crate::sub_impl_curve!($mod);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sub_impl_curve {
    ($mod: tt) => {
        impl<'a, P> From<&'a $mod::Line> for Line<P>
        where
            P: EuclideanSpace<Scalar = f64> + From<&'a $mod::CartesianPoint>,
            P::Diff: From<&'a $mod::Vector>,
        {
            fn from(line: &'a $mod::Line) -> Self {
                let p = P::from(&line.pnt);
                let q = p + P::Diff::from(&line.dir);
                Self(p, q)
            }
        }
        impl TryFrom<&$mod::Circle> for Ellipse<Point2, Matrix3> {
            type Error = String;
            fn try_from(circle: &$mod::Circle) -> Result<Self, String> {
                let radius: f64 = **circle.radius;
                let transform =
                    Matrix3::try_from(&circle.conic.position)? * Matrix3::from_scale(radius);
                Ok(Processor::new(UnitCircle::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::Circle> for Ellipse<Point3, Matrix4> {
            type Error = String;
            fn try_from(circle: &$mod::Circle) -> Result<Self, String> {
                let radius: f64 = **circle.radius;
                let transform =
                    Matrix4::try_from(&circle.conic.position)? * Matrix4::from_scale(radius);
                Ok(Processor::new(UnitCircle::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::Ellipse> for Ellipse<Point2, Matrix3> {
            type Error = String;
            fn try_from(circle: &$mod::Ellipse) -> Result<Self, String> {
                let radius0: f64 = **circle.semi_axis_1;
                let radius1: f64 = **circle.semi_axis_2;
                let transform = Matrix3::try_from(&circle.conic.position)?
                    * Matrix3::from_nonuniform_scale(radius0, radius1);
                Ok(Processor::new(UnitCircle::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::Ellipse> for Ellipse<Point3, Matrix4> {
            type Error = String;
            fn try_from(circle: &$mod::Ellipse) -> Result<Self, String> {
                let radius0: f64 = **circle.semi_axis_1;
                let radius1: f64 = **circle.semi_axis_2;
                let transform = Matrix4::try_from(&circle.conic.position)?
                    * Matrix4::from_nonuniform_scale(radius0, radius1, 1.0);
                Ok(Processor::new(UnitCircle::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::Hyperbola> for Hyperbola<Point2, Matrix3> {
            type Error = String;
            fn try_from(circle: &$mod::Hyperbola) -> Result<Self, String> {
                let radius0: f64 = **circle.semi_axis;
                let radius1: f64 = **circle.semi_imag_axis;
                let transform = Matrix3::try_from(&circle.conic.position)?
                    * Matrix3::from_nonuniform_scale(radius0, radius1);
                Ok(Processor::new(UnitHyperbola::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::Hyperbola> for Hyperbola<Point3, Matrix4> {
            type Error = String;
            fn try_from(circle: &$mod::Hyperbola) -> Result<Self, String> {
                let radius0: f64 = **circle.semi_axis;
                let radius1: f64 = **circle.semi_imag_axis;
                let transform = Matrix4::try_from(&circle.conic.position)?
                    * Matrix4::from_nonuniform_scale(radius0, radius1, 1.0);
                Ok(Processor::new(UnitHyperbola::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::Parabola> for Parabola<Point2, Matrix3> {
            type Error = String;
            fn try_from(circle: &$mod::Parabola) -> Result<Self, String> {
                let f: f64 = *circle.focal_dist;
                let transform = Matrix3::try_from(&circle.conic.position)? * Matrix3::from_scale(f);
                Ok(Processor::new(UnitParabola::new()).transformed(transform))
            }
        }
        impl TryFrom<&$mod::Parabola> for Parabola<Point3, Matrix4> {
            type Error = String;
            fn try_from(circle: &$mod::Parabola) -> Result<Self, String> {
                let f: f64 = *circle.focal_dist;
                let transform = Matrix4::try_from(&circle.conic.position)? * Matrix4::from_scale(f);
                Ok(Processor::new(UnitParabola::new()).transformed(transform))
            }
        }
        impl<'a, P: From<&'a $mod::CartesianPoint>> From<&'a $mod::Polyline> for PolylineCurve<P> {
            fn from(poly: &'a $mod::Polyline) -> Self {
                Self(poly.points.iter().map(|pt| P::from(&pt)).collect())
            }
        }
        impl<'a, P: From<&'a $mod::CartesianPoint>> From<&'a $mod::BSplineCurveWithKnots>
            for BSplineCurve<P>
        {
            fn from(curve: &'a $mod::BSplineCurveWithKnots) -> Self {
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
        impl<'a, P: From<&'a $mod::CartesianPoint>> From<&'a $mod::BezierCurve>
            for BSplineCurve<P>
        {
            fn from(curve: &'a $mod::BezierCurve) -> Self {
                let curve = &curve.b_spline_curve;
                let degree = curve.degree as usize;
                let knots = KnotVec::bezier_knot(degree);
                let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
                Self::new(knots, ctrpts)
            }
        }
        impl<'a, P: From<&'a $mod::CartesianPoint>> From<&'a $mod::QuasiUniformCurve>
            for BSplineCurve<P>
        {
            fn from(curve: &'a $mod::QuasiUniformCurve) -> Self {
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
        impl<'a, V: Homogeneous<f64>> From<&'a $mod::RationalBSplineCurve> for NURBSCurve<V>
        where V::Point: From<&'a $mod::CartesianPoint>
        {
            fn from(curve: &'a $mod::RationalBSplineCurve) -> Self {
                let bcurve = &curve.b_spline_curve;
                let degree = bcurve.degree as usize;
                let knots = KnotVec::bezier_knot(degree);
                let ctrpts = bcurve
                    .control_points_list
                    .iter()
                    .zip(&curve.weights_data)
                    .map(|(pt, w)| V::from_point_weight(pt.into(), *w))
                    .collect();
                Self::new(BSplineCurve::new(knots, ctrpts))
            }
        }
        impl<'a, P: From<&'a $mod::CartesianPoint>> From<&'a $mod::UniformCurve>
            for BSplineCurve<P>
        {
            fn from(curve: &'a $mod::UniformCurve) -> Self {
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
    };
}
