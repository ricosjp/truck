use crate::{loops_store::*, *};
use truck_meshalgo::prelude::*;
use truck_topology::{Vertex, *};

fn loop_orientation<P, C, S>(face: &Face<P, C, S>, wire: &Wire<P, C>, tol: f64) -> Option<bool>
where
	C: ParametricCurve<Point = P> + ParameterDivision1D,
	S: ParametricSurface<Point = P> + SearchParameter<Point = P, Parameter = (f64, f64)>, {
	let surface = face.get_surface();
	let integral = wire.iter().try_fold(0.0, |integral, edge| {
		let curve = edge.get_curve();
		let poly = PolylineCurve::from_curve(&curve, curve.parameter_range(), tol)
			.0
			.into_iter()
			.map(|pt| surface.search_parameter(pt, None, 100))
			.collect::<Option<Vec<_>>>()?;
		let sign = if edge.orientation() { 1.0 } else { -1.0 };
		let tmp = poly.windows(2).fold(0.0, |counter, pt| {
			counter + (pt[1].0 + pt[0].0) * (pt[1].1 - pt[0].1)
		});
		Some(integral + sign * tmp)
	})?;
	Some(integral >= 0.0)
}
