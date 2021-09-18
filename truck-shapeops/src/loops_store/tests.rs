use super::*;
use std::fmt::Debug;
use truck_geometry::*;
const TOL: f64 = 0.01;

crate::impl_from!(
	NURBSCurve<Vector4>,
	IntersectionCurve<PolylineCurve, BSplineSurface<Point3>>
);
type AlternativeIntersection = crate::test_util::Alternatives<
	NURBSCurve<Vector4>,
	IntersectionCurve<PolylineCurve, BSplineSurface<Point3>>,
>;

struct DebugDisplay<'a, T, Format> {
	entity: &'a T,
	format: Format,
}

impl<'a, P: Debug, C: Debug> Debug for DebugDisplay<'a, Loops<P, C>, WireDisplayFormat> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_tuple("Loops")
			.field(
				&self
					.entity
					.0
					.iter()
					.map(|wire| wire.display(self.format))
					.collect::<Vec<_>>(),
			)
			.finish()
	}
}

impl<'a, P: Debug, C: Debug> Debug for DebugDisplay<'a, LoopsStore<P, C>, WireDisplayFormat> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_list()
			.entries(self.entity.0.iter().map(|loops| DebugDisplay {
				entity: loops,
				format: self.format,
			}))
			.finish()
	}
}

impl<P: Debug, C: Debug> LoopsStore<P, C> {
	fn display(&self, format: WireDisplayFormat) -> DebugDisplay<Self, WireDisplayFormat> {
		DebugDisplay {
			entity: self,
			format,
		}
	}
}

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
	// prepare geoetries
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

	// prepare topologies
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

	// exec create loops stores!
	let (geom_loops_store0, _poly_loops_store0, geom_loops_store1, _poly_loops_store1) =
		create_loops_stores(&geom_shell0, &poly_shell0, &geom_shell1, &poly_shell1, TOL).unwrap();

	// check the topology
	let vertex_format = VertexDisplayFormat::AsPoint;
	let edge_id_format = EdgeDisplayFormat::VerticesTupleAndID { vertex_format };
	let wire_id_format = WireDisplayFormat::EdgesListTuple {
		edge_format: edge_id_format,
	};
	let edge_geom_format = EdgeDisplayFormat::VerticesTupleAndCurve { vertex_format };
	let wire_geom_format = WireDisplayFormat::EdgesListTuple {
		edge_format: edge_geom_format,
	};
	assert_eq!(geom_loops_store0.len(), 1);
	assert_eq!(geom_loops_store0[0].len(), 3);
	assert_eq!(geom_loops_store0[0][0].len(), 2);
	assert_eq!(geom_loops_store0[0][1].len(), 2);
	assert_eq!(geom_loops_store0[0][2].len(), 2);
	assert!(
		geom_loops_store0[0][0].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[0][1].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[0][2].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[0][0].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	assert!(
		geom_loops_store0[0][1].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	assert!(
		geom_loops_store0[0][2].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	assert_eq!(geom_loops_store1.len(), 1);
	assert_eq!(geom_loops_store1[0].len(), 3);
	assert_eq!(geom_loops_store1[0][0].len(), 2);
	assert_eq!(geom_loops_store1[0][1].len(), 2);
	assert_eq!(geom_loops_store1[0][2].len(), 2);
	assert!(
		geom_loops_store1[0][0].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[0][1].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[0][2].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[0][0].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);
	assert!(
		geom_loops_store1[0][1].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);
	assert!(
		geom_loops_store1[0][2].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);

	// check the boundary status
	let mut flags = [true; 3];
	for i in 0..3 {
		let bw = &geom_loops_store0[0][i];
		match bw.status() {
			ShapesOpStatus::Unknown => {
				let curve = bw[0].oriented_curve();
				assert_near!(curve.subs(0.0)[2], 1.0);
				assert!(flags[0]);
				flags[0] = false;
			}
			ShapesOpStatus::And => {
				let curve = bw[0].oriented_curve();
				let pt = curve.subs(0.5) - Point3::origin();
				let der = curve.der(0.5);
				assert!(pt.cross(der)[2] > 0.0);
				assert!(flags[1]);
				flags[1] = false;
			}
			ShapesOpStatus::Or => {
				let curve = bw[0].oriented_curve();
				let pt = curve.subs(0.5) - Point3::origin();
				let der = curve.der(0.5);
				assert!(pt.cross(der)[2] < 0.0);
				assert!(flags[2]);
				flags[2] = false;
			}
		}
	}
	let mut flags = [true; 3];
	for i in 0..3 {
		let bw = &geom_loops_store1[0][i];
		match bw.status() {
			ShapesOpStatus::Unknown => {
				let curve = bw[0].oriented_curve();
				assert_near!(curve.subs(0.0)[2], -1.0);
				assert!(flags[0]);
				flags[0] = false;
			}
			ShapesOpStatus::Or => {
				let curve = bw[0].oriented_curve();
				let pt = curve.subs(0.5) - Point3::origin();
				let der = curve.der(0.5);
				assert!(pt.cross(der)[2] < 0.0);
				assert!(flags[1]);
				flags[1] = false;
			}
			ShapesOpStatus::And => {
				let curve = bw[0].oriented_curve();
				let pt = curve.subs(0.5) - Point3::origin();
				let der = curve.der(0.5);
				assert!(pt.cross(der)[2] > 0.0);
				assert!(flags[2]);
				flags[2] = false;
			}
		}
	}
}

#[test]
fn rotated_intersection() {
	// prepare geoetries
	let arc00: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(1.0, 0.0, 1.0, 1.0),
			Vector4::new(0.0, 1.0, 0.0, 0.0),
			Vector4::new(-1.0, 0.0, 1.0, 1.0),
		],
	))
	.into();
	let arc01: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(-1.0, 0.0, 1.0, 1.0),
			Vector4::new(0.0, -1.0, 0.0, 0.0),
			Vector4::new(1.0, 0.0, 1.0, 1.0),
		],
	))
	.into();
	let arc02: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(1.0, 0.0, 1.0, 1.0),
			Vector4::new(0.0, 0.0, -3.0, 1.0),
			Vector4::new(-1.0, 0.0, 1.0, 1.0),
		],
	))
	.into();
	let arc10: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(0.0, -1.0, -1.0, 1.0),
			Vector4::new(1.0, 0.0, 0.0, 0.0),
			Vector4::new(0.0, 1.0, -1.0, 1.0),
		],
	))
	.into();
	let arc11: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(0.0, 1.0, -1.0, 1.0),
			Vector4::new(-1.0, 0.0, 0.0, 0.0),
			Vector4::new(0.0, -1.0, -1.0, 1.0),
		],
	))
	.into();
	let arc12: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(0.0, -1.0, -1.0, 1.0),
			Vector4::new(0.0, 0.0, 3.0, 1.0),
			Vector4::new(0.0, 1.0, -1.0, 1.0),
		],
	))
	.into();
	let (surface0, surface1) = parabola_surfaces();

	let v00 = Vertex::new(Point3::new(1.0, 0.0, 1.0));
	let v01 = Vertex::new(Point3::new(-1.0, 0.0, 1.0));
	let edge00 = Edge::new(&v00, &v01, arc00);
	let edge01 = Edge::new(&v01, &v00, arc01);
	let edge02 = Edge::new(&v00, &v01, arc02);
	let wire00: Wire<_, _> = vec![edge00, edge02.inverse()].into();
	let wire01: Wire<_, _> = vec![edge01, edge02].into();
	let face00 = Face::new(vec![wire00], surface0.clone());
	let face01 = Face::new(vec![wire01], surface0.clone());
	let geom_shell0: Shell<_, _, _> = vec![face00.inverse(), face01.inverse()].into();

	let v10 = Vertex::new(Point3::new(0.0, -1.0, -1.0));
	let v11 = Vertex::new(Point3::new(0.0, 1.0, -1.0));
	let edge10 = Edge::new(&v10, &v11, arc10);
	let edge11 = Edge::new(&v11, &v10, arc11);
	let edge12 = Edge::new(&v10, &v11, arc12);
	let wire10: Wire<_, _> = vec![edge10, edge12.inverse()].into();
	let wire11: Wire<_, _> = vec![edge12, edge11].into();
	let face10 = Face::new(vec![wire10], surface1.clone());
	let face11 = Face::new(vec![wire11], surface1.clone());
	let geom_shell1: Shell<_, _, _> = vec![face10, face11].into();

	let poly_shell0 = geom_shell0.triangulation(TOL).unwrap();
	let poly_shell1 = geom_shell1.triangulation(TOL).unwrap();
	let file = std::fs::File::create("polyshell0.obj").unwrap();
	obj::write(&poly_shell0.into_polygon(), file).unwrap();
	let file = std::fs::File::create("polyshell1.obj").unwrap();
	obj::write(&poly_shell1.into_polygon(), file).unwrap();
	let (geom_loops_store0, _poly_loops_store0, geom_loops_store1, _poly_loops_store1) =
		create_loops_stores(&geom_shell0, &poly_shell0, &geom_shell1, &poly_shell1, TOL).unwrap();

	let vertex_format = VertexDisplayFormat::AsPoint;
	let edge_id_format = EdgeDisplayFormat::VerticesTupleAndID { vertex_format };
	let wire_id_format = WireDisplayFormat::EdgesListTuple {
		edge_format: edge_id_format,
	};
	let edge_geom_format = EdgeDisplayFormat::VerticesTupleAndCurve { vertex_format };
	let wire_geom_format = WireDisplayFormat::EdgesListTuple {
		edge_format: edge_geom_format,
	};
	assert_eq!(geom_loops_store0.len(), 2);
	assert_eq!(geom_loops_store0[0].len(), 2);
	assert!(
		geom_loops_store0[0][0].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[0][1].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[0][0].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	assert!(
		geom_loops_store0[0][1].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	let (a, b) = (geom_loops_store0[0][0].len(), geom_loops_store0[0][1].len());
	let compatible0 = match geom_loops_store0[0][0].status() {
		ShapesOpStatus::And => a == 3,
		ShapesOpStatus::Or => a == 5,
		_ => false,
	};
	let compatible1 = match geom_loops_store0[0][1].status() {
		ShapesOpStatus::And => b == 3,
		ShapesOpStatus::Or => b == 5,
		_ => false,
	};
	assert!(
		compatible0 && compatible1 && a * b == 15,
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert_eq!(geom_loops_store0[1].len(), 2);
	assert!(
		geom_loops_store0[1][0].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[1][1].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[1][0].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	assert!(
		geom_loops_store0[1][1].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	let (a, b) = (geom_loops_store0[1][0].len(), geom_loops_store0[1][1].len());
	let compatible0 = match geom_loops_store0[1][0].status() {
		ShapesOpStatus::And => a == 3,
		ShapesOpStatus::Or => a == 5,
		_ => false,
	};
	let compatible1 = match geom_loops_store0[1][1].status() {
		ShapesOpStatus::And => b == 3,
		ShapesOpStatus::Or => b == 5,
		_ => false,
	};
	assert!(
		compatible0 && compatible1 && a * b == 15,
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert_eq!(geom_loops_store1.len(), 2);
	assert_eq!(geom_loops_store1[0].len(), 2);
	assert!(
		geom_loops_store1[0][0].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[0][1].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[0][0].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);
	assert!(
		geom_loops_store1[0][1].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);
	let (a, b) = (geom_loops_store1[0][0].len(), geom_loops_store1[0][1].len());
	let compatible0 = match geom_loops_store1[0][0].status() {
		ShapesOpStatus::And => a == 3,
		ShapesOpStatus::Or => a == 5,
		_ => false,
	};
	let compatible1 = match geom_loops_store1[0][1].status() {
		ShapesOpStatus::And => b == 3,
		ShapesOpStatus::Or => b == 5,
		_ => false,
	};
	assert!(
		compatible0 && compatible1 && a * b == 15,
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert_eq!(geom_loops_store1[1].len(), 2);
	assert!(
		geom_loops_store1[1][0].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[1][1].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[1][0].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);
	assert!(
		geom_loops_store1[1][1].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);
	let (a, b) = (geom_loops_store1[1][0].len(), geom_loops_store1[1][1].len());
	let compatible0 = match geom_loops_store1[1][0].status() {
		ShapesOpStatus::And => a == 3,
		ShapesOpStatus::Or => a == 5,
		_ => false,
	};
	let compatible1 = match geom_loops_store1[1][1].status() {
		ShapesOpStatus::And => b == 3,
		ShapesOpStatus::Or => b == 5,
		_ => false,
	};
	assert!(
		compatible0 && compatible1 && a * b == 15,
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
}

#[test]
fn crossing_edges() {
	let arc00: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(1.0, 0.0, 1.0, 1.0),
			Vector4::new(0.0, 1.0, 0.0, 0.0),
			Vector4::new(-1.0, 0.0, 1.0, 1.0),
		],
	))
	.into();
	let arc01: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(-1.0, 0.0, 1.0, 1.0),
			Vector4::new(0.0, -1.0, 0.0, 0.0),
			Vector4::new(1.0, 0.0, 1.0, 1.0),
		],
	))
	.into();
	let arc02: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(1.0, 0.0, 1.0, 1.0),
			Vector4::new(0.0, 0.0, -3.0, 1.0),
			Vector4::new(-1.0, 0.0, 1.0, 1.0),
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
	let arc11: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(-1.0, 0.0, -1.0, 1.0),
			Vector4::new(0.0, -1.0, 0.0, 0.0),
			Vector4::new(1.0, 0.0, -1.0, 1.0),
		],
	))
	.into();
	let arc12: AlternativeIntersection = NURBSCurve::new(BSplineCurve::new(
		KnotVec::bezier_knot(2),
		vec![
			Vector4::new(1.0, 0.0, -1.0, 1.0),
			Vector4::new(0.0, 0.0, 3.0, 1.0),
			Vector4::new(-1.0, 0.0, -1.0, 1.0),
		],
	))
	.into();
	let (surface0, surface1) = parabola_surfaces();

	let v00 = Vertex::new(Point3::new(1.0, 0.0, 1.0));
	let v01 = Vertex::new(Point3::new(-1.0, 0.0, 1.0));
	let edge00 = Edge::new(&v00, &v01, arc00);
	let edge01 = Edge::new(&v01, &v00, arc01);
	let edge02 = Edge::new(&v00, &v01, arc02);
	let wire00: Wire<_, _> = vec![edge00, edge02.inverse()].into();
	let wire01: Wire<_, _> = vec![edge01, edge02].into();
	let face00 = Face::new(vec![wire00], surface0.clone());
	let face01 = Face::new(vec![wire01], surface0.clone());
	let geom_shell0: Shell<_, _, _> = vec![face00.inverse(), face01.inverse()].into();

	let v10 = Vertex::new(Point3::new(1.0, 0.0, -1.0));
	let v11 = Vertex::new(Point3::new(-1.0, 0.0, -1.0));
	let edge10 = Edge::new(&v10, &v11, arc10);
	let edge11 = Edge::new(&v11, &v10, arc11);
	let edge12 = Edge::new(&v10, &v11, arc12);
	let wire10: Wire<_, _> = vec![edge10, edge12.inverse()].into();
	let wire11: Wire<_, _> = vec![edge11, edge12].into();
	let face10 = Face::new(vec![wire10], surface1.clone());
	let face11 = Face::new(vec![wire11], surface1.clone());
	let geom_shell1: Shell<_, _, _> = vec![face10, face11].into();

	let poly_shell0 = geom_shell0.triangulation(TOL).unwrap();
	let poly_shell1 = geom_shell1.triangulation(TOL).unwrap();
	let (geom_loops_store0, _poly_loops_store0, geom_loops_store1, _poly_loops_store1) =
		create_loops_stores(&geom_shell0, &poly_shell0, &geom_shell1, &poly_shell1, TOL).unwrap();

	let vertex_format = VertexDisplayFormat::AsPoint;
	let edge_id_format = EdgeDisplayFormat::VerticesTupleAndID { vertex_format };
	let wire_id_format = WireDisplayFormat::EdgesListTuple {
		edge_format: edge_id_format,
	};
	let edge_geom_format = EdgeDisplayFormat::VerticesTupleAndCurve { vertex_format };
	let wire_geom_format = WireDisplayFormat::EdgesListTuple {
		edge_format: edge_geom_format,
	};
	assert_eq!(geom_loops_store0.len(), 2);
	assert_eq!(geom_loops_store0[0].len(), 2);
	assert!(
		geom_loops_store0[0][0].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[0][1].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[0][0].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	assert!(
		geom_loops_store0[0][1].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	let (a, b) = (geom_loops_store0[0][0].len(), geom_loops_store0[0][1].len());
	let compatible0 = match geom_loops_store0[0][0].status() {
		ShapesOpStatus::And => a == 2,
		ShapesOpStatus::Or => a == 4,
		_ => false,
	};
	let compatible1 = match geom_loops_store0[0][1].status() {
		ShapesOpStatus::And => b == 2,
		ShapesOpStatus::Or => b == 4,
		_ => false,
	};
	assert!(
		compatible0 && compatible1 && a * b == 8,
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert_eq!(geom_loops_store0[1].len(), 2);
	assert!(
		geom_loops_store0[1][0].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[1][1].is_closed(),
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert!(
		geom_loops_store0[1][0].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	assert!(
		geom_loops_store0[1][1].is_geometric_consistent(),
		"{:?}",
		geom_loops_store0.display(wire_geom_format)
	);
	let (a, b) = (geom_loops_store0[1][0].len(), geom_loops_store0[1][1].len());
	let compatible0 = match geom_loops_store0[1][0].status() {
		ShapesOpStatus::And => a == 2,
		ShapesOpStatus::Or => a == 4,
		_ => false,
	};
	let compatible1 = match geom_loops_store0[1][1].status() {
		ShapesOpStatus::And => b == 2,
		ShapesOpStatus::Or => b == 4,
		_ => false,
	};
	assert!(
		compatible0 && compatible1 && a * b == 8,
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
	assert_eq!(geom_loops_store1.len(), 2);
	assert_eq!(geom_loops_store1[0].len(), 2);
	assert!(
		geom_loops_store1[0][0].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[0][1].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[0][0].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);
	assert!(
		geom_loops_store1[0][1].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);
	let (a, b) = (geom_loops_store1[0][0].len(), geom_loops_store1[0][1].len());
	let compatible0 = match geom_loops_store1[0][0].status() {
		ShapesOpStatus::And => a == 2,
		ShapesOpStatus::Or => a == 4,
		_ => false,
	};
	let compatible1 = match geom_loops_store1[0][1].status() {
		ShapesOpStatus::And => b == 2,
		ShapesOpStatus::Or => b == 4,
		_ => false,
	};
	assert!(
		compatible0 && compatible1 && a * b == 8,
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert_eq!(geom_loops_store1[1].len(), 2);
	assert!(
		geom_loops_store1[1][0].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[1][1].is_closed(),
		"{:?}",
		geom_loops_store1.display(wire_id_format)
	);
	assert!(
		geom_loops_store1[1][0].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);
	assert!(
		geom_loops_store1[1][1].is_geometric_consistent(),
		"{:?}",
		geom_loops_store1.display(wire_geom_format)
	);
	let (a, b) = (geom_loops_store1[1][0].len(), geom_loops_store1[1][1].len());
	let compatible0 = match geom_loops_store1[1][0].status() {
		ShapesOpStatus::And => a == 2,
		ShapesOpStatus::Or => a == 4,
		_ => false,
	};
	let compatible1 = match geom_loops_store1[1][1].status() {
		ShapesOpStatus::And => b == 2,
		ShapesOpStatus::Or => b == 4,
		_ => false,
	};
	assert!(
		compatible0 && compatible1 && a * b == 8,
		"{:?}",
		geom_loops_store0.display(wire_id_format)
	);
}
