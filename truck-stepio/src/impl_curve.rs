#[macro_export]
macro_rules! impl_curve {
	($mod: tt) => {
		impl<P: From<$mod::CartesianPoint>> From<$mod::BSplineCurveWithKnots>
			for truck_geometry::BSplineCurve<P>
		{
			fn from(curve: $mod::BSplineCurveWithKnots) -> Self {
				let knots = curve.knots.into_iter().map(|a| a.into()).collect();
				let multi = curve
					.knot_multiplicities
					.into_iter()
					.map(|n| n as usize)
					.collect();
				let knots = truck_geometry::KnotVec::from_single_multi(knots, multi).unwrap();
				let ctrpts = curve
					.b_spline_curve
					.control_points_list
					.into_iter()
					.map(Into::into)
					.collect();
				Self::new(knots, ctrpts)
			}
		}
		impl<P: From<$mod::CartesianPoint>> From<$mod::BezierCurve>
			for truck_geometry::BSplineCurve<P>
		{
			fn from(curve: $mod::BezierCurve) -> Self {
				let curve = curve.b_spline_curve;
				let degree = curve.degree as usize;
				let knots = truck_geometry::KnotVec::bezier_knot(degree);
				let ctrpts = curve
					.control_points_list
					.into_iter()
					.map(Into::into)
					.collect();
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
		where V::Point: From<$mod::CartesianPoint>
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
