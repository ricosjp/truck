use super::*;
use std::f64::consts::PI;
use truck_geometry::*;

mod double_projection_tests {
	use super::*;

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

	let theta = 2.0 * PI * rand::random::<f64>();
	let pt = Point3::new(f64::cos(theta), f64::sin(theta), 0.0);
	let t = curve.search_parameter(pt, None, 1).unwrap();
	assert!(curve.subs(t).near(&pt));
	let pt = Point3::new(1.1 * f64::cos(theta), 1.1 * f64::sin(theta), 0.0);
	assert!(curve.search_parameter(pt, None, 1).is_none());

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
