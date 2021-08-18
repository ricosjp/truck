use crate::*;
use std::collections::HashMap;
use truck_base::cgmath64::*;
use truck_meshalgo::prelude::*;
use truck_topology::{Vertex, *};

type PolylineCurve = truck_meshalgo::prelude::PolylineCurve<Point3>;

#[derive(Clone, Debug)]
struct Loops<P, C>(Vec<Wire<P, C>>);
#[derive(Clone, Debug)]
struct LoopsStore<P, C>(Vec<Loops<P, C>>);

impl<P, C> std::ops::Deref for Loops<P, C> {
	type Target = Vec<Wire<P, C>>;
	#[inline(always)]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<P, C> std::ops::DerefMut for Loops<P, C> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<P, C> std::ops::Deref for LoopsStore<P, C> {
	type Target = Vec<Loops<P, C>>;
	#[inline(always)]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<P, C> std::ops::DerefMut for LoopsStore<P, C> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<'a, P, C, S> From<&'a Face<P, C, S>> for Loops<P, C> {
	#[inline(always)]
	fn from(face: &'a Face<P, C, S>) -> Loops<P, C> { Loops(face.absolute_boundaries().clone()) }
}

impl<'a, P, C, S> std::iter::FromIterator<&'a Face<P, C, S>> for LoopsStore<P, C> {
	fn from_iter<I: IntoIterator<Item = &'a Face<P, C, S>>>(iter: I) -> Self {
		Self(iter.into_iter().map(|face| Loops::from(face)).collect())
	}
}

#[derive(Clone, Debug, Copy, PartialEq)]
enum ParameterKind {
	Front,
	Back,
	Inner(f64),
}

impl ParameterKind {
	fn try_new(t: f64, (t0, t1): (f64, f64)) -> Option<ParameterKind> {
		if t0.near(&t) {
			Some(ParameterKind::Front)
		} else if t1.near(&t) {
			Some(ParameterKind::Back)
		} else if t0 < t && t < t1 {
			Some(ParameterKind::Inner(t))
		} else {
			None
		}
	}
}

impl<P: Copy, C: Clone> Loops<P, C> {
	fn search_parameter(&self, pt: P) -> Option<(usize, usize, ParameterKind)>
	where C: ParametricCurve<Point = P> + SearchParameter<Point = P, Parameter = f64> {
		self.iter()
			.enumerate()
			.flat_map(move |(i, wire)| wire.iter().enumerate().map(move |(j, edge)| (i, j, edge)))
			.find_map(|(i, j, edge)| {
				let curve = edge.get_curve();
				curve.search_parameter(pt, None, 1).and_then(|t| {
					let kind = ParameterKind::try_new(t, curve.parameter_range())?;
					Some((i, j, kind))
				})
			})
	}

	fn change_vertex(&mut self, old_vertex: &Vertex<P>, new_vertex: &Vertex<P>) {
		let mut emap = HashMap::<EdgeID<C>, Edge<P, C>>::new();
		self.iter_mut()
			.flat_map(|wire| wire.iter_mut())
			.for_each(|edge| {
				let mut new_edge = if edge.absolute_front() == old_vertex {
					if let Some(new_edge) = emap.get(&edge.id()) {
						new_edge.clone()
					} else {
						let new_edge =
							Edge::new(new_vertex, edge.absolute_back(), edge.get_curve());
						emap.insert(edge.id(), new_edge.clone());
						new_edge
					}
				} else if edge.absolute_back() == old_vertex {
					if let Some(new_edge) = emap.get(&edge.id()) {
						new_edge.clone()
					} else {
						let new_edge =
							Edge::new(edge.absolute_front(), new_vertex, edge.get_curve());
						emap.insert(edge.id(), new_edge.clone());
						new_edge
					}
				} else {
					return;
				};
				if !edge.orientation() {
					new_edge.invert();
				}
				*edge = new_edge;
			})
	}

	fn swap_edge_into_wire(&mut self, edge_id: EdgeID<C>, new_wire: &Wire<P, C>) {
		self.iter_mut().for_each(|wire| {
			let mut iter = wire.iter().enumerate();
			if let Some((idx, _)) = iter.find(|(_, edge)| edge.id() == edge_id) {
				wire.swap_edge_into_wire(idx, new_wire.clone());
			}
		});
	}

	#[inline(always)]
	fn add_independent_loop(&mut self, r#loop: Wire<P, C>) {
		self.push(r#loop.inverse());
		self.push(r#loop);
	}

	fn add_edge(&mut self, edge0: Edge<P, C>) -> (Option<(usize, usize)>, Option<(usize, usize)>) {
		let a = self.iter().enumerate().find_map(|(i, wire)| {
			wire.iter().enumerate().find_map(|(j, edge)| {
				if edge.front() == edge0.back() {
					Some((i, j))
				} else {
					None
				}
			})
		});
		if let Some((wire_index0, edge_index0)) = a {
			self[wire_index0].rotate_left(edge_index0);
			self[wire_index0].push_front(edge0.clone());
			self[wire_index0].push_back(edge0.inverse());
		}
		let b = self.iter().enumerate().find_map(|(i, wire)| {
			wire.iter().enumerate().find_map(|(j, edge)| {
				if edge.front() == edge0.front() {
					Some((i, j))
				} else {
					None
				}
			})
		});
		match (a, b) {
			(Some((wire_index0, _)), Some((wire_index1, edge_index1))) => {
				if wire_index0 == wire_index1 {
					let new_wire = self[wire_index0].split_off(edge_index1);
					self.push(new_wire);
				} else {
					let mut new_wire0 = self[wire_index1].clone();
					let mut new_wire1 = new_wire0.split_off(edge_index1);
					new_wire0.append(&mut self[wire_index0]);
					new_wire0.append(&mut new_wire1);
					self[wire_index0] = new_wire0;
					self.swap_remove(wire_index1);
				}
			}
			(None, Some((wire_index1, edge_index1))) => {
				self[wire_index1].rotate_left(edge_index1);
				self[wire_index1].push_front(edge0.inverse());
				self[wire_index1].push_back(edge0);
			}
			(None, None) => self.push(vec![edge0.inverse(), edge0].into()),
			_ => {}
		}
		(a, b)
	}
}

impl<P: Copy + Tolerance, C: Clone> LoopsStore<P, C> {
	#[inline(always)]
	fn change_vertex(&mut self, old_vertex: &Vertex<P>, new_vertex: &Vertex<P>) {
		self.iter_mut()
			.for_each(|loops| loops.change_vertex(old_vertex, new_vertex));
	}

	#[inline(always)]
	fn swap_edge_into_wire(&mut self, edge_id: EdgeID<C>, new_wire: &Wire<P, C>) {
		self.iter_mut()
			.for_each(|loops| loops.swap_edge_into_wire(edge_id, new_wire))
	}

	fn add_polygon_vertex(
		&mut self,
		loops_index: usize,
		v: &Vertex<P>,
	) -> Option<(usize, usize, ParameterKind)>
	where
		C: Cut<Point = P> + SearchParameter<Point = P, Parameter = f64>,
	{
		let pt = v.get_point();
		let (wire_index, edge_index, kind) = self[loops_index].search_parameter(pt)?;
		match kind {
			ParameterKind::Front => {
				let old_vertex = self[loops_index][wire_index][edge_index].front().clone();
				self.change_vertex(&old_vertex, v);
			}
			ParameterKind::Back => {
				let old_vertex = self[loops_index][wire_index][edge_index].back().clone();
				self.change_vertex(&old_vertex, v);
			}
			ParameterKind::Inner(t) => {
				let edge = &self[loops_index][wire_index][edge_index];
				let edge_id = edge.id();
				let (edge0, edge1) = edge.cut_with_parameter(v, t)?;
				let new_wire: Wire<_, _> = vec![edge0, edge1].into();
				self.swap_edge_into_wire(edge_id, &new_wire);
			}
		}
		Some((wire_index, edge_index, kind))
	}
}

impl<C> LoopsStore<Point3, C> {
	fn add_geom_vertex<S>(
		&mut self,
		loops_index: usize,
		wire_index: usize,
		edge_index: usize,
		v: &Vertex<Point3>,
		kind: ParameterKind,
		another_surface: &S,
	) -> Option<()>
	where
		C: Cut<Point = Point3, Vector = Vector3>
			+ SearchNearestParameter<Point = Point3, Parameter = f64>,
		S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>,
	{
		match kind {
			ParameterKind::Front => {
				let old_vertex = self[loops_index][wire_index][edge_index].front().clone();
				self.change_vertex(&old_vertex, v);
			}
			ParameterKind::Back => {
				let old_vertex = self[loops_index][wire_index][edge_index].back().clone();
				self.change_vertex(&old_vertex, v);
			}
			ParameterKind::Inner(_) => {
				let curve = self[loops_index][wire_index][edge_index].get_curve();
				let (pt, t, _) = curve_surface_projection(
					&curve,
					None,
					another_surface,
					None,
					v.get_point(),
					100,
				)?;
				v.set_point(pt);
				let edge = &self[loops_index][wire_index][edge_index];
				let edge_id = edge.id();
				let (edge0, edge1) = edge.cut_with_parameter(v, t)?;
				let new_wire: Wire<_, _> = vec![edge0, edge1].into();
				self.swap_edge_into_wire(edge_id, &new_wire);
			}
		}
		Some(())
	}
}

fn curve_surface_projection<C, S>(
	curve: &C,
	curve_hint: Option<f64>,
	surface: &S,
	surface_hint: Option<(f64, f64)>,
	point: Point3,
	trials: usize,
) -> Option<(Point3, f64, Point2)>
where
	C: ParametricCurve<Point = Point3, Vector = Vector3>
		+ SearchNearestParameter<Point = Point3, Parameter = f64>,
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>,
{
	if trials == 0 {
		return None;
	}
	let t = curve.search_nearest_parameter(point, curve_hint, 10)?;
	let pt0 = curve.subs(t);
	let (u, v) = surface.search_nearest_parameter(point, surface_hint, 10)?;
	let pt1 = surface.subs(u, v);
	if point.near(&pt0) && point.near(&pt1) && pt0.near(&pt1) {
		Some((point, t, Point2::new(u, v)))
	} else {
		let l = curve.der(t);
		let n = surface.normal(u, v);
		let t0 = (pt1 - pt0).dot(n) / l.dot(n);
		curve_surface_projection(
			curve,
			Some(t),
			surface,
			Some((u, v)),
			pt0 + t0 * l,
			trials - 1,
		)
	}
}

#[cfg_attr(rustfmt, rustfmt_skip)]
trait Length { fn len(&self) -> usize; }
#[cfg_attr(rustfmt, rustfmt_skip)]
impl Length for PolylineCurve { fn len(&self) -> usize { self.0.len() } }
#[cfg_attr(rustfmt, rustfmt_skip)]
impl<S> Length for IntersectionCurve<S> { fn len(&self) -> usize { self.polyline().len() } }

fn create_independent_loop<P, C, D>(poly_curve: C) -> Wire<P, D>
where
	C: Cut<Point = P> + Length,
	D: From<C>, {
	let t = if poly_curve.len() % 2 == 0 {
		poly_curve.len() as f64 / 2.0
	} else {
		(poly_curve.len() - 1) as f64 / 2.0
	};

	let mut poly_curve0 = poly_curve.clone();
	let poly_curve1 = poly_curve0.cut(t);
	let v0 = Vertex::new(poly_curve0.front());
	let v1 = Vertex::new(poly_curve1.front());
	let edge0 = Edge::new(&v0, &v1, poly_curve0.into());
	let edge1 = Edge::new(&v1, &v0, poly_curve1.into());
	vec![edge0, edge1].into()
}

pub fn polyline_stores<C, S>(
	geom_shell0: &Shell<Point3, C, S>,
	poly_shell0: &Shell<Point3, PolylineCurve, PolygonMesh>,
	geom_shell1: &Shell<Point3, C, S>,
	poly_shell1: &Shell<Point3, PolylineCurve, PolygonMesh>,
	tol: f64,
) -> Option<()>
where
	C: SearchNearestParameter<Point = Point3, Parameter = f64>
		+ SearchParameter<Point = Point3, Parameter = f64>
		+ Cut<Point = Point3, Vector = Vector3>
		+ From<IntersectionCurve<S>>,
	S: ParametricSurface3D + SearchNearestParameter<Point = Point3, Parameter = (f64, f64)>,
{
	let mut geom_loops_store0: LoopsStore<_, _> = geom_shell0.face_iter().collect();
	let mut poly_loops_store0: LoopsStore<_, _> = poly_shell0.face_iter().collect();
	let mut geom_loops_store1: LoopsStore<_, _> = geom_shell1.face_iter().collect();
	let mut poly_loops_store1: LoopsStore<_, _> = poly_shell1.face_iter().collect();
	let store0_len = geom_loops_store0.len();
	let store1_len = geom_loops_store1.len();
	(0..store0_len)
		.flat_map(move |i| (0..store1_len).map(move |j| (i, j)))
		.try_for_each(|(face_index0, face_index1)| {
			let surface0 = geom_shell0[face_index0].get_surface();
			let surface1 = geom_shell1[face_index1].get_surface();
			let polygon0 = poly_shell0[face_index0].get_surface();
			let polygon1 = poly_shell1[face_index1].get_surface();
			intersection_curve::intersection_curves(
				surface0.clone(),
				&polygon0,
				surface1.clone(),
				&polygon1,
				tol,
			)
			.into_iter()
			.try_for_each(|(polyline, intersection_curve)| {
				let mut intersection_curve = intersection_curve?;
				if polyline.front().near(&polyline.back()) {
					let poly_wire = create_independent_loop(polyline);
					poly_loops_store0[face_index0].add_independent_loop(poly_wire.clone());
					poly_loops_store1[face_index1].add_independent_loop(poly_wire);
					let geom_wire = create_independent_loop(intersection_curve);
					geom_loops_store0[face_index0].add_independent_loop(geom_wire.clone());
					geom_loops_store1[face_index0].add_independent_loop(geom_wire);
				} else {
					let pv0 = Vertex::new(polyline.front());
					let pv1 = Vertex::new(polyline.back());
					let gv0 = Vertex::new(polyline.front());
					let gv1 = Vertex::new(polyline.back());
					let idx0 = poly_loops_store0.add_polygon_vertex(face_index0, &pv0);
					if let Some((wire_index, edge_index, kind)) = idx0 {
						geom_loops_store0.add_geom_vertex(
							face_index0,
							wire_index,
							edge_index,
							&gv0,
							kind,
							&surface1,
						);
						let polyline = intersection_curve.polyline_mut();
						*polyline.first_mut().unwrap() = gv0.get_point();
					}
					let idx1 = poly_loops_store0.add_polygon_vertex(face_index0, &pv1);
					if let Some((wire_index, edge_index, kind)) = idx1 {
						geom_loops_store0.add_geom_vertex(
							face_index0,
							wire_index,
							edge_index,
							&gv1,
							kind,
							&surface1,
						);
						let polyline = intersection_curve.polyline_mut();
						*polyline.last_mut().unwrap() = gv1.get_point();
					}
					let pedge = Edge::new(&pv0, &pv1, polyline);
					let gedge = Edge::new(&gv0, &gv1, intersection_curve.into());
					poly_loops_store0[face_index0].add_edge(pedge.clone());
					geom_loops_store0[face_index0].add_edge(gedge.clone());

					let idx0 = poly_loops_store1.add_polygon_vertex(face_index1, &pv0);
					if let Some((wire_index, edge_index, kind)) = idx0 {
						geom_loops_store1.add_geom_vertex(
							face_index1,
							wire_index,
							edge_index,
							&gv0,
							kind,
							&surface0,
						);
					}
					let idx1 = poly_loops_store1.add_polygon_vertex(face_index1, &pv1);
					if let Some((wire_index, edge_index, kind)) = idx1 {
						geom_loops_store1.add_geom_vertex(
							face_index1,
							wire_index,
							edge_index,
							&gv1,
							kind,
							&surface1,
						);
					}
					poly_loops_store1[face_index1].add_edge(pedge.clone());
					geom_loops_store1[face_index1].add_edge(gedge.clone());
	
				}
				Some(())
			})
		})
}
