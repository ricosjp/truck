use super::*;
use std::f64::consts::PI;

impl<P> UnitCircle<P> {
	/// constructor
	#[inline]
	pub const fn new() -> Self {
		Self(std::marker::PhantomData)
	}
}

impl ParametricCurve for UnitCircle<Point2> {
	type Point = Point2;
	type Vector = Vector2;
	#[inline]
	fn subs(&self, t: f64) -> Self::Point {
		Point2::new(f64::cos(t), f64::sin(t))
	}
	#[inline]
	fn der(&self, t: f64) -> Self::Vector {
		Vector2::new(-f64::sin(t), f64::cos(t))
	}
	#[inline]
	fn der2(&self, t: f64) -> Self::Vector {
		Vector2::new(-f64::cos(t), -f64::sin(t))
	}
}

impl BoundedCurve for UnitCircle<Point2> {
	#[inline]
	fn parameter_range(&self) -> (f64, f64) {
		(0.0, 2.0 * PI)
	}
}

impl ParametricCurve for UnitCircle<Point3> {
	type Point = Point3;
	type Vector = Vector3;
	#[inline]
	fn subs(&self, t: f64) -> Self::Point {
		Point3::new(f64::cos(t), f64::sin(t), 0.0)
	}
	#[inline]
	fn der(&self, t: f64) -> Self::Vector {
		Vector3::new(-f64::sin(t), f64::cos(t), 0.0)
	}
	#[inline]
	fn der2(&self, t: f64) -> Self::Vector {
		Vector3::new(-f64::cos(t), -f64::sin(t), 0.0)
	}
}

impl BoundedCurve for UnitCircle<Point3> {
	#[inline]
	fn parameter_range(&self) -> (f64, f64) {
		(0.0, 2.0 * PI)
	}
}

impl<P> ParameterDivision1D for UnitCircle<P>
where
	UnitCircle<P>: ParametricCurve<Point = P>,
{
	type Point = P;
	fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<P>) {
		nonpositive_tolerance!(tol);
		let tol = f64::min(tol, 0.8);
		let delta = 2.0 * f64::acos(1.0 - tol);
		let n = 1 + ((range.1 - range.0) / delta) as usize;
		let params = (0..=n)
			.map(|i| {
				let t = i as f64 / n as f64;
				range.0 * (1.0 - t) + range.1 * t
			})
			.collect::<Vec<_>>();
		let pts = params.iter().map(|t| self.subs(*t)).collect();
		(params, pts)
	}
}

impl SearchNearestParameter for UnitCircle<Point2> {
	type Point = Point2;
	type Parameter = f64;
	fn search_nearest_parameter(&self, pt: Point2, _: Option<f64>, _: usize) -> Option<f64> {
		let v = pt.to_vec();
		if v.magnitude2().so_small2() {
			return None;
		}
		let v = v.normalize();
		let theta = f64::acos(v.x);
		let theta = match v.y > 0.0 {
			true => theta,
			false => 2.0 * PI - theta,
		};
		Some(theta)
	}
}

impl SearchParameter for UnitCircle<Point2> {
	type Point = Point2;
	type Parameter = f64;
	fn search_parameter(&self, pt: Point2, _: Option<f64>, _: usize) -> Option<f64> {
		let v = pt.to_vec();
		if !v.magnitude2().near2(&1.0) {
			return None;
		}
		let v = v.normalize();
		let theta = f64::acos(v.x);
		let theta = match v.y > 0.0 {
			true => theta,
			false => 2.0 * PI - theta,
		};
		Some(theta)
	}
}

impl SearchNearestParameter for UnitCircle<Point3> {
	type Point = Point3;
	type Parameter = f64;
	fn search_nearest_parameter(&self, pt: Point3, _: Option<f64>, _: usize) -> Option<f64> {
		UnitCircle::<Point2>::new().search_nearest_parameter(Point2::new(pt.x, pt.y), None, 0)
	}
}

impl SearchParameter for UnitCircle<Point3> {
	type Point = Point3;
	type Parameter = f64;
	fn search_parameter(&self, pt: Point3, _: Option<f64>, _: usize) -> Option<f64> {
		if !f64::abs(pt.z).so_small() {
			return None;
		}
		UnitCircle::<Point2>::new().search_parameter(Point2::new(pt.x, pt.y), None, 0)
	}
}

#[test]
fn parameter_division() {
	let c = UnitCircle::<Point2>::new();
	let (_div, pts) = c.parameter_division(c.parameter_range(), 0.05);
	for a in pts.windows(2) {
		let p = a[0].midpoint(a[1]);
		assert!(p.to_vec().magnitude() > 0.95);
	}
}
