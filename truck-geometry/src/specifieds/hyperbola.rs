use super::*;

impl<P> UnitHyperbola<P> {
	/// constructor
	#[inline]
	pub const fn new() -> UnitHyperbola<P> { UnitHyperbola(std::marker::PhantomData) }
}

impl ParametricCurve for UnitHyperbola<Point2> {
	type Point = Point2;
	type Vector = Vector2;
	#[inline]
	fn subs(&self, t: f64) -> Self::Point { Point2::new(f64::cosh(t), f64::sinh(t)) }
	#[inline]
	fn der(&self, t: f64) -> Self::Vector { Vector2::new(f64::sinh(t), f64::cosh(t)) }
	#[inline]
	fn der2(&self, t: f64) -> Self::Vector { Vector2::new(f64::cosh(t), f64::sinh(t)) }
}

impl ParametricCurve for UnitHyperbola<Point3> {
	type Point = Point3;
	type Vector = Vector3;
	#[inline]
	fn subs(&self, t: f64) -> Self::Point { Point3::new(f64::cosh(t), f64::sinh(t), 0.0) }
	#[inline]
	fn der(&self, t: f64) -> Self::Vector { Vector3::new(f64::sinh(t), f64::cosh(t), 0.0) }
	#[inline]
	fn der2(&self, t: f64) -> Self::Vector { Vector3::new(f64::cosh(t), f64::sinh(t), 0.0) }
}

impl<P> ParameterDivision1D for UnitHyperbola<P>
where
	UnitHyperbola<P>: ParametricCurve<Point = P>,
	P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
{
	type Point = P;
	fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<P>) {
		algo::curve::parameter_division(self, range, tol)
	}
}
