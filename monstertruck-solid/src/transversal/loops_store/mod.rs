#![allow(clippy::many_single_char_names)]

use super::*;
use monstertruck_core::cgmath64::*;
use monstertruck_geometry::prelude::*;
use monstertruck_meshing::prelude::*;
use monstertruck_topology::{Vertex, *};
use rustc_hash::FxHashMap as HashMap;
use std::cmp::Ordering;
use thiserror::Error;

type PolylineCurve = monstertruck_meshing::prelude::PolylineCurve<Point3>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapesOpStatus {
    Unknown,
    And,
    Or,
}

impl ShapesOpStatus {
    fn not(self) -> Self {
        match self {
            Self::Unknown => Self::Unknown,
            Self::And => Self::Or,
            Self::Or => Self::And,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BoundaryWire<P, C> {
    wire: Wire<P, C>,
    status: ShapesOpStatus,
}

impl<P, C> BoundaryWire<P, C> {
    #[inline(always)]
    pub fn new(wire: Wire<P, C>, status: ShapesOpStatus) -> Self { Self { wire, status } }
    #[inline(always)]
    pub fn status(&self) -> ShapesOpStatus { self.status }
    #[inline(always)]
    pub fn invert(&mut self) {
        self.wire.invert();
        self.status = self.status.not();
    }
    #[inline(always)]
    pub fn inverse(&self) -> Self {
        Self {
            wire: self.wire.inverse(),
            status: self.status.not(),
        }
    }
}

impl ShapesOpStatus {
    fn from_is_curve<C, S0, S1>(curve: &IntersectionCurve<C, S0, S1>) -> Option<ShapesOpStatus>
    where
        C: ParametricCurve3D + BoundedCurve,
        S0: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
        S1: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>, {
        let (t0, t1) = curve.range_tuple();
        let t = (t0 + t1) / 2.0;
        let (_, pt0, pt1) = curve.search_triple(t, 100)?;
        let der = curve.leader().der(t);
        let normal0 = curve.surface0().normal(pt0[0], pt0[1]);
        let normal1 = curve.surface1().normal(pt1[0], pt1[1]);
        match normal0.cross(der).dot(normal1) > 0.0 {
            true => Some(ShapesOpStatus::Or),
            false => Some(ShapesOpStatus::And),
        }
    }
}

impl<P, C> std::ops::Deref for BoundaryWire<P, C> {
    type Target = Wire<P, C>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target { &self.wire }
}

impl<P, C> std::ops::DerefMut for BoundaryWire<P, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.wire }
}

#[derive(Clone, Debug)]
pub struct Loops<P, C>(Vec<BoundaryWire<P, C>>);
#[derive(Clone, Debug)]
pub struct LoopsStore<P, C>(Vec<Loops<P, C>>);

impl<P, C> std::ops::Deref for Loops<P, C> {
    type Target = Vec<BoundaryWire<P, C>>;
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

impl<P, C> FromIterator<BoundaryWire<P, C>> for Loops<P, C> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = BoundaryWire<P, C>>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl<'a, P, C, S> From<&'a Face<P, C, S>> for Loops<P, C> {
    #[inline(always)]
    fn from(face: &'a Face<P, C, S>) -> Loops<P, C> {
        face.absolute_boundaries()
            .iter()
            .map(|wire| BoundaryWire::new(wire.clone(), ShapesOpStatus::Unknown))
            .collect()
    }
}

impl<'a, P: 'a, C: 'a, S: 'a> FromIterator<&'a Face<P, C, S>> for LoopsStore<P, C> {
    fn from_iter<I: IntoIterator<Item = &'a Face<P, C, S>>>(iter: I) -> Self {
        Self(iter.into_iter().map(Loops::from).collect())
    }
}

impl<'a, P, C> IntoIterator for &'a LoopsStore<P, C> {
    type Item = <&'a Vec<Loops<P, C>> as IntoIterator>::Item;
    type IntoIter = <&'a Vec<Loops<P, C>> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
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

    fn from_nearest(t: f64, (t0, t1): (f64, f64)) -> Option<ParameterKind> {
        Self::try_new(t, (t0, t1)).or(if t <= t0 + TOLERANCE {
            Some(ParameterKind::Front)
        } else if t1 - TOLERANCE <= t {
            Some(ParameterKind::Back)
        } else if t0 < t && t < t1 {
            Some(ParameterKind::Inner(t))
        } else {
            None
        })
    }
}

impl<P: Copy, C: Clone> Loops<P, C> {
    fn search_parameter(&self, pt: P) -> Option<(usize, usize, ParameterKind)>
    where
        P: Tolerance,
        C: BoundedCurve<Point = P>
            + SearchParameter<D1, Point = P>
            + SearchNearestParameter<D1, Point = P>, {
        self.iter()
            .enumerate()
            .flat_map(move |(i, wire)| wire.iter().enumerate().map(move |(j, edge)| (i, j, edge)))
            .find_map(|(i, j, edge)| {
                let curve = edge.curve();
                curve
                    .search_parameter(pt, None, 20)
                    .or_else(|| {
                        let t = curve.search_nearest_parameter(pt, None, 20)?;
                        curve.subs(t).near(&pt).then_some(t)
                    })
                    .and_then(|t| {
                        let kind = ParameterKind::try_new(t, curve.range_tuple())?;
                        Some((i, j, kind))
                    })
            })
    }

    fn change_vertex(
        &mut self,
        old_vertex: &Vertex<P>,
        new_vertex: &Vertex<P>,
        emap: &mut HashMap<EdgeId<C>, Edge<P, C>>,
    ) {
        self.iter_mut()
            .flat_map(|wire| wire.iter_mut())
            .for_each(|edge| {
                let mut new_edge = if edge.absolute_front() == old_vertex {
                    emap.entry(edge.id()).or_insert_with(|| {
                        Edge::new(new_vertex, edge.absolute_back(), edge.curve())
                    })
                } else if edge.absolute_back() == old_vertex {
                    emap.entry(edge.id()).or_insert_with(|| {
                        Edge::new(edge.absolute_front(), new_vertex, edge.curve())
                    })
                } else {
                    return;
                }
                .clone();
                if !edge.orientation() {
                    new_edge.invert();
                }
                // Remove the edge from the HashMap when it is no longer there because Id reassignment will occur.
                if edge.count() == 1 {
                    emap.remove(&edge.id());
                }
                *edge = new_edge;
            })
    }

    fn swap_edge_into_wire(&mut self, edge_id: EdgeId<C>, new_wire: &Wire<P, C>) {
        self.iter_mut().for_each(|wire| {
            let mut iter = wire.iter().enumerate();
            if let Some((idx, edge)) = iter.find(|(_, edge)| edge.id() == edge_id) {
                if edge.orientation() {
                    wire.swap_edge_into_wire(idx, new_wire.clone());
                } else {
                    wire.swap_edge_into_wire(idx, new_wire.inverse());
                }
            }
        });
    }

    #[inline(always)]
    fn add_independent_loop(&mut self, r#loop: BoundaryWire<P, C>) {
        self.push(r#loop.inverse());
        self.push(r#loop);
    }

    fn add_edge(
        &mut self,
        edge0: Edge<P, C>,
        status: ShapesOpStatus,
    ) -> [Option<(usize, usize)>; 2] {
        let a = self.iter().enumerate().find_map(|(i, wire)| {
            wire.iter().enumerate().find_map(|(j, edge)| {
                if edge.front() == edge0.back() {
                    Some((i, j))
                } else {
                    None
                }
            })
        });
        let b = self.iter().enumerate().find_map(|(i, wire)| {
            wire.iter().enumerate().find_map(|(j, edge)| {
                if edge.front() == edge0.front() {
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
        match (a, b) {
            (Some((wire_index0, edge_index0)), Some((wire_index1, edge_index1))) => {
                if wire_index0 == wire_index1 {
                    let len = self[wire_index0].len() - 2;
                    let edge_index1 = (len + edge_index1 - edge_index0) % len + 1;
                    let new_wire = self[wire_index0].split_off(edge_index1);
                    self[wire_index0].status = status;
                    self.push(BoundaryWire::new(new_wire, status.not()));
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
            (None, None) => self.push(BoundaryWire::new(
                vec![edge0.inverse(), edge0].into(),
                ShapesOpStatus::Unknown,
            )),
            _ => {}
        }
        [a, b]
    }
}

impl<P: Copy + Tolerance, C: Clone> LoopsStore<P, C> {
    #[inline(always)]
    fn change_vertex(
        &mut self,
        old_vertex: &Vertex<P>,
        new_vertex: &Vertex<P>,
        emap: &mut HashMap<EdgeId<C>, Edge<P, C>>,
    ) {
        self.iter_mut()
            .for_each(|loops| loops.change_vertex(old_vertex, new_vertex, emap));
    }

    #[inline(always)]
    fn swap_edge_into_wire(&mut self, edge_id: EdgeId<C>, new_wire: &Wire<P, C>) {
        self.iter_mut()
            .for_each(|loops| loops.swap_edge_into_wire(edge_id, new_wire))
    }
}

impl<C: Clone> Loops<Point3, C> {
    fn nearest_distance2(&self, pt: Point3) -> Option<f64>
    where C: BoundedCurve<Point = Point3> + SearchNearestParameter<D1, Point = Point3> {
        self.iter()
            .flat_map(|wire| wire.iter())
            .filter_map(|edge| {
                let curve = edge.curve();
                let t = curve.search_nearest_parameter(pt, None, 20)?;
                Some((curve.subs(t) - pt).magnitude2())
            })
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
    }

    fn search_parameter_with_tolerance(
        &self,
        pt: Point3,
        snap_tol: f64,
    ) -> Option<(usize, usize, ParameterKind)>
    where
        C: BoundedCurve<Point = Point3>
            + SearchParameter<D1, Point = Point3>
            + SearchNearestParameter<D1, Point = Point3>,
    {
        self.search_parameter(pt).or_else(|| {
            self.iter()
                .enumerate()
                .flat_map(move |(i, wire)| {
                    wire.iter().enumerate().map(move |(j, edge)| (i, j, edge))
                })
                .filter_map(|(i, j, edge)| {
                    let curve = edge.curve();
                    let t = curve.search_nearest_parameter(pt, None, 20)?;
                    let kind = ParameterKind::from_nearest(t, curve.range_tuple())?;
                    let t = match kind {
                        ParameterKind::Front => curve.range_tuple().0,
                        ParameterKind::Back => curve.range_tuple().1,
                        ParameterKind::Inner(t) => t,
                    };
                    let dist2 = (curve.subs(t) - pt).magnitude2();
                    Some((dist2, i, j, kind))
                })
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal))
                .and_then(|(dist2, i, j, kind)| {
                    (dist2 <= snap_tol * snap_tol).then_some((i, j, kind))
                })
        })
    }
}

impl<C> LoopsStore<Point3, C> {
    fn has_edge_between_points(
        &self,
        loops_index: usize,
        p0: Point3,
        p1: Point3,
        snap_tol: f64,
    ) -> bool {
        self[loops_index]
            .iter()
            .flat_map(|wire| wire.iter())
            .any(|edge| {
                let q0 = edge.front().point();
                let q1 = edge.back().point();
                (q0.distance2(p0) <= snap_tol * snap_tol && q1.distance2(p1) <= snap_tol * snap_tol)
                    || (q0.distance2(p1) <= snap_tol * snap_tol
                        && q1.distance2(p0) <= snap_tol * snap_tol)
            })
    }

    fn find_near_vertex(
        &self,
        loops_index: usize,
        point: Point3,
        snap_tol: f64,
    ) -> Option<Vertex<Point3>> {
        self[loops_index]
            .iter()
            .flat_map(|wire| wire.iter())
            .flat_map(|edge| [edge.absolute_front().clone(), edge.absolute_back().clone()])
            .filter_map(|vertex| {
                let dist2 = (vertex.point() - point).magnitude2();
                (dist2 <= snap_tol * snap_tol).then_some((dist2, vertex))
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal))
            .map(|(_, vertex)| vertex)
    }

    fn add_polygon_vertex_with_tolerance(
        &mut self,
        loops_index: usize,
        v: &Vertex<Point3>,
        snap_tol: f64,
        emap: &mut HashMap<EdgeId<C>, Edge<Point3, C>>,
    ) -> Option<(usize, usize, ParameterKind)>
    where
        C: Cut<Point = Point3>
            + SearchParameter<D1, Point = Point3>
            + SearchNearestParameter<D1, Point = Point3>,
    {
        let pt = v.point();
        let (wire_index, edge_index, kind) =
            self[loops_index].search_parameter_with_tolerance(pt, snap_tol)?;
        match kind {
            ParameterKind::Front => {
                let old_vertex = self[loops_index][wire_index][edge_index]
                    .absolute_front()
                    .clone();
                self.change_vertex(&old_vertex, v, emap);
            }
            ParameterKind::Back => {
                let old_vertex = self[loops_index][wire_index][edge_index]
                    .absolute_back()
                    .clone();
                self.change_vertex(&old_vertex, v, emap);
            }
            ParameterKind::Inner(t) => {
                let edge = self[loops_index][wire_index][edge_index].absolute_clone();
                let point = edge.curve().subs(t);
                v.set_point(point);
                let edge_id = edge.id();
                let (edge0, edge1) = edge.cut_with_parameter(v, t)?;
                let new_wire: Wire<_, _> = vec![edge0, edge1].into();
                self.swap_edge_into_wire(edge_id, &new_wire);
            }
        }
        Some((wire_index, edge_index, kind))
    }

    fn add_geom_vertex<S>(
        &mut self,
        (loops_index, wire_index, edge_index): (usize, usize, usize),
        v: &Vertex<Point3>,
        kind: ParameterKind,
        another_surface: &S,
        emap: &mut HashMap<EdgeId<C>, Edge<Point3, C>>,
    ) -> Option<()>
    where
        C: Cut<Point = Point3, Vector = Vector3> + SearchNearestParameter<D1, Point = Point3>,
        S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
    {
        match kind {
            ParameterKind::Front => {
                let old_vertex = self[loops_index][wire_index][edge_index]
                    .absolute_front()
                    .clone();
                v.set_point(old_vertex.point());
                self.change_vertex(&old_vertex, v, emap);
            }
            ParameterKind::Back => {
                let old_vertex = self[loops_index][wire_index][edge_index]
                    .absolute_back()
                    .clone();
                v.set_point(old_vertex.point());
                self.change_vertex(&old_vertex, v, emap);
            }
            ParameterKind::Inner(_) => {
                let edge = self[loops_index][wire_index][edge_index].absolute_clone();
                let curve = edge.curve();
                let (t0, t1) = curve.range_tuple();
                let (_, t, _) =
                    curve_surface_projection(&curve, None, another_surface, None, v.point(), 100)?;
                if t < t0 + TOLERANCE {
                    let old_vertex = edge.absolute_front().clone();
                    v.set_point(old_vertex.point());
                    self.change_vertex(&old_vertex, v, emap);
                    return Some(());
                }
                if t1 - TOLERANCE < t {
                    let old_vertex = edge.absolute_back().clone();
                    v.set_point(old_vertex.point());
                    self.change_vertex(&old_vertex, v, emap);
                    return Some(());
                }
                v.set_point(curve.subs(t));
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
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
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
        let denominator = l.dot(n);
        if denominator.near(&0.0) {
            return Some((pt0.midpoint(pt1), t, Point2::new(u, v)));
        }
        let t0 = (pt1 - pt0).dot(n) / denominator;
        if !t0.is_finite() {
            return Some((pt0.midpoint(pt1), t, Point2::new(u, v)));
        }
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

fn create_independent_loop<P, C, D>(mut poly_curve0: C) -> Wire<P, D>
where
    C: Cut<Point = P>,
    D: From<C>, {
    let (t0, t1) = poly_curve0.range_tuple();
    let t = (t0 + t1) / 2.0;
    let poly_curve1 = poly_curve0.cut(t);
    let v0 = Vertex::new(poly_curve0.front());
    let v1 = Vertex::new(poly_curve1.front());
    let edge0 = Edge::new(&v0, &v1, poly_curve0.into());
    let edge1 = Edge::new(&v1, &v0, poly_curve1.into());
    wire![edge0, edge1]
}

#[allow(dead_code)]
pub struct LoopsStoreQuadruple<C> {
    pub geom_loops_store0: LoopsStore<Point3, C>,
    pub poly_loops_store0: LoopsStore<Point3, PolylineCurve>,
    pub geom_loops_store1: LoopsStore<Point3, C>,
    pub poly_loops_store1: LoopsStore<Point3, PolylineCurve>,
}

#[derive(Debug, Error)]
pub enum CreateLoopsStoreError {
    #[error("missing triangulated polygon for shell {shell_index} face {face_index}.")]
    MissingPolygon {
        shell_index: usize,
        face_index: usize,
    },
    #[error("failed to extract intersection curves for face pair ({face_index0}, {face_index1}).")]
    IntersectionCurvesFailed {
        face_index0: usize,
        face_index1: usize,
    },
    #[error(
        "failed to classify intersection curve {curve_index} for face pair ({face_index0}, {face_index1})."
    )]
    CurveStatusFailed {
        face_index0: usize,
        face_index1: usize,
        curve_index: usize,
    },
    #[error(
        "failed to place `{vertex_label}` vertex for curve {curve_index} on shell {shell_index} face {face_index} against face {another_face_index}."
    )]
    GeomVertexProjectionFailed {
        shell_index: usize,
        face_index: usize,
        another_face_index: usize,
        curve_index: usize,
        vertex_label: &'static str,
    },
}

#[derive(Clone)]
struct DeferredSegment<C> {
    face_index: usize,
    status: ShapesOpStatus,
    pv0: Vertex<Point3>,
    pv1: Vertex<Point3>,
    gv0: Vertex<Point3>,
    gv1: Vertex<Point3>,
    polyline: PolylineCurve,
    geom_curve: C,
    pv0_resolved: bool,
    pv1_resolved: bool,
}

fn deferred_key(face_index: usize, point: Point3, key_tol: f64) -> (usize, [i64; 3]) {
    let x = (point[0] / key_tol).round() as i64;
    let y = (point[1] / key_tol).round() as i64;
    let z = (point[2] / key_tol).round() as i64;
    (face_index, [x, y, z])
}

fn flush_deferred_segments<C: Clone>(
    poly_loops_store: &mut LoopsStore<Point3, PolylineCurve>,
    geom_loops_store: &mut LoopsStore<Point3, C>,
    deferred: &mut Vec<DeferredSegment<C>>,
    snap_tol: f64,
    key_tol: f64,
) {
    let debug_missing = std::env::var("MT_BOOL_DEBUG_ENDPOINTS").is_ok();
    if deferred.is_empty() {
        return;
    }
    let mut unresolved_counts: HashMap<(usize, [i64; 3]), usize> = HashMap::default();
    deferred.iter().for_each(|segment| {
        if !segment.pv0_resolved {
            *unresolved_counts
                .entry(deferred_key(
                    segment.face_index,
                    segment.pv0.point(),
                    key_tol,
                ))
                .or_insert(0) += 1;
        }
        if !segment.pv1_resolved {
            *unresolved_counts
                .entry(deferred_key(
                    segment.face_index,
                    segment.pv1.point(),
                    key_tol,
                ))
                .or_insert(0) += 1;
        }
    });
    let mut pending = std::mem::take(deferred);
    loop {
        let mut progress = false;
        let mut next_pending = Vec::new();
        std::mem::take(&mut pending)
            .into_iter()
            .for_each(|mut segment| {
                let face_index = segment.face_index;
                if let Some(v) =
                    poly_loops_store.find_near_vertex(face_index, segment.pv0.point(), snap_tol)
                {
                    segment.pv0 = v;
                }
                if let Some(v) =
                    poly_loops_store.find_near_vertex(face_index, segment.pv1.point(), snap_tol)
                {
                    segment.pv1 = v;
                }
                if let Some(v) =
                    geom_loops_store.find_near_vertex(face_index, segment.gv0.point(), snap_tol)
                {
                    segment.gv0 = v;
                }
                if let Some(v) =
                    geom_loops_store.find_near_vertex(face_index, segment.gv1.point(), snap_tol)
                {
                    segment.gv1 = v;
                }
                let pv0_known = segment.pv0_resolved
                    || poly_loops_store
                        .find_near_vertex(face_index, segment.pv0.point(), snap_tol)
                        .is_some();
                let pv1_known = segment.pv1_resolved
                    || poly_loops_store
                        .find_near_vertex(face_index, segment.pv1.point(), snap_tol)
                        .is_some();
                let pv0_key = deferred_key(face_index, segment.pv0.point(), key_tol);
                let pv1_key = deferred_key(face_index, segment.pv1.point(), key_tol);
                let pv0_ok = pv0_known || unresolved_counts.get(&pv0_key).copied().unwrap_or(0) > 1;
                let pv1_ok = pv1_known || unresolved_counts.get(&pv1_key).copied().unwrap_or(0) > 1;
                if !pv0_ok || !pv1_ok {
                    next_pending.push(segment);
                    return;
                }
                if poly_loops_store.has_edge_between_points(
                    face_index,
                    segment.pv0.point(),
                    segment.pv1.point(),
                    snap_tol,
                ) {
                    if !segment.pv0_resolved {
                        unresolved_counts
                            .entry(pv0_key)
                            .and_modify(|count| *count = count.saturating_sub(1));
                    }
                    if !segment.pv1_resolved {
                        unresolved_counts
                            .entry(pv1_key)
                            .and_modify(|count| *count = count.saturating_sub(1));
                    }
                    return;
                }
                let pedge = Edge::new(&segment.pv0, &segment.pv1, segment.polyline);
                let gedge = Edge::new(&segment.gv0, &segment.gv1, segment.geom_curve);
                poly_loops_store[face_index].add_edge(pedge, segment.status);
                geom_loops_store[face_index].add_edge(gedge, segment.status);
                if !segment.pv0_resolved {
                    unresolved_counts
                        .entry(pv0_key)
                        .and_modify(|count| *count = count.saturating_sub(1));
                }
                if !segment.pv1_resolved {
                    unresolved_counts
                        .entry(pv1_key)
                        .and_modify(|count| *count = count.saturating_sub(1));
                }
                progress = true;
            });
        if !progress {
            break;
        }
        if next_pending.is_empty() {
            break;
        }
        pending = next_pending;
    }
    if debug_missing && !pending.is_empty() {
        pending.iter().for_each(|segment| {
            eprintln!(
                "debug deferred remaining face={} status={:?} p0={:?} p1={:?} resolved=({}, {})",
                segment.face_index,
                segment.status,
                segment.pv0.point(),
                segment.pv1.point(),
                segment.pv0_resolved,
                segment.pv1_resolved,
            );
        });
    }
    *deferred = pending;
}

pub fn create_loops_stores_with_tolerance<C, S>(
    geom_shell0: &Shell<Point3, C, S>,
    poly_shell0: &Shell<Point3, PolylineCurve, Option<PolygonMesh>>,
    geom_shell1: &Shell<Point3, C, S>,
    poly_shell1: &Shell<Point3, PolylineCurve, Option<PolygonMesh>>,
    snap_tol: f64,
) -> std::result::Result<LoopsStoreQuadruple<C>, CreateLoopsStoreError>
where
    C: SearchNearestParameter<D1, Point = Point3>
        + SearchParameter<D1, Point = Point3>
        + Cut<Point = Point3, Vector = Vector3>
        + From<IntersectionCurve<PolylineCurve, S, S>>,
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
{
    let snap_tol = f64::max(snap_tol, 10.0 * TOLERANCE);
    let debug_missing = std::env::var("MT_BOOL_DEBUG_ENDPOINTS").is_ok();
    let vertex_merge_tol = 100.0 * TOLERANCE;
    let to_vertex_key = |face_index: usize, point: Point3| {
        let x = (point[0] / vertex_merge_tol).round() as i64;
        let y = (point[1] / vertex_merge_tol).round() as i64;
        let z = (point[2] / vertex_merge_tol).round() as i64;
        (face_index, [x, y, z])
    };
    let mut geom_loops_store0: LoopsStore<_, _> = geom_shell0.face_iter().collect();
    let mut poly_loops_store0: LoopsStore<_, _> = poly_shell0.face_iter().collect();
    let mut geom_loops_store1: LoopsStore<_, _> = geom_shell1.face_iter().collect();
    let mut poly_loops_store1: LoopsStore<_, _> = poly_shell1.face_iter().collect();
    let mut poly_vertex_map0: HashMap<(usize, [i64; 3]), Vertex<Point3>> = HashMap::default();
    let mut poly_vertex_map1: HashMap<(usize, [i64; 3]), Vertex<Point3>> = HashMap::default();
    let mut geom_vertex_map0: HashMap<(usize, [i64; 3]), Vertex<Point3>> = HashMap::default();
    let mut geom_vertex_map1: HashMap<(usize, [i64; 3]), Vertex<Point3>> = HashMap::default();
    let mut deferred0 = Vec::<DeferredSegment<C>>::new();
    let mut deferred1 = Vec::<DeferredSegment<C>>::new();
    let store0_len = geom_loops_store0.len();
    let store1_len = geom_loops_store1.len();
    let max_passes = std::env::var("MT_BOOL_PASSES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|p| p.max(1))
        .unwrap_or(1);
    for pass in 0..max_passes {
        let mut pass_added = 0usize;
        let mut pass_unresolved = 0usize;
        (0..store0_len)
            .flat_map(move |i| (0..store1_len).map(move |j| (i, j)))
            .try_for_each(
                |(face_index0, face_index1)| -> std::result::Result<(), CreateLoopsStoreError> {
                    let ori0 = geom_shell0[face_index0].orientation();
                    let ori1 = geom_shell1[face_index1].orientation();
                    let surface0 = geom_shell0[face_index0].surface();
                    let surface1 = geom_shell1[face_index1].surface();
                    let polygon0 = poly_shell0[face_index0]
                        .surface()
                        .ok_or(CreateLoopsStoreError::MissingPolygon {
                            shell_index: 0,
                            face_index: face_index0,
                        })?;
                    let polygon1 = poly_shell1[face_index1]
                        .surface()
                        .ok_or(CreateLoopsStoreError::MissingPolygon {
                            shell_index: 1,
                            face_index: face_index1,
                        })?;
                    intersection_curve::intersection_curves(
                        surface0.clone(),
                        &polygon0,
                        surface1.clone(),
                        &polygon1,
                    )
                    .ok_or(CreateLoopsStoreError::IntersectionCurvesFailed {
                        face_index0,
                        face_index1,
                    })?
                    .into_iter()
                    .enumerate()
                    .try_for_each(
                        |(curve_index, (polyline, intersection_curve))| -> std::result::Result<(), CreateLoopsStoreError> {
                            let mut intersection_curve = intersection_curve.into();
                            let status = ShapesOpStatus::from_is_curve(&intersection_curve).ok_or(
                                CreateLoopsStoreError::CurveStatusFailed {
                                    face_index0,
                                    face_index1,
                                    curve_index,
                                },
                            )?;
                            let map = std::env::var("MT_BOOL_STATUS_MAP")
                                .ok()
                                .and_then(|s| s.parse::<u16>().ok());
                            let pick = |bit: u16, default: ShapesOpStatus| {
                                if let Some(map) = map {
                                    if (map >> bit) & 1 == 0 {
                                        status
                                    } else {
                                        status.not()
                                    }
                                } else {
                                    default
                                }
                            };
                            let (status0, status1) = match (ori0, ori1) {
                                (true, true) => (pick(0, status), pick(1, status.not())),
                                (true, false) => (pick(2, status.not()), pick(3, status.not())),
                                (false, true) => (pick(4, status), pick(5, status)),
                                (false, false) => (pick(6, status.not()), pick(7, status)),
                            };
                            let is_closed = polyline.front().near(&polyline.back());
                            if is_closed {
                                if pass == 0 {
                                    let poly_wire = create_independent_loop(polyline);
                                    poly_loops_store0[face_index0]
                                        .add_independent_loop(BoundaryWire::new(poly_wire.clone(), status0));
                                    poly_loops_store1[face_index1]
                                        .add_independent_loop(BoundaryWire::new(poly_wire, status1));
                                    let geom_wire = create_independent_loop(intersection_curve);
                                    geom_loops_store0[face_index0]
                                        .add_independent_loop(BoundaryWire::new(geom_wire.clone(), status0));
                                    geom_loops_store1[face_index1]
                                        .add_independent_loop(BoundaryWire::new(geom_wire, status1));
                                }
                                return Ok(());
                            }
                            let mut pv0 = Vertex::new(polyline.front());
                            let mut pv1 = Vertex::new(polyline.back());
                            let mut gv0 = Vertex::new(polyline.front());
                            let mut gv1 = Vertex::new(polyline.back());
                            if let Some(vertex) = poly_vertex_map0
                                .get(&to_vertex_key(face_index0, pv0.point()))
                                .cloned()
                            {
                                pv0 = vertex;
                            }
                            if let Some(vertex) = poly_vertex_map0
                                .get(&to_vertex_key(face_index0, pv1.point()))
                                .cloned()
                            {
                                pv1 = vertex;
                            }
                            if let Some(vertex) = poly_vertex_map1
                                .get(&to_vertex_key(face_index1, pv0.point()))
                                .cloned()
                            {
                                pv0 = vertex;
                            }
                            if let Some(vertex) = poly_vertex_map1
                                .get(&to_vertex_key(face_index1, pv1.point()))
                                .cloned()
                            {
                                pv1 = vertex;
                            }
                            if let Some(vertex) = geom_vertex_map0
                                .get(&to_vertex_key(face_index0, gv0.point()))
                                .cloned()
                            {
                                gv0 = vertex;
                            }
                            if let Some(vertex) = geom_vertex_map0
                                .get(&to_vertex_key(face_index0, gv1.point()))
                                .cloned()
                            {
                                gv1 = vertex;
                            }
                            if let Some(vertex) = geom_vertex_map1
                                .get(&to_vertex_key(face_index1, gv0.point()))
                                .cloned()
                            {
                                gv0 = vertex;
                            }
                            if let Some(vertex) = geom_vertex_map1
                                .get(&to_vertex_key(face_index1, gv1.point()))
                                .cloned()
                            {
                                gv1 = vertex;
                            }
                            if debug_missing {
                                let target0 = Point3::new(0.28867513459481287, 0.16666666666666669, 0.0);
                                let target1 = Point3::new(0.0, 0.16666666666666669, 0.28867513459481287);
                                let is_target = |point: Point3| point.near(&target0) || point.near(&target1);
                                if is_target(pv0.point()) || is_target(pv1.point()) {
                                    eprintln!(
                                        "debug target curve pass={pass} face_pair=({face_index0},{face_index1}) curve={curve_index} ori0={ori0} ori1={ori1} pv0={:?} pv1={:?}",
                                        pv0.point(),
                                        pv1.point()
                                    );
                                }
                            }
                            let mut pemap0 = HashMap::default();
                            let mut pemap1 = HashMap::default();
                            let mut gemap0 = HashMap::default();
                            let mut gemap1 = HashMap::default();
                            let idx00 = poly_loops_store0.add_polygon_vertex_with_tolerance(
                                face_index0,
                                &pv0,
                                snap_tol,
                                &mut pemap0,
                            );
                            if let Some((wire_index, edge_index, kind)) = idx00 {
                                geom_loops_store0.add_geom_vertex(
                                    (face_index0, wire_index, edge_index),
                                    &gv0,
                                    kind,
                                    &surface1,
                                    &mut gemap0,
                                )
                                .ok_or(CreateLoopsStoreError::GeomVertexProjectionFailed {
                                    shell_index: 0,
                                    face_index: face_index0,
                                    another_face_index: face_index1,
                                    curve_index,
                                    vertex_label: "front",
                                })?;
                                let polyline = intersection_curve.leader_mut();
                                // SAFETY: a polyline curve always has at least one point.
                                *polyline.first_mut().unwrap() = gv0.point();
                            } else {
                                let reused_poly =
                                    poly_loops_store0.find_near_vertex(face_index0, pv0.point(), snap_tol);
                                if let Some(vertex) = &reused_poly {
                                    pv0 = vertex.clone();
                                }
                                let reused_geom =
                                    geom_loops_store0.find_near_vertex(face_index0, gv0.point(), snap_tol);
                                if let Some(vertex) = &reused_geom {
                                    gv0 = vertex.clone();
                                }
                                if debug_missing && reused_poly.is_none() && reused_geom.is_none() {
                                    let dist2 = poly_loops_store0[face_index0].nearest_distance2(pv0.point());
                                    eprintln!(
                                        "debug unresolved idx00 pass={pass} face_pair=({face_index0},{face_index1}) curve={curve_index} pt={:?} nearest_dist2={dist2:?}",
                                        pv0.point(),
                                    );
                                }
                            }
                            let idx01 = poly_loops_store0.add_polygon_vertex_with_tolerance(
                                face_index0,
                                &pv1,
                                snap_tol,
                                &mut pemap1,
                            );
                            if let Some((wire_index, edge_index, kind)) = idx01 {
                                geom_loops_store0.add_geom_vertex(
                                    (face_index0, wire_index, edge_index),
                                    &gv1,
                                    kind,
                                    &surface1,
                                    &mut gemap1,
                                )
                                .ok_or(CreateLoopsStoreError::GeomVertexProjectionFailed {
                                    shell_index: 0,
                                    face_index: face_index0,
                                    another_face_index: face_index1,
                                    curve_index,
                                    vertex_label: "back",
                                })?;
                                let polyline = intersection_curve.leader_mut();
                                // SAFETY: a polyline curve always has at least one point.
                                *polyline.last_mut().unwrap() = gv1.point();
                            } else {
                                let reused_poly =
                                    poly_loops_store0.find_near_vertex(face_index0, pv1.point(), snap_tol);
                                if let Some(vertex) = &reused_poly {
                                    pv1 = vertex.clone();
                                }
                                let reused_geom =
                                    geom_loops_store0.find_near_vertex(face_index0, gv1.point(), snap_tol);
                                if let Some(vertex) = &reused_geom {
                                    gv1 = vertex.clone();
                                }
                                if debug_missing && reused_poly.is_none() && reused_geom.is_none() {
                                    let dist2 = poly_loops_store0[face_index0].nearest_distance2(pv1.point());
                                    eprintln!(
                                        "debug unresolved idx01 pass={pass} face_pair=({face_index0},{face_index1}) curve={curve_index} pt={:?} nearest_dist2={dist2:?}",
                                        pv1.point(),
                                    );
                                }
                            }
                            let idx10 = poly_loops_store1.add_polygon_vertex_with_tolerance(
                                face_index1,
                                &pv0,
                                snap_tol,
                                &mut pemap0,
                            );
                            if let Some((wire_index, edge_index, kind)) = idx10 {
                                geom_loops_store1.add_geom_vertex(
                                    (face_index1, wire_index, edge_index),
                                    &gv0,
                                    kind,
                                    &surface0,
                                    &mut gemap0,
                                )
                                .ok_or(CreateLoopsStoreError::GeomVertexProjectionFailed {
                                    shell_index: 1,
                                    face_index: face_index1,
                                    another_face_index: face_index0,
                                    curve_index,
                                    vertex_label: "front",
                                })?;
                                let polyline = intersection_curve.leader_mut();
                                // SAFETY: a polyline curve always has at least one point.
                                *polyline.first_mut().unwrap() = gv0.point();
                            } else {
                                let reused_poly =
                                    poly_loops_store1.find_near_vertex(face_index1, pv0.point(), snap_tol);
                                if let Some(vertex) = &reused_poly {
                                    pv0 = vertex.clone();
                                }
                                let reused_geom =
                                    geom_loops_store1.find_near_vertex(face_index1, gv0.point(), snap_tol);
                                if let Some(vertex) = &reused_geom {
                                    gv0 = vertex.clone();
                                }
                                if debug_missing && reused_poly.is_none() && reused_geom.is_none() {
                                    let dist2 = poly_loops_store1[face_index1].nearest_distance2(pv0.point());
                                    eprintln!(
                                        "debug unresolved idx10 pass={pass} face_pair=({face_index0},{face_index1}) curve={curve_index} pt={:?} nearest_dist2={dist2:?}",
                                        pv0.point(),
                                    );
                                }
                            }
                            let idx11 = poly_loops_store1.add_polygon_vertex_with_tolerance(
                                face_index1,
                                &pv1,
                                snap_tol,
                                &mut pemap1,
                            );
                            if let Some((wire_index, edge_index, kind)) = idx11 {
                                geom_loops_store1.add_geom_vertex(
                                    (face_index1, wire_index, edge_index),
                                    &gv1,
                                    kind,
                                    &surface0,
                                    &mut gemap1,
                                )
                                .ok_or(CreateLoopsStoreError::GeomVertexProjectionFailed {
                                    shell_index: 1,
                                    face_index: face_index1,
                                    another_face_index: face_index0,
                                    curve_index,
                                    vertex_label: "back",
                                })?;
                                let polyline = intersection_curve.leader_mut();
                                // SAFETY: a polyline curve always has at least one point.
                                *polyline.last_mut().unwrap() = gv1.point();
                            } else {
                                let reused_poly =
                                    poly_loops_store1.find_near_vertex(face_index1, pv1.point(), snap_tol);
                                if let Some(vertex) = &reused_poly {
                                    pv1 = vertex.clone();
                                }
                                let reused_geom =
                                    geom_loops_store1.find_near_vertex(face_index1, gv1.point(), snap_tol);
                                if let Some(vertex) = &reused_geom {
                                    gv1 = vertex.clone();
                                }
                                if debug_missing && reused_poly.is_none() && reused_geom.is_none() {
                                    let dist2 = poly_loops_store1[face_index1].nearest_distance2(pv1.point());
                                    eprintln!(
                                        "debug unresolved idx11 pass={pass} face_pair=({face_index0},{face_index1}) curve={curve_index} pt={:?} nearest_dist2={dist2:?}",
                                        pv1.point(),
                                    );
                                }
                            }
                            if pv0.point().near(&pv1.point()) || gv0.point().near(&gv1.point()) {
                                return Ok(());
                            }
                            let all_indices_placed =
                                idx00.is_some() && idx01.is_some() && idx10.is_some() && idx11.is_some();
                            let duplicated0 = poly_loops_store0.has_edge_between_points(
                                face_index0,
                                pv0.point(),
                                pv1.point(),
                                snap_tol,
                            );
                            let duplicated1 = poly_loops_store1.has_edge_between_points(
                                face_index1,
                                pv0.point(),
                                pv1.point(),
                                snap_tol,
                            );
                            if debug_missing
                                && (idx00.is_none()
                                    || idx01.is_none()
                                    || idx10.is_none()
                                    || idx11.is_none()
                                    || duplicated0
                                    || duplicated1)
                                {
                                    eprintln!(
                                        "debug curve pass={pass} face_pair=({face_index0},{face_index1}) curve={curve_index} idx00={} idx01={} idx10={} idx11={} dup0={} dup1={} status0={status0:?} status1={status1:?} p0={:?} p1={:?}",
                                        idx00.is_some(),
                                        idx01.is_some(),
                                        idx10.is_some(),
                                        idx11.is_some(),
                                        duplicated0,
                                        duplicated1,
                                        pv0.point(),
                                        pv1.point(),
                                    );
                                }
                            let policy = std::env::var("MT_BOOL_POLICY")
                                .ok()
                                .and_then(|s| s.parse::<u8>().ok())
                                .unwrap_or(0);
                            let require_all = (policy & 1) != 0;
                            let skip_either_duplicate = (policy & 2) != 0;
                            let allow_partial = (policy & 4) != 0;
                            if require_all && !all_indices_placed {
                                pass_unresolved += 1;
                                return Ok(());
                            }
                            if skip_either_duplicate && (duplicated0 || duplicated1) {
                                return Ok(());
                            }
                            if duplicated0 && duplicated1 {
                                return Ok(());
                            }
                            let can_add0 = !duplicated0
                                && if require_all || !allow_partial {
                                    all_indices_placed
                                } else {
                                    idx00.is_some() || idx01.is_some()
                                };
                            let can_add1 = !duplicated1
                                && if require_all || !allow_partial {
                                    all_indices_placed
                                } else {
                                    idx10.is_some() || idx11.is_some()
                                };
                            if !can_add0 && !can_add1 {
                                if !duplicated0 && (idx00.is_some() || idx01.is_some()) {
                                    deferred0.push(DeferredSegment {
                                        face_index: face_index0,
                                        status: status0,
                                        pv0: pv0.clone(),
                                        pv1: pv1.clone(),
                                        gv0: gv0.clone(),
                                        gv1: gv1.clone(),
                                        polyline: polyline.clone(),
                                        geom_curve: intersection_curve.clone().into(),
                                        pv0_resolved: idx00.is_some(),
                                        pv1_resolved: idx01.is_some(),
                                    });
                                }
                                if !duplicated1 && (idx10.is_some() || idx11.is_some()) {
                                    deferred1.push(DeferredSegment {
                                        face_index: face_index1,
                                        status: status1,
                                        pv0: pv0.clone(),
                                        pv1: pv1.clone(),
                                        gv0: gv0.clone(),
                                        gv1: gv1.clone(),
                                        polyline: polyline.clone(),
                                        geom_curve: intersection_curve.clone().into(),
                                        pv0_resolved: idx10.is_some(),
                                        pv1_resolved: idx11.is_some(),
                                    });
                                }
                                if !all_indices_placed {
                                    pass_unresolved += 1;
                                }
                                return Ok(());
                            }
                            if can_add0 {
                                let pedge = Edge::new(&pv0, &pv1, polyline.clone());
                                let gedge = Edge::new(&gv0, &gv1, intersection_curve.clone().into());
                                poly_loops_store0[face_index0].add_edge(pedge.clone(), status0);
                                geom_loops_store0[face_index0].add_edge(gedge.clone(), status0);
                                poly_vertex_map0
                                    .insert(to_vertex_key(face_index0, pv0.point()), pv0.clone());
                                poly_vertex_map0
                                    .insert(to_vertex_key(face_index0, pv1.point()), pv1.clone());
                                geom_vertex_map0
                                    .insert(to_vertex_key(face_index0, gv0.point()), gv0.clone());
                                geom_vertex_map0
                                    .insert(to_vertex_key(face_index0, gv1.point()), gv1.clone());
                                pass_added += 1;
                            } else if !duplicated0 && (idx00.is_some() || idx01.is_some()) {
                                deferred0.push(DeferredSegment {
                                    face_index: face_index0,
                                    status: status0,
                                    pv0: pv0.clone(),
                                    pv1: pv1.clone(),
                                    gv0: gv0.clone(),
                                    gv1: gv1.clone(),
                                    polyline: polyline.clone(),
                                    geom_curve: intersection_curve.clone().into(),
                                    pv0_resolved: idx00.is_some(),
                                    pv1_resolved: idx01.is_some(),
                                });
                            }
                            if can_add1 {
                                let pedge = Edge::new(&pv0, &pv1, polyline.clone());
                                let gedge = Edge::new(&gv0, &gv1, intersection_curve.clone().into());
                                poly_loops_store1[face_index1].add_edge(pedge, status1);
                                geom_loops_store1[face_index1].add_edge(gedge, status1);
                                poly_vertex_map1
                                    .insert(to_vertex_key(face_index1, pv0.point()), pv0.clone());
                                poly_vertex_map1
                                    .insert(to_vertex_key(face_index1, pv1.point()), pv1.clone());
                                geom_vertex_map1
                                    .insert(to_vertex_key(face_index1, gv0.point()), gv0.clone());
                                geom_vertex_map1
                                    .insert(to_vertex_key(face_index1, gv1.point()), gv1.clone());
                                pass_added += 1;
                            } else if !duplicated1 && (idx10.is_some() || idx11.is_some()) {
                                deferred1.push(DeferredSegment {
                                    face_index: face_index1,
                                    status: status1,
                                    pv0: pv0.clone(),
                                    pv1: pv1.clone(),
                                    gv0: gv0.clone(),
                                    gv1: gv1.clone(),
                                    polyline: polyline.clone(),
                                    geom_curve: intersection_curve.into(),
                                    pv0_resolved: idx10.is_some(),
                                    pv1_resolved: idx11.is_some(),
                                });
                            }
                            Ok(())
                        },
                    )
                },
            )?;
        if pass_unresolved == 0 || pass_added == 0 {
            break;
        }
    }
    flush_deferred_segments(
        &mut poly_loops_store0,
        &mut geom_loops_store0,
        &mut deferred0,
        snap_tol,
        vertex_merge_tol,
    );
    flush_deferred_segments(
        &mut poly_loops_store1,
        &mut geom_loops_store1,
        &mut deferred1,
        snap_tol,
        vertex_merge_tol,
    );
    Ok(LoopsStoreQuadruple {
        geom_loops_store0,
        poly_loops_store0,
        geom_loops_store1,
        poly_loops_store1,
    })
}

#[cfg(test)]
pub fn create_loops_stores<C, S>(
    geom_shell0: &Shell<Point3, C, S>,
    poly_shell0: &Shell<Point3, PolylineCurve, Option<PolygonMesh>>,
    geom_shell1: &Shell<Point3, C, S>,
    poly_shell1: &Shell<Point3, PolylineCurve, Option<PolygonMesh>>,
) -> std::result::Result<LoopsStoreQuadruple<C>, CreateLoopsStoreError>
where
    C: SearchNearestParameter<D1, Point = Point3>
        + SearchParameter<D1, Point = Point3>
        + Cut<Point = Point3, Vector = Vector3>
        + From<IntersectionCurve<PolylineCurve, S, S>>,
    S: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
{
    create_loops_stores_with_tolerance(
        geom_shell0,
        poly_shell0,
        geom_shell1,
        poly_shell1,
        10.0 * TOLERANCE,
    )
}

#[cfg(test)]
mod tests;
