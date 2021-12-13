use super::*;
use std::f64::consts::PI;

impl<P> UnitCircle<P> {
	/// constructor
	#[inline]
	pub const fn new() -> Self { Self(std::marker::PhantomData) }
}

impl ParametricCurve for UnitCircle<Point2> {
	type Point = Point2;
	type Vector = Vector2;
	#[inline]
	fn subs(&self, t: f64) -> Self::Point { Point2::new(f64::cos(t), f64::sin(t)) }
	#[inline]
	fn der(&self, t: f64) -> Self::Vector { Vector2::new(-f64::sin(t), f64::cos(t)) }
	#[inline]
	fn der2(&self, t: f64) -> Self::Vector { Vector2::new(-f64::cos(t), -f64::sin(t)) }
}

impl BoundedCurve for UnitCircle<Point2> {
	#[inline]
	fn parameter_range(&self) -> (f64, f64) { (0.0, 2.0 * PI) }
}

impl ParametricCurve for UnitCircle<Point3> {
	type Point = Point3;
	type Vector = Vector3;
	#[inline]
	fn subs(&self, t: f64) -> Self::Point { Point3::new(f64::cos(t), f64::sin(t), 0.0) }
	#[inline]
	fn der(&self, t: f64) -> Self::Vector { Vector3::new(-f64::sin(t), f64::cos(t), 0.0) }
	#[inline]
	fn der2(&self, t: f64) -> Self::Vector { Vector3::new(-f64::cos(t), -f64::sin(t), 0.0) }
}

impl BoundedCurve for UnitCircle<Point3> {
	#[inline]
	fn parameter_range(&self) -> (f64, f64) { (0.0, 2.0 * PI) }
}
