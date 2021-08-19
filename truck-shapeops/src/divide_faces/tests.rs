use super::*;
use truck_geometry::*;
const TOL: f64 = 0.05;

crate::impl_from!(
	NURBSCurve<Vector4>,
	IntersectionCurve<BSplineSurface<Point3>>
);
type AlternativeIntersection =
	crate::test_util::Alternatives<NURBSCurve<Vector4>, IntersectionCurve<BSplineSurface<Point3>>>;

fn parabola_surfaces() -> (BSplineSurface<Point3>, BSplineSurface<Point3>) {
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
	(
		BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl0),
		BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl1),
	)
}

#[test]
fn independent_intersection() {
	let arc00: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(1.0, 0.0, 1.0, 1.0),
			Vector4::new(0.0, 1.0, 0.0, 0.0),
			Vector4::new(-1.0, 0.0, 1.0, 1.0),
		],
	))
	.into();
	let arc01 = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(-1.0, 0.0, 1.0, 1.0),
			Vector4::new(0.0, -1.0, 0.0, 0.0),
			Vector4::new(1.0, 0.0, 1.0, 1.0),
		],
	))
	.into();
	let arc10: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(1.0, 0.0, -1.0, 1.0),
			Vector4::new(0.0, 1.0, 0.0, 0.0),
			Vector4::new(-1.0, 0.0, -1.0, 1.0),
		],
	))
	.into();
	let arc11 = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(-1.0, 0.0, -1.0, 1.0),
			Vector4::new(0.0, -1.0, 0.0, 0.0),
			Vector4::new(1.0, 0.0, -1.0, 1.0),
		],
	))
	.into();

	let (surface0, surface1) = parabola_surfaces();

	let v00 = Vertex::new(Point3::new(1.0, 0.0, 1.0));
	let v01 = Vertex::new(Point3::new(-1.0, 0.0, 1.0));
	let v10 = Vertex::new(Point3::new(1.0, 0.0, -1.0));
	let v11 = Vertex::new(Point3::new(-1.0, 0.0, -1.0));
	let wire0: Wire<_, _> = vec![Edge::new(&v00, &v01, arc00), Edge::new(&v01, &v00, arc01)].into();
	let wire1: Wire<_, _> = vec![Edge::new(&v10, &v11, arc10), Edge::new(&v11, &v10, arc11)].into();
	let geom_shell0: Shell<_, _, _> = vec![Face::new(vec![wire0], surface0).inverse()].into();
	let geom_shell1: Shell<_, _, _> = vec![Face::new(vec![wire1], surface1)].into();
	let poly_shell0 = geom_shell0.triangulation(TOL).unwrap();
	let poly_shell1 = geom_shell1.triangulation(TOL).unwrap();
	let (geom_loops_store0, geom_loops_store1, poly_loops_store0, poly_loops_store1) =
		create_loops_stores(&geom_shell0, &poly_shell0, &geom_shell1, &poly_shell1, TOL).unwrap();
	assert_eq!(geom_loops_store0[0].len(), 3);
	assert_eq!(geom_loops_store0[0][0].len(), 2);
	assert_eq!(geom_loops_store0[0][1].len(), 2);
	assert_eq!(geom_loops_store0[0][2].len(), 2);
	assert_eq!(geom_loops_store1[0].len(), 3);
	assert_eq!(geom_loops_store1[0][0].len(), 2);
	assert_eq!(geom_loops_store1[0][1].len(), 2);
	assert_eq!(geom_loops_store1[0][2].len(), 2);
	assert_eq!(poly_loops_store0[0].len(), 3);
	assert_eq!(poly_loops_store0[0][0].len(), 2);
	assert_eq!(poly_loops_store0[0][1].len(), 2);
	assert_eq!(poly_loops_store0[0][2].len(), 2);
	assert_eq!(poly_loops_store1[0].len(), 3);
	assert_eq!(poly_loops_store1[0][0].len(), 2);
	assert_eq!(poly_loops_store1[0][1].len(), 2);
	assert_eq!(poly_loops_store1[0][2].len(), 2);
}
