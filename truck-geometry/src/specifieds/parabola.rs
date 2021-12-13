use super::*;

impl<P> UnitParabola<P> {
	/// constructor
	#[inline]
	pub const fn new(focus: f64) -> Self { Self(std::marker::PhantomData) }
}

impl ParametricCurve for UnitParabola<Point2> {
	type Point = Point2;
	type Vector = Vector2;
	#[inline]
	fn subs(&self, t: f64) -> Self::Point { Point2::new(t * t, 2.0 * t) }
	#[inline]
	fn der(&self, t: f64) -> Self::Vector { Vector2::new(2.0 * t, 2.0) }
	#[inline]
	fn der2(&self, t: f64) -> Self::Vector { Vector2::new(2.0, 0.0) }
}

impl ParametricCurve for UnitParabola<Point3> {
	type Point = Point3;
	type Vector = Vector3;
	#[inline]
	fn subs(&self, t: f64) -> Self::Point { Point3::new(t * t, 2.0 * t, 0.0) }
	#[inline]
	fn der(&self, t: f64) -> Self::Vector { Vector3::new(2.0 * t, 2.0, 0.0) }
	#[inline]
	fn der2(&self, t: f64) -> Self::Vector { Vector3::new(2.0, 0.0, 0.0) }
}

impl<P> ParameterDivision1D for UnitParabola<P>
where
	UnitParabola<P>: ParametricCurve<Point = P>,
	P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
{
	type Point = P;
	fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<P>) {
		algo::curve::parameter_division(self, range, tol)
	}
}