use truck_base::cgmath64::*;
use truck_meshalgo::prelude::*;

#[derive(Debug, Clone)]
pub struct IntersectionCurve<S> {
	surface0: S,
	surface1: S,
	polyline: PolylineCurve<Point3>,
	params0: PolylineCurve<Point2>,
	params1: PolylineCurve<Point2>,
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
	/// Returns the parameters of `self.surface0()`. We assume the following.
	/// # Assumption
	/// ```ignore
	/// // let i: usize = ...;
	/// let p = self.parameters0()[i];
	/// assert!(self.surface0().subs(p[0], p[1]), self.polyline()[i]);
	/// ```
	#[inline(always)]
	pub fn parameters0(&self) -> &PolylineCurve<Point2> { &self.params0 }
	/// Returns the parameters of `self.surface0()`. We assume the following.
	/// # Assumption
	/// ```ignore
	/// // let i: usize = ...;
	/// let p = self.parameters1()[i];
	/// assert!(self.surface1().subs(p[0], p[1]), self.polyline()[i]);
	/// ```
	#[inline(always)]
	pub fn parameters1(&self) -> &PolylineCurve<Point2> { &self.params1 }
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
		let mut params0 = PolylineCurve(Vec::new());
		let mut params1 = PolylineCurve(Vec::new());
		for p in poly.windows(2) {
			let n = (p[1] - p[0]).normalize();
			let (q, p0, p1) = double_projection(&surface0, None, &surface1, None, p[0], n, 100)?;
			polyline.push(q);
			params0.push(Point2::from(p0));
			params1.push(Point2::from(p1));
		}
		let (q, p0, p1) = if poly[0].near(&poly[poly.len() - 1]) {
			(polyline[0], params0[0], params1[0])
		} else {
			let n = (poly[poly.len() - 1] - poly[poly.len() - 2]).normalize();
			let (q, p0, p1) = double_projection(
				&surface0,
				None,
				&surface1,
				None,
				poly[poly.len() - 1],
				n,
				100,
			)?;
			(q, Point2::from(p0), Point2::from(p1))
		};
		polyline.push(q);
		params0.push(p0);
		params1.push(p1);
		Some(Self {
			surface0,
			surface1,
			polyline,
			params0,
			params1,
			tol,
		})
	}
}

impl<S> ParametricCurve for IntersectionCurve<S>
where S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>
{
	type Point = Point3;
	type Vector = Vector3;
	#[inline(always)]
	fn parameter_range(&self) -> (f64, f64) { (0.0, self.polyline.len() as f64) }
	fn subs(&self, t: f64) -> Point3 {
		if t < 0.0 {
			self.polyline[0]
		} else if t + 1.0 > self.polyline.len() as f64 {
			*self.polyline.last().unwrap()
		} else if t.floor().near(&t) {
			self.polyline[t as usize]
		} else {
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
	}
	fn der(&self, t: f64) -> Vector3 {
		if t < 0.0 || t + 1.0 > self.polyline.len() as f64 {
			Vector3::zero()
		} else if (t + TOLERANCE / 2.0).floor().near(&t) {
			let i = t as usize;
			let n = if i + 1 == self.polyline.len() {
				self.polyline[i] - self.polyline[i - 1]
			} else {
				self.polyline[i + 1] - self.polyline[i]
			};
			let i = (t + TOLERANCE) as usize;
			let d = self
				.surface0
				.normal(self.params0[i][0], self.params0[i][1])
				.cross(self.surface1.normal(self.params1[i][0], self.params1[i][1]))
				.normalize();
			d * (n.dot(n) / d.dot(n))
		} else {
			let i = t as usize;
			let n = if i + 1 == self.polyline.len() {
				self.polyline[i] - self.polyline[i - 1]
			} else {
				self.polyline[i + 1] - self.polyline[i]
			};
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
		let mut params0 = self.params0.cut(t);
		let mut params1 = self.params1.cut(t);
		if !(t + TOLERANCE / 2.0).floor().near(&t) {
			let (pt, p0, p1) =
				double_projection(&self.surface0, None, &self.surface1, None, pt, der, 100)
					.unwrap();
			*self.polyline.last_mut().unwrap() = pt;
			*polyline.first_mut().unwrap() = pt;
			*self.params0.last_mut().unwrap() = p0;
			*params0.first_mut().unwrap() = p0;
			*self.params1.last_mut().unwrap() = p1;
			*params1.first_mut().unwrap() = p1;
		}
		Self {
			surface0: self.surface0.clone(),
			surface1: self.surface1.clone(),
			polyline,
			params0,
			params1,
			tol: self.tol,
		}
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
mod double_projection_tests {
	use super::*;
	use truck_geometry::*;

	fn create_axis(n: Vector3) -> (Vector3, Vector3) {
		let idx = if n[0].abs() < n[1].abs() { 0 } else { 1 };
		let idx = if n[idx].abs() < n[2].abs() { idx } else { 2 };
		let mut e = Vector3::zero();
		e[idx] = 1.0;
		let x = n.cross(e).normalize();
		(x, n.cross(x))
	}

	fn exec_plane_case() {
		let c0 = Point3::new(
			2.0 * rand::random::<f64>() - 1.0,
			2.0 * rand::random::<f64>() - 1.0,
			2.0 * rand::random::<f64>() - 1.0,
		);
		let n0 = Vector3::new(
			2.0 * rand::random::<f64>() - 1.0,
			2.0 * rand::random::<f64>() - 1.0,
			2.0 * rand::random::<f64>() - 1.0,
		)
		.normalize();
		let (x, y) = create_axis(n0);
		let plane0 = Plane::new(c0, c0 + x, c0 + y);
		let c1 = Point3::new(
			2.0 * rand::random::<f64>() - 1.0,
			2.0 * rand::random::<f64>() - 1.0,
			2.0 * rand::random::<f64>() - 1.0,
		);
		let n1 = Vector3::new(
			2.0 * rand::random::<f64>() - 1.0,
			2.0 * rand::random::<f64>() - 1.0,
			2.0 * rand::random::<f64>() - 1.0,
		)
		.normalize();
		let (x, y) = create_axis(n1);
		let plane1 = Plane::new(c1, c1 + x, c1 + y);
		let n = n0.cross(n1).normalize();
		let mut o = None;
		for i in 0..10 {
			let t = i as f64;
			let p = Point3::origin() + t * n;
			let (q, p0, p1) = double_projection(&plane0, None, &plane1, None, p, n, 100)
				.unwrap_or_else(|| {
					panic!("plane0: {:?}\nplane1: {:?}\n p: {:?}", plane0, plane1, p)
				});
			assert_near!(q, plane0.subs(p0.x, p0.y));
			assert_near!(q, plane1.subs(p1.x, p1.y));
			if let Some(o) = o {
				assert_near!(q.distance2(o), t * t);
			} else {
				o = Some(q);
			}
		}
	}

	#[test]
	fn plane_case() { (0..100).for_each(|_| exec_plane_case()); }

	#[test]
	fn sphere_case() {
		let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 1.0), f64::sqrt(2.0));
		let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -1.0), f64::sqrt(2.0));
		for _ in 0..100 {
			let t = 2.0 * std::f64::consts::PI * rand::random::<f64>();
			let r = 2.0 * rand::random::<f64>();
			let p = Point3::new(r * f64::cos(t), r * f64::sin(t), 0.0);
			let n = Vector3::new(-f64::sin(t), f64::cos(t), 0.0);
			let (q, p0, p1) = double_projection(&sphere0, None, &sphere1, None, p, n, 100)
				.unwrap_or_else(|| panic!("p: {:?}", p));
			assert_near!(q, sphere0.subs(p0.x, p0.y));
			assert_near!(q, sphere1.subs(p1.x, p1.y));
			assert_near!(q, Point3::new(f64::cos(t), f64::sin(t), 0.0));
		}
	}
}

#[test]
fn intersection_curve_sphere_case() {
	use std::f64::consts::PI;
	use truck_geometry::*;
	let sphere0 = Sphere::new(Point3::new(0.0, 0.0, 1.0), f64::sqrt(2.0));
	let sphere1 = Sphere::new(Point3::new(0.0, 0.0, -1.0), f64::sqrt(2.0));
	const M: usize = 5;
	let polyline = (0..=M)
		.map(|i| {
			let t = 2.0 * PI * i as f64 / M as f64;
			Point3::new(0.8 * f64::cos(t), 0.8 * f64::sin(t), 0.0)
		})
		.collect::<PolylineCurve<_>>();
	let curve = IntersectionCurve::try_new(sphere0, sphere1, polyline, 0.5).unwrap();

	const N: usize = 100;
	let mut sum = 0.0;
	for i in 0..N {
		let t = 2.0 * PI * i as f64 / N as f64;
		let pt = curve.subs(t);
		assert_near!(pt.distance(Point3::origin()), 1.0);
		let vec = curve.der(t);
		assert!(pt.dot(vec).so_small());
		assert!(vec[2].so_small());
		sum += vec.magnitude() * 2.0 * PI / N as f64;
	}
	assert!(f64::abs(sum - 2.0 * PI) < 0.1, "{}", sum);

	let mut curve0 = curve.clone();
	let curve1 = curve0.cut(2.5);
	assert_near!(curve0.front(), curve.front());
	assert_near!(curve0.back(), curve.subs(2.5));
	assert_near!(curve1.front(), curve.subs(2.5));
	assert_near!(curve1.back(), curve.back());
	let mut curve0 = curve.clone();
	let curve1 = curve0.cut(2.0);
	assert_near!(curve0.front(), curve.front());
	assert_near!(curve0.back(), curve.subs(2.0));
	assert_near!(curve1.front(), curve.subs(2.0));
	assert_near!(curve1.back(), curve.back());
}

#[test]
fn collide_parabola() {
	use truck_geometry::*;
	const TOL: f64 = 0.05;

	// define surfaces
	#[cfg_attr(rustfmt, rustfmt_skip)]
	let ctrl0 = vec![
		vec![Point3::new(-1.0, -1.0, 3.0), Point3::new(-1.0, 0.0, -1.0), Point3::new(-1.0, 1.0, 3.0)],
		vec![Point3::new(0.0, -1.0, -1.0), Point3::new(0.0, 0.0, -5.0), Point3::new(0.0, 1.0, -1.0)],
		vec![Point3::new(1.0, -1.0, 3.0), Point3::new(1.0, 0.0, -1.0), Point3::new(1.0, 1.0, 3.0)],
	];
	#[cfg_attr(rustfmt, rustfmt_skip)]
	let ctrl1 = vec![
		vec![Point3::new(-1.0, -1.0, -3.0), Point3::new(-1.0, 0.0, 1.0), Point3::new(-1.0, 1.0, -3.0)],
		vec![Point3::new(0.0, -1.0, 1.0), Point3::new(0.0, 0.0, 5.0), Point3::new(0.0, 1.0, 1.0)],
		vec![Point3::new(1.0, -1.0, -3.0), Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 1.0, -3.0)],
	];
	let surface0 = BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl0);
	let surface1 = BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl1);

	// meshing surface
	let instant = std::time::Instant::now();
	let polygon0 =
		StructuredMesh::from_surface(&surface0, surface0.parameter_range(), TOL).destruct();
	let polygon1 =
		StructuredMesh::from_surface(&surface1, surface1.parameter_range(), TOL).destruct();
	println!("Meshing Surfaces: {}s", instant.elapsed().as_secs_f64());
	// extract intersection curves
	let instant = std::time::Instant::now();
	let curves = intersection_curves(surface0, &polygon0, surface1, &polygon1, TOL);
	println!(
		"Extracting Intersection: {}s",
		instant.elapsed().as_secs_f64()
	);
	assert_eq!(curves.len(), 1);
	let curve = curves[0].1.clone().unwrap();
	const N: usize = 100;
	for i in 0..N {
		let t1 = curve.parameter_range().1;
		let t = t1 * i as f64 / N as f64;
		let pt = curve.subs(t);
		assert_near!(pt.distance(Point3::origin()) * 0.5, f64::sqrt(0.5) * 0.5);
	}
}
