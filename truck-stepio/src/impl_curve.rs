#[macro_export]
macro_rules! impl_curve {
    ($mod: tt) => {
        impl<'a, P> From<&'a $mod::Line> for truck_geometry::BSplineCurve<P>
        where
            P: EuclideanSpace<Scalar = f64> + From<&'a $mod::CartesianPoint>,
            P::Diff: From<&'a $mod::Vector>,
        {
            fn from(line: &'a $mod::Line) -> Self {
                let knots = truck_geometry::KnotVec::bezier_knot(1);
                let ctrlpts = vec![
                    P::from(&line.pnt),
                    P::from(&line.pnt) + P::Diff::from(&line.dir),
                ];
                Self::new(knots, ctrlpts)
            }
        }
        impl<'a, P: From<&'a $mod::CartesianPoint>> From<&'a $mod::BSplineCurveWithKnots>
            for truck_geometry::BSplineCurve<P>
        {
            fn from(curve: &'a $mod::BSplineCurveWithKnots) -> Self {
                let knots = curve.knots.iter().map(|a| **a).collect();
                let multi = curve
                    .knot_multiplicities
                    .iter()
                    .map(|n| *n as usize)
                    .collect();
                let knots = truck_geometry::KnotVec::from_single_multi(knots, multi).unwrap();
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
            for truck_geometry::BSplineCurve<P>
        {
            fn from(curve: &'a $mod::BezierCurve) -> Self {
                let curve = &curve.b_spline_curve;
                let degree = curve.degree as usize;
                let knots = truck_geometry::KnotVec::bezier_knot(degree);
                let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
                Self::new(knots, ctrpts)
            }
        }
        impl<P: From<$mod::CartesianPoint>> From<$mod::QuasiUniformCurve>
            for truck_geometry::BSplineCurve<P>
        {
            fn from(curve: $mod::QuasiUniformCurve) -> Self {
                let curve = curve.b_spline_curve;
                let num_ctrl = curve.control_points_list.len();
                let degree = curve.degree as usize;
                let division = num_ctrl + 2 - degree;
                let mut knots = truck_geometry::KnotVec::uniform_knot(degree, division);
                knots.transform(division as f64, 0.0);
                let ctrpts = curve
                    .control_points_list
                    .into_iter()
                    .map(Into::into)
                    .collect();
                Self::new(knots, ctrpts)
            }
        }
        impl<V: Homogeneous<f64>> From<$mod::RationalBSplineCurve> for truck_geometry::NURBSCurve<V>
        where
            V::Point: From<$mod::CartesianPoint>,
        {
            fn from(curve: $mod::RationalBSplineCurve) -> Self {
                let bcurve = curve.b_spline_curve;
                let degree = bcurve.degree as usize;
                let knots = truck_geometry::KnotVec::bezier_knot(degree);
                let ctrpts = bcurve
                    .control_points_list
                    .into_iter()
                    .zip(curve.weights_data)
                    .map(|(pt, w)| V::from_point_weight(pt.into(), w))
                    .collect();
                Self::new(truck_geometry::BSplineCurve::new(knots, ctrpts))
            }
        }
        impl<P: From<$mod::CartesianPoint>> From<$mod::UniformCurve>
            for truck_geometry::BSplineCurve<P>
        {
            fn from(curve: $mod::UniformCurve) -> Self {
                use std::convert::TryInto;
                let curve = curve.b_spline_curve;
                let num_ctrl = curve.control_points_list.len();
                let degree = curve.degree as usize;
                let knots = (0..degree + num_ctrl + 1)
                    .map(|i| i as f64 - degree as f64)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                let ctrpts = curve
                    .control_points_list
                    .into_iter()
                    .map(Into::into)
                    .collect();
                Self::new(knots, ctrpts)
            }
        }
    };
}
