use super::*;
use truck_base::cgmath64::control_point::ControlPoint;

impl<P: Copy> Line<P> {
	/// initialize line from vector
	#[inline]
	pub fn from_origin_direction<V>(origin: P, direction: V) -> Self
	where P: std::ops::Add<V, Output = P> {
		Self(origin, origin + direction)
	}
	/// to a bspline curve
	#[inline]
	pub fn to_bspline(&self) -> BSplineCurve<P> {
		BSplineCurve::new(KnotVec::bezier_knot(1), vec![self.0, self.1])
	}
}

impl<P: ControlPoint<f64>> ParametricCurve for Line<P> {
	type Point = P;
	type Vector = P::Diff;
	#[inline]
	fn subs(&self, t: f64) -> Self::Point { self.0 + (self.1 - self.0) * t }
	#[inline]
	fn der(&self, _: f64) -> Self::Vector { self.1 - self.0 }
	#[inline]
	fn der2(&self, _: f64) -> Self::Vector { Self::Vector::zero() }
	#[inline]
	fn parameter_range(&self) -> (f64, f64) { (0.0, 1.0) }
}

impl<P: ControlPoint<f64>> Cut for Line<P> {
	#[inline]
	fn cut(&mut self, t: f64) -> Self {
		let r = self.subs(t);
		let res = Self(r, self.1);
		self.1 = r;
		res
	}
}

impl<P: ControlPoint<f64>> ParameterDivision1D for Line<P> {
	type Point = P;
	#[inline]
	fn parameter_division(&self, range: (f64, f64), _: f64) -> (Vec<f64>, Vec<P>) {
		(
			vec![range.0, range.1],
			vec![self.subs(range.0), self.subs(range.1)],
		)
	}
}

impl<P: Copy> Invertible for Line<P> {
	#[inline]
	fn invert(&mut self) {
		let r = self.0;
		self.0 = self.1;
		self.1 = r;
	}
	#[inline]
	fn inverse(&self) -> Self { Self(self.1, self.0) }
}

impl<P> SearchNearestParameter for Line<P>
where
	P: ControlPoint<f64>,
	P::Diff: InnerSpace<Scalar = f64>,
{
	type Point = P;
	type Parameter = f64;
	#[inline]
	fn search_nearest_parameter(&self, pt: P, _: Option<f64>, _: usize) -> Option<f64> {
		let b = self.1 - self.0;
		Some((pt - self.0).dot(b) / b.dot(b))
	}
}

impl<P> SearchParameter for Line<P>
where
	P: ControlPoint<f64> + Tolerance,
	P::Diff: InnerSpace<Scalar = f64>,
{
	type Point = P;
	type Parameter = f64;
	#[inline]
	fn search_parameter(&self, pt: P, _: Option<f64>, _: usize) -> Option<f64> {
		let b = self.1 - self.0;
		let t = (pt - self.0).dot(b) / b.dot(b);
		match self.subs(t).near(&pt) {
			true => Some(t),
			false => None,
		}
	}
}

#[test]
fn line() {
	let line = Line(Point2::new(1.0, 0.0), Point2::new(0.0, 1.0));

	// subs
	assert_near!(line.subs(0.4), Point2::new(0.6, 0.4));

	// inverse
	let line_inverse = line.inverse();
	assert_eq!(line.0, line_inverse.1);
	assert_eq!(line.1, line_inverse.0);

	// cut
	let mut line0 = line.clone();
	let line1 = line0.cut(0.4);
	assert_eq!(line.0, line0.0);
	assert_near!(line0.1, line.subs(0.4));
	assert_eq!(line0.1, line1.0);
	assert_eq!(line1.1, line.1);

	// SNP
	assert_near!(
		line.search_nearest_parameter(Point2::new(1.0, 1.0), None, 0)
			.unwrap(),
		0.5
	);
	assert!(line
		.search_parameter(Point2::new(1.0, 1.0), None, 0)
		.is_none());
}
