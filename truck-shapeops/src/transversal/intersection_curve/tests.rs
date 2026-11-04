use super::*;
use std::f64::consts::PI;

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
    let curve: IntersectionCurve<_, _, _> =
        IntersectionCurveWithParameters::try_new(sphere0, sphere1, polyline)
            .unwrap()
            .into();

    const N: usize = 100;
    let mut sum = 0.0;
    let (t0, t1) = curve.range_tuple();
    for i in 0..=N {
        let t = t0 + (t1 - t0) * i as f64 / N as f64;
        let pt = curve.subs(t);
        assert_near!(pt.distance(Point3::origin()), 1.0);
        let vec = curve.der(t);
        assert!(pt.dot(vec).so_small(), "{i} {t} {vec:?}");
        assert!(vec[2].so_small());
        let denom = if matches!(i, 0 | N) { 2.0 } else { 1.0 };
        sum += vec.magnitude() / denom * (t1 - t0) / N as f64;
    }
    assert!(
        f64::abs(sum - 2.0 * PI) < 0.01,
        "res: {}\nans: {}",
        sum,
        2.0 * PI
    );

    let theta = 2.0 * PI * rand::random::<f64>();
    let pt = Point3::new(f64::cos(theta), f64::sin(theta), 0.0);
    let t = curve.search_parameter(pt, None, 10).unwrap();
    assert_near!(curve.subs(t), pt);
    let pt = Point3::new(1.1 * f64::cos(theta), 1.1 * f64::sin(theta), 0.0);
    assert!(curve.search_parameter(pt, None, 10).is_none());
    let t = curve.search_nearest_parameter(pt, None, 10).unwrap();
    assert_near!(curve.subs(t).distance(pt), 0.1);

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
    #[rustfmt::skip]
	let ctrl0 = vec![
		vec![Point3::new(-1.0, -1.0, 3.0), Point3::new(-1.0, 0.0, -1.0), Point3::new(-1.0, 1.0, 3.0)],
		vec![Point3::new(0.0, -1.0, -1.0), Point3::new(0.0, 0.0, -5.0), Point3::new(0.0, 1.0, -1.0)],
		vec![Point3::new(1.0, -1.0, 3.0), Point3::new(1.0, 0.0, -1.0), Point3::new(1.0, 1.0, 3.0)],
	];
    #[rustfmt::skip]
	let ctrl1 = vec![
		vec![Point3::new(-1.0, -1.0, -3.0), Point3::new(-1.0, 0.0, 1.0), Point3::new(-1.0, 1.0, -3.0)],
		vec![Point3::new(0.0, -1.0, 1.0), Point3::new(0.0, 0.0, 5.0), Point3::new(0.0, 1.0, 1.0)],
		vec![Point3::new(1.0, -1.0, -3.0), Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 1.0, -3.0)],
	];
    let surface0 = BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl0);
    let surface1 = BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl1);

    // meshing surface
    let instant = std::time::Instant::now();
    let polygon0 = StructuredMesh::from_surface(&surface0, surface0.range_tuple(), TOL).destruct();
    let polygon1 = StructuredMesh::from_surface(&surface1, surface1.range_tuple(), TOL).destruct();
    println!("Meshing Surfaces: {}s", instant.elapsed().as_secs_f64());
    // extract intersection curves
    let instant = std::time::Instant::now();
    let curves = intersection_curves(surface0, &polygon0, surface1, &polygon1).unwrap();
    println!(
        "Extracting Intersection: {}s",
        instant.elapsed().as_secs_f64()
    );
    assert_eq!(curves.len(), 1);
    let curve = curves[0].1.clone();
    const N: usize = 100;
    for i in 0..N {
        let t1 = curve.range_tuple().1;
        let t = t1 * i as f64 / N as f64;
        let pt = curve.subs(t);
        assert_near!(pt.distance(Point3::origin()) * 0.5, f64::sqrt(0.5) * 0.5);
    }
}

#[test]
fn gpu_ssi_seeds_newton_raphson_convergence() {
    #[rustfmt::skip]
    let ctrl0 = vec![
        vec![Point3::new(-1.0, -1.0, 3.0), Point3::new(-1.0, 0.0, -1.0), Point3::new(-1.0, 1.0, 3.0)],
        vec![Point3::new(0.0, -1.0, -1.0), Point3::new(0.0, 0.0, -5.0), Point3::new(0.0, 1.0, -1.0)],
        vec![Point3::new(1.0, -1.0, 3.0), Point3::new(1.0, 0.0, -1.0), Point3::new(1.0, 1.0, 3.0)],
    ];
    #[rustfmt::skip]
    let ctrl1 = vec![
        vec![Point3::new(-1.0, -1.0, -3.0), Point3::new(-1.0, 0.0, 1.0), Point3::new(-1.0, 1.0, -3.0)],
        vec![Point3::new(0.0, -1.0, 1.0), Point3::new(0.0, 0.0, 5.0), Point3::new(0.0, 1.0, 1.0)],
        vec![Point3::new(1.0, -1.0, -3.0), Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 1.0, -3.0)],
    ];
    let surface0 = BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl0);
    let surface1 = BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl1);

    let seeds = Dispatcher::global()
        .find_surface_intersection_seeds(&surface0, &surface1, 0.05);

    assert!(!seeds.is_empty(), "Seeds finder should find candidate intersection seeds");

    for &(uv0, uv1) in &seeds {
        // Newton-Raphson refinement
        let mut p0 = uv0;
        let mut p1 = uv1;
        let mut converged = false;
        for _ in 0..30 {
            let pt0 = surface0.subs(p0.x, p0.y);
            let pt1 = surface1.subs(p1.x, p1.y);
            if pt0.distance(pt1) < 1e-7 {
                converged = true;
                break;
            }
            if let Some(new_p0) = surface0.search_nearest_parameter(pt1, Some((p0.x, p0.y)), 10) {
                p0 = Point2::new(new_p0.0, new_p0.1);
            }
            let pt0_new = surface0.subs(p0.x, p0.y);
            if let Some(new_p1) = surface1.search_nearest_parameter(pt0_new, Some((p1.x, p1.y)), 10) {
                p1 = Point2::new(new_p1.0, new_p1.1);
            }
        }
        let final_pt0 = surface0.subs(p0.x, p0.y);
        let final_pt1 = surface1.subs(p1.x, p1.y);
        assert!(
            converged || final_pt0.distance(final_pt1) < 1e-4,
            "Seed {:?} -> {:?} should converge under Newton-Raphson nearest point search (final dist = {})",
            (uv0, uv1), (p0, p1), final_pt0.distance(final_pt1)
        );
    }
}

