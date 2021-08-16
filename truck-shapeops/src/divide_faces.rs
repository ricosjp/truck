#![allow(dead_code)]

use crate::*;
use std::collections::HashMap;
use truck_base::cgmath64::*;
use truck_meshalgo::prelude::*;
use truck_topology::{Vertex, *};

type PolylineCurve = truck_meshalgo::prelude::PolylineCurve<Point3>;

/// Extracts boundaries of each face
#[inline(always)]
fn face_loops_map<P, C, S>(shell: Shell<P, C, S>) -> HashMap<FaceID<S>, Vec<Wire<P, C>>> {
	shell
		.face_iter()
		.map(|face| (face.id(), face.boundaries()))
		.collect()
}

fn reflect_intersection<C, S>(
	poly_curve: PolylineCurve,
	surface0: S,
	surface1: S,
	tol: f64,
	geom_face_id0: FaceID<S>,
	poly_face_id0: FaceID<PolygonMesh>,
	geom_face_id1: FaceID<S>,
	poly_face_id1: FaceID<PolygonMesh>,
	geom_semishell0: &mut HashMap<FaceID<S>, Vec<Wire<Point3, C>>>,
	poly_semishell0: &mut HashMap<FaceID<PolygonMesh>, Vec<Wire<Point3, PolylineCurve>>>,
	geom_semishell1: &mut HashMap<FaceID<S>, Vec<Wire<Point3, C>>>,
	poly_semishell1: &mut HashMap<FaceID<PolygonMesh>, Vec<Wire<Point3, PolylineCurve>>>,
) where
	C: ParametricCurve<Point = Point3, Vector = Vector3>
		+ SearchNearestParameter<Point = Point3, Parameter = f64>
		+ SearchParameter<Point = Point3, Parameter = f64>
		+ Cut
		+ From<IntersectionCurve<S>>,
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>,
{
	let pt0 = *poly_curve.0.first().unwrap();
	let pt1 = *poly_curve.0.last().unwrap();
	if pt0.near(&pt1) {
		add_independent_loop(
			poly_curve,
			surface0,
			surface1,
			tol,
			geom_semishell0.get_mut(&geom_face_id0).unwrap(),
			poly_semishell0.get_mut(&poly_face_id0).unwrap(),
			geom_semishell1.get_mut(&geom_face_id1).unwrap(),
			poly_semishell1.get_mut(&poly_face_id1).unwrap(),
		);
	} else {
		let (pv0, gv0) = add_vertex0(
			poly_semishell0,
			poly_face_id0,
			geom_semishell0,
			geom_face_id0,
			pt0,
		);
		add_vertex1(
			poly_semishell1,
			poly_face_id1,
			geom_semishell1,
			geom_face_id1,
			&pv0,
			&gv0,
		);
		let (pv1, gv1) = add_vertex0(
			poly_semishell0,
			poly_face_id0,
			geom_semishell0,
			geom_face_id0,
			pt1,
		);
		add_vertex1(
			poly_semishell1,
			poly_face_id1,
			geom_semishell1,
			geom_face_id1,
			&pv1,
			&gv1,
		);
		let edge = Edge::new(&pv0, &pv1, poly_curve);
		concat_edge_to_wire(
			edge.clone(),
			poly_semishell0.get_mut(&poly_face_id0).unwrap(),
		);
		concat_edge_to_wire(edge, poly_semishell1.get_mut(&poly_face_id1).unwrap());
	}
}

fn add_independent_loop<C, S>(
	poly_curve: PolylineCurve,
	surface0: S,
	surface1: S,
	tol: f64,
	geom_loops0: &mut Vec<Wire<Point3, C>>,
	poly_loops0: &mut Vec<Wire<Point3, PolylineCurve>>,
	geom_loops1: &mut Vec<Wire<Point3, C>>,
	poly_loops1: &mut Vec<Wire<Point3, PolylineCurve>>,
) where
	C: ParametricCurve<Point = Point3, Vector = Vector3>
		+ SearchNearestParameter<Point = Point3, Parameter = f64>
		+ SearchParameter<Point = Point3, Parameter = f64>
		+ Cut
		+ From<IntersectionCurve<S>>,
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>,
{
	let t = if poly_curve.len() % 2 == 0 {
		poly_curve.len() as f64 / 2.0
	} else {
		(poly_curve.len() - 1) as f64 / 2.0
	};

	let mut poly_curve0 = poly_curve.clone();
	let poly_curve1 = poly_curve0.cut(t);
	let v0 = Vertex::new(poly_curve0.front());
	let v1 = Vertex::new(poly_curve1.front());
	let edge0 = Edge::new(&v0, &v1, poly_curve0);
	let edge1 = Edge::new(&v1, &v0, poly_curve1);
	let poly_wire: Wire<_, _> = vec![edge0, edge1].into();
	poly_loops0.push(poly_wire.inverse());
	poly_loops1.push(poly_wire.inverse());
	poly_loops0.push(poly_wire.clone());
	poly_loops1.push(poly_wire);

	let mut curve0 = IntersectionCurve::try_new(surface0, surface1, poly_curve, tol).unwrap();
	let curve1 = curve0.cut(t);
	let v0 = Vertex::new(curve0.front());
	let v1 = Vertex::new(curve0.back());
	let edge0 = Edge::new(&v0, &v1, curve0.into());
	let edge1 = Edge::new(&v1, &v0, curve1.into());
	let geom_wire: Wire<_, _> = vec![edge0, edge1].into();
	geom_loops0.push(geom_wire.inverse());
	geom_loops1.push(geom_wire.inverse());
	geom_loops0.push(geom_wire.clone());
	geom_loops1.push(geom_wire);
}

fn add_vertex0<C, S>(
	poly_semishell: &mut HashMap<FaceID<PolygonMesh>, Vec<Wire<Point3, PolylineCurve>>>,
	poly_face_id: FaceID<PolygonMesh>,
	geom_semishell: &mut HashMap<FaceID<S>, Vec<Wire<Point3, C>>>,
	geom_face_id: FaceID<S>,
	pt: Point3,
) -> (Vertex<Point3>, Vertex<Point3>)
where
	C: ParametricCurve<Point = Point3, Vector = Vector3>
		+ SearchNearestParameter<Point = Point3, Parameter = f64>
		+ SearchParameter<Point = Point3, Parameter = f64>
		+ Cut,
{
	let poly_loops = poly_semishell.get_mut(&poly_face_id).unwrap();
	let geom_loops = geom_semishell.get(&geom_face_id).unwrap();

	let (i, j, t) = poly_loops
		.iter()
		.enumerate()
		.flat_map(move |(i, wire)| wire.iter().enumerate().map(move |(j, edge)| (i, j, edge)))
		.find_map(|(i, j, edge)| {
			let poly_curve = edge.get_curve();
			poly_curve
				.search_parameter(pt, None, 1)
				.filter(|t| -TOLERANCE < *t && t + 1.0 < poly_loops.len() as f64 + TOLERANCE)
				.map(|t| (i, j, t))
		})
		.unwrap();
	if t.so_small() {
		(
			poly_loops[i][j].front().clone(),
			geom_loops[i][j].front().clone(),
		)
	} else if (t + 1.0).near(&(poly_loops.len() as f64)) {
		(
			poly_loops[i][j].back().clone(),
			geom_loops[i][j].back().clone(),
		)
	} else {
		let v0 = Vertex::new(pt);
		let (edge0, edge1) = poly_loops[i][j].cut(&v0).unwrap();
		let wire0: Wire<_, _> = vec![edge0, edge1].into();
		let edge_id = poly_loops[i][j].id();
		poly_semishell.values_mut().flatten().for_each(|wire| {
			let idx = wire
				.iter()
				.enumerate()
				.find(|(_, edge)| edge.id() == edge_id)
				.unwrap()
				.0;
			wire.swap_edge_into_wire(idx, wire0.clone());
		});
		let curve = geom_loops[i][j].get_curve();
		let t = curve.search_nearest_parameter(pt, None, 100).unwrap();
		let pt = curve.subs(t);
		let v1 = Vertex::new(pt);
		let (edge0, edge1) = geom_loops[i][j].cut(&v1).unwrap();
		let wire0: Wire<_, _> = vec![edge0, edge1].into();
		let edge_id = geom_loops[i][j].id();
		geom_semishell.values_mut().flatten().for_each(|wire| {
			let idx = wire
				.iter()
				.enumerate()
				.find(|(_, edge)| edge.id() == edge_id)
				.unwrap()
				.0;
			wire.swap_edge_into_wire(idx, wire0.clone());
		});
		(v0, v1)
	}
}

fn add_vertex1<C, S>(
	poly_semishell: &mut HashMap<FaceID<PolygonMesh>, Vec<Wire<Point3, PolylineCurve>>>,
	poly_face_id: FaceID<PolygonMesh>,
	geom_semishell: &mut HashMap<FaceID<S>, Vec<Wire<Point3, C>>>,
	geom_face_id: FaceID<S>,
	pv: &Vertex<Point3>,
	gv: &Vertex<Point3>,
) where
	C: ParametricCurve<Point = Point3, Vector = Vector3>
		+ SearchNearestParameter<Point = Point3, Parameter = f64>
		+ SearchParameter<Point = Point3, Parameter = f64>
		+ Cut,
{
	let pt = pv.get_point();
	let poly_loops = poly_semishell.get_mut(&poly_face_id).unwrap();

	let (i, j, t) = poly_loops
		.iter()
		.enumerate()
		.flat_map(move |(i, wire)| wire.iter().enumerate().map(move |(j, edge)| (i, j, edge)))
		.find_map(|(i, j, edge)| {
			let poly_curve = edge.get_curve();
			poly_curve
				.search_parameter(pt, None, 1)
				.filter(|t| -TOLERANCE < *t && t + 1.0 < poly_loops.len() as f64 + TOLERANCE)
				.map(|t| (i, j, t))
		})
		.unwrap();
	if TOLERANCE < t && t + 1.0 < poly_loops.len() as f64 - TOLERANCE {
		let (edge0, edge1) = poly_loops[i][j].cut(&pv).unwrap();
		let wire0: Wire<_, _> = vec![edge0, edge1].into();
		let edge_id = poly_loops[i][j].id();
		poly_semishell.values_mut().flatten().for_each(|wire| {
			let idx = wire
				.iter()
				.enumerate()
				.find(|(_, edge)| edge.id() == edge_id)
				.unwrap()
				.0;
			wire.swap_edge_into_wire(idx, wire0.clone());
		});
	} else {
		let v0 = if t.so_small() {
			poly_loops[i][j].front().clone()
		} else {
			poly_loops[i][j].back().clone()
		};
		if &v0 == pv {
			return;
		}
		let mut edge_map = HashMap::<EdgeID<PolylineCurve>, Edge<Point3, PolylineCurve>>::new();
		poly_semishell
			.values_mut()
			.flatten()
			.flat_map(|wire| wire.iter_mut())
			.for_each(|edge| {
				if let Some(new_edge) = edge_map.get(&edge.id()) {
					if edge.orientation() {
						*edge = new_edge.clone();
					} else {
						*edge = new_edge.inverse();
					}
				} else {
					let mut new_edge = if &v0 == edge.absolute_front() {
						Edge::new(&v0, edge.absolute_back(), edge.get_curve())
					} else if &v0 == edge.absolute_back() {
						Edge::new(edge.absolute_front(), &v0, edge.get_curve())
					} else {
						return;
					};
					if !edge.orientation() {
						new_edge.invert();
					}
					edge_map.insert(edge.id(), new_edge.clone());
					*edge = new_edge;
				}
			});
	}
}

fn concat_edge_to_wire<P, C>(edge: Edge<P, C>, wires: &mut Vec<Wire<P, C>>) {
	let v0 = edge.front().clone();
	let v1 = edge.back().clone();
	let (i0, j0) = wires
		.iter_mut()
		.enumerate()
		.find_map(|(i, wire)| {
			wire.iter()
				.enumerate()
				.find(|(_, e)| e.front() == &v0)
				.map(|(j, _)| (i, j))
		})
		.unwrap();
	wires[i0].rotate_left(j0);
	wires[i0].push_front(edge.clone());
	wires[i0].push_back(edge.inverse());
	let (i1, j1) = wires
		.iter_mut()
		.enumerate()
		.find_map(|(i, wire)| {
			wire.iter()
				.enumerate()
				.find(|(_, e)| e.front() == &v1)
				.map(|(j, _)| (i, j))
		})
		.unwrap();
	if i0 == i1 {
		let new_wire = wires[i0].split_off(j1);
		wires.push(new_wire);
	} else {
		wires[i1].rotate_left(j1);
		let mut new_wire = wires[i1].clone();
		wires[i0].append(&mut new_wire);
		wires.swap_remove(i1);
	}
}

fn add_vertex<C, S>(
	geom_shell: &mut Shell<Point3, C, S>,
	poly_shell: &mut Shell<Point3, PolylineCurve, PolygonMesh>,
	pt: Point3,
) -> Option<(Vertex<Point3>, Vertex<Point3>)>
where
	C: ParametricCurve<Point = Point3, Vector = Vector3>
		+ SearchNearestParameter<Point = Point3, Parameter = f64>
		+ SearchParameter<Point = Point3, Parameter = f64>
		+ Cut,
{
	let (t, len, geom_edge, poly_edge) = geom_shell
		.edge_iter()
		.zip(poly_shell.edge_iter())
		.find_map(|(geom_edge, poly_edge)| {
			let polyline = poly_edge.get_curve();
			if let Some(t) = polyline.search_parameter(pt, None, 1) {
				Some((t, polyline.len() as f64 - 1.0, geom_edge, poly_edge))
			} else {
				None
			}
		})?;

	if t.so_small() {
		Some((
			poly_edge.absolute_front().clone(),
			geom_edge.absolute_front().clone(),
		))
	} else if t.near(&len) {
		Some((
			poly_edge.absolute_back().clone(),
			geom_edge.absolute_back().clone(),
		))
	} else {
		let poly_edge_id = poly_edge.id();
		let v0 = Vertex::new(pt);
		poly_shell.cut_edge(poly_edge_id, &v0);

		let geom_edge_id = geom_edge.id();
		let curve = geom_edge.get_curve();
		let t = curve.search_nearest_parameter(pt, None, 100)?;
		let v1 = Vertex::new(curve.subs(t));
		geom_shell.cut_edge(geom_edge_id, &v1);
		Some((v0, v1))
	}
}

fn intersection_curves<C, S>(
	geom_shell0: &mut Shell<Point3, C, S>,
	poly_shell0: &mut Shell<Point3, PolylineCurve, PolygonMesh>,
	geom_shell1: &mut Shell<Point3, C, S>,
	poly_shell1: &mut Shell<Point3, PolylineCurve, PolygonMesh>,
	tol: f64,
) -> Vec<Vec<()>>
where
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>,
{
	let (len0, len1) = (geom_shell0.len(), geom_shell1.len());
	(0..len0)
		.map(move |i| {
			let surface0 = geom_shell0[i].get_surface();
			let polygon0 = poly_shell0[i].get_surface();
			(0..len1)
				.map(|j| {
					let surface1 = geom_shell1[j].get_surface();
					let polygon1 = poly_shell1[j].get_surface();
					let intersection_curves: Vec<IntersectionCurve<S>> =
						intersection_curve::intersection_curves(
							surface0.clone(),
							&polygon0,
							surface1,
							&polygon1,
							tol,
						)
						.into_iter()
						.filter_map(Into::into)
						.collect();
					intersection_curves.into_iter().for_each(|curve| {
						let pt0 = curve.front();
						let pt1 = curve.back();
					});
				})
				.collect()
		})
		.collect()
}
