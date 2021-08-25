use serde::{Deserialize, Serialize};
use truck_base::cgmath64::*;
use truck_meshalgo::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntersectionCurve<S> {
	surface0: S,
	surface1: S,
	polyline: PolylineCurve<Point3>,
	tol: f64,
}

pub fn double_projection<S>(
	surface0: &S,
	hint0: Option<(f64, f64)>,
	surface1: &S,
	hint1: Option<(f64, f64)>,
	point: Point3,
	normal: Vector3,
	trials: usize,
) -> Option<(Point3, Point2, Point2)>
where
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>,
{
	if trials == 0 {
		return None;
	}
	let (u0, v0) = surface0.search_nearest_parameter(point, hint0, 10)?;
	let pt0 = surface0.subs(u0, v0);
	let (u1, v1) = surface1.search_nearest_parameter(point, hint1, 10)?;
	let pt1 = surface1.subs(u1, v1);
	if point.near(&pt0) && point.near(&pt1) && pt0.near(&pt1) {
		Some((point, Point2::new(u0, v0), Point2::new(u1, v1)))
	} else {
		let n0 = surface0.normal(u0, v0);
		let n1 = surface1.normal(u1, v1);
		let mat = Matrix3::from_cols(n0, n1, normal).transpose();
		let inv = mat.invert()?;
		let pt = inv * Vector3::new(pt0.dot(n0), pt1.dot(n1), point.dot(normal));
		double_projection(
			surface0,
			Some((u0, v0)),
			surface1,
			Some((u1, v1)),
			Point3::from_vec(pt),
			normal,
			trials - 1,
		)
	}
}

impl<S> IntersectionCurve<S> {
	/// This curve is a part of intersection of `self.surface0()` and `self.surface1()`.
	#[inline(always)]
	pub fn surface0(&self) -> &S { &self.surface0 }
	/// This curve is a part of intersection of `self.surface0()` and `self.surface1()`.
	#[inline(always)]
	pub fn surface1(&self) -> &S { &self.surface1 }
	/// Returns the polyline leading this curve.
	#[inline(always)]
	pub fn polyline(&self) -> &PolylineCurve<Point3> { &self.polyline }
	/// Returns the polyline leading this curve.
	#[inline(always)]
	pub(super) fn polyline_mut(&mut self) -> &mut PolylineCurve<Point3> { &mut self.polyline }
	/// The tolerance for generating this intersection curve.
	#[inline(always)]
	pub fn tolerance(&self) -> f64 { self.tol }
}

impl<S> IntersectionCurve<S>
where S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	pub(super) fn try_new(
		surface0: S,
		surface1: S,
		poly: PolylineCurve<Point3>,
		tol: f64,
	) -> Option<Self> {
		let mut polyline = PolylineCurve(Vec::new());
		for p in poly.windows(2) {
			let n = (p[1] - p[0]).normalize();
			let q = double_projection(&surface0, None, &surface1, None, p[0], n, 100)?.0;
			polyline.push(q);
		}
		let q = if poly[0].near(&poly[poly.len() - 1]) {
			polyline[0]
		} else {
			let n = (poly[poly.len() - 1] - poly[poly.len() - 2]).normalize();
			double_projection(
				&surface0,
				None,
				&surface1,
				None,
				poly[poly.len() - 1],
				n,
				100,
			)?.0
		};
		polyline.push(q);
		Some(Self {
			surface0,
			surface1,
			polyline,
			tol,
		})
	}

	pub fn remeshing(&mut self) -> bool {
		let div = algo::curve::parameter_division(self, self.parameter_range(), self.tol);
		let mut polyline = PolylineCurve(Vec::new());
		for t in div {
			let pt = self.polyline().subs(t);
			let normal = self.polyline().der(t);
			let (pt, _, _) = match double_projection(
				self.surface0(),
				None,
				self.surface1(),
				None,
				pt,
				normal,
				100,
			) {
				Some(got) => got,
				None => return false,
			};
			polyline.push(pt);
		}
		self.polyline = polyline;
		true
	}
}

impl<S> ParametricCurve for IntersectionCurve<S>
where S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	type Point = Point3;
	type Vector = Vector3;
	#[inline(always)]
	fn parameter_range(&self) -> (f64, f64) { (0.0, self.polyline.len() as f64 - 1.0) }
	fn subs(&self, t: f64) -> Point3 {
		double_projection(
			&self.surface0,
			None,
			&self.surface1,
			None,
			self.polyline.subs(t),
			self.polyline.der(t),
			100,
		)
		.unwrap()
		.0
	}
	fn der(&self, t: f64) -> Vector3 {
		let n = self.polyline.der(t);
		let (_, p0, p1) = double_projection(
			&self.surface0,
			None,
			&self.surface1,
			None,
			self.polyline.subs(t),
			n.normalize(),
			100,
		)
		.unwrap();
		let d = self
			.surface0
			.normal(p0.x, p0.y)
			.cross(self.surface1.normal(p1.x, p1.y))
			.normalize();
		d * (n.dot(n) / d.dot(n))
	}
	/// This method is unimplemented! Should panic!!
	#[inline(always)]
	fn der2(&self, _: f64) -> Vector3 {
		unimplemented!();
	}
}

impl<S> ParameterDivision1D for IntersectionCurve<S>
where S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	#[inline(always)]
	fn parameter_division(&self, range: (f64, f64), tol: f64) -> Vec<f64> {
		if self.tol <= tol {
			(0..self.polyline.len() - 1).map(|i| i as f64).collect()
		} else {
			algo::curve::parameter_division(self, range, tol)
		}
	}
}

impl<S> Cut for IntersectionCurve<S>
where S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	#[inline(always)]
	fn cut(&mut self, t: f64) -> Self {
		let pt = self.polyline.subs(t);
		let der = self.polyline.der(t);
		let mut polyline = self.polyline.cut(t);
		if !(t + TOLERANCE / 2.0).floor().near(&t) {
			let (pt, _, _) =
				double_projection(&self.surface0, None, &self.surface1, None, pt, der, 100)
					.unwrap();
			*self.polyline.last_mut().unwrap() = pt;
			*polyline.first_mut().unwrap() = pt;
		}
		Self {
			surface0: self.surface0.clone(),
			surface1: self.surface1.clone(),
			polyline,
			tol: self.tol,
		}
	}
}

impl<S: Clone> Invertible for IntersectionCurve<S> {
	fn invert(&mut self) {
		self.polyline.invert();
	}
}

impl<S> SearchParameter for IntersectionCurve<S>
where S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	type Point = Point3;
	type Parameter = f64;
	fn search_parameter(&self, point: Point3, _: Option<f64>, _: usize) -> Option<f64> {
		let t = self
			.polyline
			.search_nearest_parameter(point, None, 1)
			.unwrap();
		let pt = self.subs(t);
		match pt.near(&point) {
			true => Some(t),
			false => None,
		}
	}
}

/// Only derive from leading polyline. Not precise.
impl<S> SearchNearestParameter for IntersectionCurve<S>
where S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	type Point = Point3;
	type Parameter = f64;
	fn search_nearest_parameter(&self, point: Point3, _: Option<f64>, _: usize) -> Option<f64> {
		self.polyline.search_nearest_parameter(point, None, 1)
	}
}
pub fn intersection_curves<S>(
	surface0: S,
	polygon0: &PolygonMesh,
	surface1: S,
	polygon1: &PolygonMesh,
	tol: f64,
) -> Vec<(PolylineCurve<Point3>, Option<IntersectionCurve<S>>)>
where
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>,
{
	let interferences = polygon0.extract_interference(polygon1);
	let polylines = crate::polyline_construction::construct_polylines(&interferences);
	polylines
		.into_iter()
		.map(|polyline| {
			let curve = IntersectionCurve::try_new(
				surface0.clone(),
				surface1.clone(),
				polyline.clone(),
				tol,
			);
			(polyline, curve)
		})
		.collect()
}

#[cfg(test)]
mod tests;
