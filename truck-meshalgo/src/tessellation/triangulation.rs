#![allow(clippy::many_single_char_names)]

use super::*;
use crate::filters::{NormalFilters, StructuringFilter};
use crate::Point2;
use array_macro::array;
use handles::FixedVertexHandle;
use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

type SPoint2 = spade::Point2<f64>;
type Cdt = ConstrainedDelaunayTriangulation<SPoint2>;
type MeshedShell = Shell<Point3, PolylineCurve, Option<PolygonMesh>>;
type MeshedCShell = CompressedShell<Point3, PolylineCurve, Option<PolygonMesh>>;

pub(super) trait SP<S>:
    Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> + Parallelizable {
}
impl<S, F> SP<S> for F where F: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> + Parallelizable {}

pub(super) fn search_parameter_sp<S: MeshableSurface>(trials: usize) -> impl SP<S> {
    move |surface: &S, point: Point3, hint: Option<(f64, f64)>| {
        surface
            .search_parameter(point, hint, trials)
            .or_else(|| surface.search_parameter(point, None, trials))
    }
}

pub(super) fn search_nearest_parameter_sp<S: RobustMeshableSurface>(trials: usize) -> impl SP<S> {
    move |surface: &S, point: Point3, hint: Option<(f64, f64)>| {
        surface
            .search_parameter(point, hint, trials)
            .or_else(|| surface.search_parameter(point, None, trials))
            .or_else(|| surface.search_nearest_parameter(point, hint, trials))
            .or_else(|| surface.search_nearest_parameter(point, None, trials))
    }
}

/// Compatibility wrapper: searches parameter with 100 trials.
#[cfg(test)]
pub(super) fn by_search_parameter<S: MeshableSurface>(
    surface: &S,
    point: Point3,
    hint: Option<(f64, f64)>,
) -> Option<(f64, f64)> {
    search_parameter_sp::<S>(100)(surface, point, hint)
}

/// Tessellates faces
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn shell_tessellation<'a, C, S>(
    shell: &Shell<Point3, C, S>,
    tolerance: f64,
    sp: impl SP<S>,
    quad_config: QuadOptions,
) -> MeshedShell
where
    C: PolylineableCurve + 'a,
    S: PreMeshableSurface + 'a,
{
    let vmap: HashMap<_, _> = shell
        .vertex_par_iter()
        .map(|v| (v.id(), v.mapped(Point3::clone)))
        .collect();
    let eset: HashMap<_, _> = shell.edge_par_iter().map(move |e| (e.id(), e)).collect();
    let edge_map: HashMap<_, _> = eset
        .into_par_iter()
        .map(move |(id, edge)| {
            let v0 = vmap.get(&edge.absolute_front().id()).unwrap();
            let v1 = vmap.get(&edge.absolute_back().id()).unwrap();
            let curve = edge.curve();
            let poly = PolylineCurve::from_curve(&curve, curve.range_tuple(), tolerance);
            (id, Edge::debug_new(v0, v1, poly))
        })
        .collect();
    let create_edge = |edge: &Edge<Point3, C>| -> Edge<_, _> {
        let new_edge = edge_map.get(&edge.id()).unwrap();
        match edge.orientation() {
            true => new_edge.clone(),
            false => new_edge.inverse(),
        }
    };
    let create_boundary =
        |wire: &Wire<Point3, C>| -> Wire<_, _> { wire.edge_iter().map(create_edge).collect() };
    let create_face = move |face: &Face<Point3, C, S>| -> Face<_, _, _> {
        let wires: Vec<_> = face
            .absolute_boundaries()
            .iter()
            .map(create_boundary)
            .collect();
        shell_create_polygon(
            &face.surface(),
            wires,
            face.orientation(),
            tolerance,
            &sp,
            quad_config,
        )
    };
    shell.face_par_iter().map(create_face).collect()
}

/// Tessellates faces
#[cfg(any(target_arch = "wasm32", test))]
pub(super) fn shell_tessellation_single_thread<'a, C, S>(
    shell: &'a Shell<Point3, C, S>,
    tolerance: f64,
    sp: impl SP<S>,
    quad_config: QuadOptions,
) -> MeshedShell
where
    C: PolylineableCurve + 'a,
    S: PreMeshableSurface + 'a,
{
    use truck_base::entry_map::FxEntryMap as EntryMap;
    use truck_topology::Vertex as TVertex;
    let mut vmap = EntryMap::new(
        move |v: &TVertex<Point3>| v.id(),
        move |v| v.mapped(Point3::clone),
    );
    let mut edge_map = EntryMap::new(
        move |edge: &'a Edge<Point3, C>| edge.id(),
        move |edge| {
            let vf = edge.absolute_front();
            let v0 = vmap.entry_or_insert(vf).clone();
            let vb = edge.absolute_back();
            let v1 = vmap.entry_or_insert(vb).clone();
            let curve = edge.curve();
            let poly = PolylineCurve::from_curve(&curve, curve.range_tuple(), tolerance);
            Edge::debug_new(&v0, &v1, poly)
        },
    );
    let mut create_edge = move |edge: &'a Edge<Point3, C>| -> Edge<_, _> {
        let new_edge = edge_map.entry_or_insert(edge);
        match edge.orientation() {
            true => new_edge.clone(),
            false => new_edge.inverse(),
        }
    };
    let mut create_boundary = move |wire: &'a Wire<Point3, C>| -> Wire<_, _> {
        wire.edge_iter().map(&mut create_edge).collect()
    };
    let create_face = move |face: &'a Face<Point3, C, S>| -> Face<_, _, _> {
        let wires: Vec<_> = face
            .absolute_boundaries()
            .iter()
            .map(&mut create_boundary)
            .collect();
        shell_create_polygon(
            &face.surface(),
            wires,
            face.orientation(),
            tolerance,
            &sp,
            quad_config,
        )
    };
    shell.face_iter().map(create_face).collect()
}

/// Tessellates faces
pub(super) fn cshell_tessellation<'a, C, S>(
    shell: &CompressedShell<Point3, C, S>,
    tolerance: f64,
    sp: impl SP<S>,
    quad_config: QuadOptions,
) -> MeshedCShell
where
    C: PolylineableCurve + 'a,
    S: PreMeshableSurface + 'a,
{
    let vertices = shell.vertices.clone();
    let tessellate_edge = |edge: &CompressedEdge<C>| {
        let curve = &edge.curve;
        CompressedEdge {
            vertices: edge.vertices,
            curve: PolylineCurve::from_curve(curve, curve.range_tuple(), tolerance),
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let edges: Vec<_> = shell.edges.par_iter().map(tessellate_edge).collect();
    #[cfg(target_arch = "wasm32")]
    let edges: Vec<_> = shell.edges.iter().map(tessellate_edge).collect();
    let tessellate_face = |face: &CompressedFace<S>| {
        let boundaries = face.boundaries.clone();
        let surface = &face.surface;

        // Fast path: untrimmed face with bounded surface domain.
        let is_untrimmed = boundaries.iter().all(|wire| wire.is_empty());
        if is_untrimmed {
            if let (Some(urange), Some(vrange)) = surface.try_range_tuple() {
                let polygon =
                    untrimmed_tessellation(surface, (urange, vrange), tolerance, quad_config.mode);
                return CompressedFace {
                    boundaries,
                    orientation: face.orientation,
                    surface: Some(polygon),
                };
            }
        }

        let create_edge = |edge_idx: &CompressedEdgeIndex| match edge_idx.orientation {
            true => Some(edges.get(edge_idx.index)?.curve.clone()),
            false => Some(edges.get(edge_idx.index)?.curve.inverse()),
        };
        let create_boundary = |wire: &Vec<CompressedEdgeIndex>| {
            let wire_iter = wire.iter().filter_map(create_edge);
            PolyBoundaryPiece::try_new(surface, wire_iter, &sp)
        };
        let preboundary: Option<Vec<_>> = boundaries.iter().map(create_boundary).collect();
        let polygon: Option<PolygonMesh> = preboundary.map(|preboundary| {
            let boundary = PolyBoundary::new(preboundary, &surface, tolerance);
            trimming_tessellation(&surface, &boundary, tolerance, quad_config)
        });
        CompressedFace {
            boundaries,
            orientation: face.orientation,
            surface: polygon,
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let faces = shell.faces.par_iter().map(tessellate_face).collect();
    #[cfg(target_arch = "wasm32")]
    let faces = shell.faces.iter().map(tessellate_face).collect();
    MeshedCShell {
        vertices,
        edges,
        faces,
    }
}

fn shell_create_polygon<S: PreMeshableSurface>(
    surface: &S,
    wires: Vec<Wire<Point3, PolylineCurve>>,
    orientation: bool,
    tolerance: f64,
    sp: impl SP<S>,
    quad_config: QuadOptions,
) -> Face<Point3, PolylineCurve, Option<PolygonMesh>> {
    // Fast path: untrimmed face with bounded surface domain.
    let is_untrimmed = wires.iter().all(|w| w.is_empty());
    let polygon = if is_untrimmed {
        if let (Some(urange), Some(vrange)) = surface.try_range_tuple() {
            Some(untrimmed_tessellation(
                surface,
                (urange, vrange),
                tolerance,
                quad_config.mode,
            ))
        } else {
            None
        }
    } else {
        let preboundary = wires
            .iter()
            .map(|wire: &Wire<_, _>| {
                let wire_iter = wire.iter().map(Edge::oriented_curve);
                PolyBoundaryPiece::try_new(surface, wire_iter, &sp)
            })
            .collect::<Option<Vec<_>>>();
        preboundary.map(|preboundary| {
            let boundary = PolyBoundary::new(preboundary, &surface, tolerance);
            trimming_tessellation(surface, &boundary, tolerance, quad_config)
        })
    };
    let mut new_face = Face::debug_new(wires, polygon);
    if !orientation {
        new_face.invert();
    }
    new_face
}

#[derive(Clone, Copy, Debug, derive_more::Deref, derive_more::DerefMut)]
struct SurfacePoint {
    point: Point3,
    #[deref]
    #[deref_mut]
    uv: Point2,
}

impl From<(Point2, Point3)> for SurfacePoint {
    fn from((uv, point): (Point2, Point3)) -> Self { Self { point, uv } }
}

#[derive(Debug, Default, Clone)]
struct PolyBoundaryPiece(Vec<SurfacePoint>);

impl PolyBoundaryPiece {
    fn try_new<S: PreMeshableSurface>(
        surface: &S,
        wire: impl Iterator<Item = PolylineCurve>,
        sp: impl SP<S>,
    ) -> Option<Self> {
        let (up, vp) = (surface.u_period(), surface.v_period());
        let (urange, vrange) = surface.try_range_tuple();
        let mut bdry3d: Vec<Point3> = wire
            .flat_map(|poly_edge| {
                let n = poly_edge.len() - 1;
                poly_edge.into_iter().take(n)
            })
            .collect();
        bdry3d.push(bdry3d[0]);
        let mut previous = None;
        let mut vec = bdry3d
            .into_iter()
            .flat_map(|pt| {
                let (mut u, mut v) = match sp(surface, pt, previous) {
                    Some(hint) => hint,
                    None => return vec![None],
                };
                if let (Some(up), Some((u0, _))) = (up, previous) {
                    u = get_mindiff(u, u0, up);
                }
                if let (Some(vp), Some((_, v0))) = (vp, previous) {
                    v = get_mindiff(v, v0, vp);
                }
                let res = (|| {
                    if let Some((u0, v0)) = previous {
                        if !u0.near(&u) && surface.uder(u0, v0).so_small() {
                            return vec![
                                Some((Point2::new(u, v0), pt).into()),
                                Some((Point2::new(u, v), pt).into()),
                            ];
                        } else if !v0.near(&v) && surface.vder(u0, v0).so_small() {
                            return vec![
                                Some((Point2::new(u0, v), pt).into()),
                                Some((Point2::new(u, v), pt).into()),
                            ];
                        }
                    }
                    vec![Some((Point2::new(u, v), pt).into())]
                })();
                previous = Some((u, v));
                res
            })
            .collect::<Option<Vec<SurfacePoint>>>()?;
        let grav = vec.iter().fold(Point2::origin(), |g, p| g + p.uv.to_vec()) / vec.len() as f64;
        if let (Some(up), Some((u0, _))) = (up, urange) {
            let quot = f64::floor((grav.x - u0) / up);
            vec.iter_mut().for_each(|p| p.x -= quot * up);
        }
        if let (Some(vp), Some((v0, _))) = (vp, vrange) {
            let quot = f64::floor((grav.y - v0) / vp);
            vec.iter_mut().for_each(|p| p.y -= quot * vp);
        }
        let last = *vec.last().unwrap();
        if !vec[0].near(&last) {
            let Point2 { x: u0, y: v0 } = last.uv;
            if surface.uder(u0, v0).so_small() || surface.vder(u0, v0).so_small() {
                vec.push(vec[0]);
            }
        }
        Some(Self(vec))
    }
}

fn abs_diff(previous: f64) -> impl Fn(&f64, &f64) -> std::cmp::Ordering {
    let f = move |x: &f64| f64::abs(x - previous);
    move |x: &f64, y: &f64| f(x).partial_cmp(&f(y)).unwrap()
}
fn get_mindiff(u: f64, u0: f64, up: f64) -> f64 {
    let closure = |i| u + i as f64 * up;
    (-2..=2).map(closure).min_by(abs_diff(u0)).unwrap()
}

#[derive(Debug, Clone)]
struct PolyBoundary {
    loops: Vec<Vec<SurfacePoint>>,
    /// UV-space axis-aligned bounding box for cheap rejection in `include()`.
    uv_min: Point2,
    uv_max: Point2,
}

impl Default for PolyBoundary {
    fn default() -> Self {
        Self {
            loops: Vec::new(),
            uv_min: Point2::new(f64::INFINITY, f64::INFINITY),
            uv_max: Point2::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }
}

fn normalize_range(curve: &mut Vec<SurfacePoint>, compidx: usize, (u0, u1): (f64, f64)) {
    let p = curve[0];
    let q = curve[curve.len() - 1];
    let tmp = f64::min(p[compidx], q[compidx]) + TOLERANCE;
    let del = f64::floor((tmp - u0) / (u1 - u0)) * (u1 - u0);
    curve.iter_mut().for_each(|p| p[compidx] -= del);
    let Some(i) = curve
        .iter()
        .position(|p| (curve[0][compidx] - u1) * (p[compidx] - u1) < 0.0)
    else {
        return;
    };
    let mut curve1 = curve.split_off(i + 1);
    curve1.pop();
    curve1.insert(0, curve[i]);
    match curve[0][compidx] < curve[curve.len() - 1][compidx] {
        true => curve1.iter_mut(),
        false => curve.iter_mut(),
    }
    .for_each(|p| p[compidx] -= u1 - u0);
    curve1.append(curve);
    *curve = curve1;
}

fn loop_orientation(curve: &[SurfacePoint]) -> bool {
    curve
        .iter()
        .circular_tuple_windows()
        .fold(0.0, |sum, (p, q)| sum + (q.x + p.x) * (q.y - p.y))
        > 0.0
}

type UvKey = (u64, u64);

fn uv_key(uv: Point2) -> UvKey { (uv.x.to_bits(), uv.y.to_bits()) }

fn surface_point_with_cache(
    surface: &impl PreMeshableSurface,
    uv: Point2,
    point_cache: &mut HashMap<UvKey, Point3>,
) -> SurfacePoint {
    let point = *point_cache
        .entry(uv_key(uv))
        .or_insert_with(|| surface.subs(uv.x, uv.y));
    (uv, point).into()
}

impl PolyBoundary {
    fn new(
        pieces: Vec<PolyBoundaryPiece>,
        surface: &impl PreMeshableSurface,
        tolerance: f64,
    ) -> Self {
        let (mut closed, mut open) = (Vec::new(), Vec::new());
        pieces.into_iter().for_each(|PolyBoundaryPiece(mut vec)| {
            match vec[0].uv.distance(vec[vec.len() - 1].uv) < 1.0e-3 {
                true => {
                    vec.pop();
                    closed.push(vec)
                }
                false => open.push(vec),
            }
        });
        fn connect_edges<P>(vecs: impl IntoIterator<Item = Vec<P>>) -> Vec<P> {
            let closure = |vec: Vec<P>| {
                let len = vec.len();
                vec.into_iter().take(len - 1)
            };
            vecs.into_iter().flat_map(closure).collect()
        }
        let mut point_cache = HashMap::<UvKey, Point3>::default();
        match open.len() {
            1 => {
                let mut curve = open.pop().unwrap();
                let p = curve[0];
                let q = curve[curve.len() - 1];
                if let (Some((u0, u1)), Some((v0, v1))) = surface.try_range_tuple() {
                    if p.x < q.x - TOLERANCE {
                        normalize_range(&mut curve, 0, (u0, u1));
                        let p = curve[0];
                        let q = curve[curve.len() - 1];
                        let x = surface_point_with_cache(
                            surface,
                            Point2::new(u0, v1),
                            &mut point_cache,
                        );
                        let y = surface_point_with_cache(
                            surface,
                            Point2::new(u1, v1),
                            &mut point_cache,
                        );
                        let vec0 = polyline_on_surface(surface, q, y, tolerance, &mut point_cache);
                        let vec1 = polyline_on_surface(surface, y, x, tolerance, &mut point_cache);
                        let vec2 = polyline_on_surface(surface, x, p, tolerance, &mut point_cache);
                        closed.push(connect_edges([vec0, vec1, vec2, curve]));
                    } else if q.x < p.x - TOLERANCE {
                        normalize_range(&mut curve, 0, (u0, u1));
                        let p = curve[0];
                        let q = curve[curve.len() - 1];
                        let x = surface_point_with_cache(
                            surface,
                            Point2::new(u1, v0),
                            &mut point_cache,
                        );
                        let y = surface_point_with_cache(
                            surface,
                            Point2::new(u0, v0),
                            &mut point_cache,
                        );
                        let vec0 = polyline_on_surface(surface, q, y, tolerance, &mut point_cache);
                        let vec1 = polyline_on_surface(surface, y, x, tolerance, &mut point_cache);
                        let vec2 = polyline_on_surface(surface, x, p, tolerance, &mut point_cache);
                        closed.push(connect_edges([vec0, vec1, vec2, curve]));
                    } else if p.y < q.y - TOLERANCE {
                        normalize_range(&mut curve, 1, (v0, v1));
                        let p = curve[0];
                        let q = curve[curve.len() - 1];
                        let x = surface_point_with_cache(
                            surface,
                            Point2::new(u0, v0),
                            &mut point_cache,
                        );
                        let y = surface_point_with_cache(
                            surface,
                            Point2::new(u0, v1),
                            &mut point_cache,
                        );
                        let vec0 = polyline_on_surface(surface, q, y, tolerance, &mut point_cache);
                        let vec1 = polyline_on_surface(surface, y, x, tolerance, &mut point_cache);
                        let vec2 = polyline_on_surface(surface, x, p, tolerance, &mut point_cache);
                        closed.push(connect_edges([vec0, vec1, vec2, curve]));
                    } else if q.y < p.y - TOLERANCE {
                        normalize_range(&mut curve, 1, (v0, v1));
                        let p = curve[0];
                        let q = curve[curve.len() - 1];
                        let x = surface_point_with_cache(
                            surface,
                            Point2::new(u1, v1),
                            &mut point_cache,
                        );
                        let y = surface_point_with_cache(
                            surface,
                            Point2::new(u1, v0),
                            &mut point_cache,
                        );
                        let vec0 = polyline_on_surface(surface, q, y, tolerance, &mut point_cache);
                        let vec1 = polyline_on_surface(surface, y, x, tolerance, &mut point_cache);
                        let vec2 = polyline_on_surface(surface, x, p, tolerance, &mut point_cache);
                        closed.push(connect_edges([vec0, vec1, vec2, curve]));
                    }
                }
            }
            2 => {
                let mut curve1 = open.pop().unwrap();
                let mut curve0 = open.pop().unwrap();
                fn end_pts<T: Copy>(vec: &[T]) -> (T, T) { (vec[0], vec[vec.len() - 1]) }
                let ((p0, p1), (q0, q1)) = (end_pts(&curve0), end_pts(&curve1));
                if !p0.x.near(&p1.x) && !q0.x.near(&q1.x) {
                    if let (Some(urange), _) = surface.try_range_tuple() {
                        normalize_range(&mut curve0, 0, urange);
                        normalize_range(&mut curve1, 0, urange);
                    }
                } else if !p0.y.near(&p1.y) && !q0.y.near(&q1.y) {
                    if let (_, Some(vrange)) = surface.try_range_tuple() {
                        normalize_range(&mut curve0, 1, vrange);
                        normalize_range(&mut curve1, 1, vrange);
                    }
                }
                let ((p0, p1), (q0, q1)) = (end_pts(&curve0), end_pts(&curve1));
                let vec0 = polyline_on_surface(surface, p1, q0, tolerance, &mut point_cache);
                let vec1 = polyline_on_surface(surface, q1, p0, tolerance, &mut point_cache);
                closed.push(connect_edges([curve0, vec0, curve1, vec1]));
            }
            _ => {}
        }
        if !closed.iter().any(|curve| loop_orientation(curve)) {
            if let (Some((u0, u1)), Some((v0, v1))) = surface.try_range_tuple() {
                let p = [
                    surface_point_with_cache(surface, Point2::new(u0, v0), &mut point_cache),
                    surface_point_with_cache(surface, Point2::new(u1, v0), &mut point_cache),
                    surface_point_with_cache(surface, Point2::new(u1, v1), &mut point_cache),
                    surface_point_with_cache(surface, Point2::new(u0, v1), &mut point_cache),
                ];
                let vec0 = polyline_on_surface(surface, p[0], p[1], tolerance, &mut point_cache);
                let vec1 = polyline_on_surface(surface, p[1], p[2], tolerance, &mut point_cache);
                let vec2 = polyline_on_surface(surface, p[2], p[3], tolerance, &mut point_cache);
                let vec3 = polyline_on_surface(surface, p[3], p[0], tolerance, &mut point_cache);
                closed.push(connect_edges([vec0, vec1, vec2, vec3]));
            }
        }
        let (mut uv_min, mut uv_max) = (
            Point2::new(f64::INFINITY, f64::INFINITY),
            Point2::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        );
        for pt in closed.iter().flatten() {
            uv_min.x = f64::min(uv_min.x, pt.x);
            uv_min.y = f64::min(uv_min.y, pt.y);
            uv_max.x = f64::max(uv_max.x, pt.x);
            uv_max.y = f64::max(uv_max.y, pt.y);
        }
        Self {
            loops: closed,
            uv_min,
            uv_max,
        }
    }

    /// whether `c` is included in the domain with boundary = `self`.
    fn include(&self, c: Point2) -> bool {
        // AABB early reject.
        if c.x < self.uv_min.x || c.x > self.uv_max.x || c.y < self.uv_min.y || c.y > self.uv_max.y
        {
            return false;
        }
        let t = 2.0 * std::f64::consts::PI * HashGen::hash1(c);
        let r = Vector2::new(f64::cos(t), f64::sin(t));
        self.loops
            .iter()
            .flat_map(|vec| vec.iter().circular_tuple_windows())
            .try_fold(0_i32, move |counter, (p0, p1)| {
                let a = **p0 - c;
                let b = **p1 - c;
                let s0 = r.x * a.y - r.y * a.x; // v times a
                let s1 = r.x * b.y - r.y * b.x; // v times b
                let s2 = a.x * b.y - a.y * b.x; // a times b
                let x = s2 / (s1 - s0);
                if x.so_small() && s0 * s1 < 0.0 {
                    None
                } else if x > 0.0 && s0 <= 0.0 && s1 > 0.0 {
                    Some(counter + 1)
                } else if x > 0.0 && s0 >= 0.0 && s1 < 0.0 {
                    Some(counter - 1)
                } else {
                    Some(counter)
                }
            })
            .map(|counter| counter > 0)
            .unwrap_or(false)
    }

    /// Inserts points and adds constraint into triangulation.
    fn insert_to(
        &self,
        triangulation: &mut Cdt,
        boundary_map: &mut HashMap<FixedVertexHandle, Point3>,
    ) {
        let poly2tri: Vec<_> = self
            .loops
            .iter()
            .flatten()
            .map(|pt| {
                let p = [spade_round(pt.x), spade_round(pt.y)];
                match triangulation.insert(SPoint2::from(p)) {
                    Err(_) => None,
                    Ok(idx) => {
                        boundary_map.insert(idx, pt.point);
                        Some(idx)
                    }
                }
            })
            .collect();
        let mut prev: Option<usize> = None;
        let mut counter = 0;
        self.loops
            .iter()
            .map(Vec::len)
            .flat_map(|len| {
                let range = counter..counter + len;
                counter += len;
                range.circular_tuple_windows()
            })
            .for_each(|(i, j)| {
                let Some(vj) = poly2tri[j] else { return };
                if let Some(p) = prev {
                    let Some(v) = poly2tri[p] else { return };
                    if triangulation.can_add_constraint(v, vj) {
                        triangulation.add_constraint(v, vj);
                        prev = None;
                    }
                } else {
                    let Some(vi) = poly2tri[i] else { return };
                    if triangulation.can_add_constraint(vi, vj) {
                        triangulation.add_constraint(vi, vj);
                    } else {
                        prev = Some(i);
                    }
                }
            });
    }
}

fn spade_round(x: f64) -> f64 {
    match f64::abs(x) < MIN_ALLOWED_VALUE {
        true => 0.0,
        false => x,
    }
}

/// Tessellates a bounded surface without any trimming.
///
/// Generates a structured grid from parameter division, then triangulates
/// each quad cell into two triangles. Skips CDT and inclusion tests entirely.
fn untrimmed_tessellation<S>(
    surface: &S,
    range: ((f64, f64), (f64, f64)),
    tolerance: f64,
    quad_mode: QuadMode,
) -> PolygonMesh
where
    S: PreMeshableSurface,
{
    let (udiv, vdiv) = surface.parameter_division(range, tolerance);
    let nu = udiv.len();
    let nv = vdiv.len();
    let mut positions = Vec::with_capacity(nu * nv);
    let mut uv_coords = Vec::with_capacity(nu * nv);
    let mut normals = Vec::with_capacity(nu * nv);
    for u in &udiv {
        for v in &vdiv {
            positions.push(surface.subs(*u, *v));
            uv_coords.push(Vector2::new(*u, *v));
            normals.push(surface.normal(*u, *v));
        }
    }
    let idx = |i: usize, j: usize| -> usize { i * nv + j };
    let sv = |k: usize| -> StandardVertex { [k, k, k].into() };
    let (tri_faces, quad_faces) = if quad_mode == QuadMode::Triangles {
        let tri_faces: Vec<[StandardVertex; 3]> = (0..nu - 1)
            .flat_map(|i| {
                (0..nv - 1).flat_map(move |j| {
                    let a = idx(i, j);
                    let b = idx(i, j + 1);
                    let c = idx(i + 1, j + 1);
                    let d = idx(i + 1, j);
                    [[sv(a), sv(b), sv(c)], [sv(a), sv(c), sv(d)]]
                })
            })
            .collect();
        (tri_faces, Vec::new())
    } else {
        let quad_faces: Vec<[StandardVertex; 4]> = (0..nu - 1)
            .flat_map(|i| {
                (0..nv - 1).map(move |j| {
                    let a = idx(i, j);
                    let b = idx(i, j + 1);
                    let c = idx(i + 1, j + 1);
                    let d = idx(i + 1, j);
                    [sv(a), sv(b), sv(c), sv(d)]
                })
            })
            .collect();
        (Vec::new(), quad_faces)
    };
    let mut mesh = PolygonMesh::debug_new(
        StandardAttributes {
            positions,
            uv_coords,
            normals,
        },
        Faces::from_tri_and_quad_faces(tri_faces, quad_faces),
    );
    mesh.make_face_compatible_to_normal();
    mesh
}

/// Tessellates one surface trimmed by polyline.
fn trimming_tessellation<S>(
    surface: &S,
    polyboundary: &PolyBoundary,
    tolerance: f64,
    quad_config: QuadOptions,
) -> PolygonMesh
where
    S: PreMeshableSurface,
{
    if quad_config.mode == QuadMode::IsoQuads {
        if let Some(mut mesh) = iso_quad_trimmed_tessellation(surface, polyboundary, tolerance) {
            mesh.make_face_compatible_to_normal();
            mesh
        } else {
            let mut mesh = cdt_trimming_tessellation(surface, polyboundary, tolerance);
            mesh.make_face_compatible_to_normal();
            mesh
        }
    } else {
        let mut mesh = cdt_trimming_tessellation(surface, polyboundary, tolerance);
        mesh.make_face_compatible_to_normal();
        apply_quad_mode(&mut mesh, quad_config);
        mesh
    }
}

fn cdt_trimming_tessellation<S>(
    surface: &S,
    polyboundary: &PolyBoundary,
    tolerance: f64,
) -> PolygonMesh
where
    S: PreMeshableSurface,
{
    let mut triangulation = Cdt::new();
    let mut boundary_map = HashMap::<FixedVertexHandle, Point3>::default();
    polyboundary.insert_to(&mut triangulation, &mut boundary_map);
    insert_surface(&mut triangulation, surface, polyboundary, tolerance);
    triangulation_into_polymesh(
        triangulation.vertices(),
        triangulation.inner_faces(),
        surface,
        polyboundary,
        &boundary_map,
    )
}

fn iso_quad_trimmed_tessellation<S>(
    surface: &S,
    polyboundary: &PolyBoundary,
    tolerance: f64,
) -> Option<PolygonMesh>
where
    S: PreMeshableSurface,
{
    let range = (
        (polyboundary.uv_min.x, polyboundary.uv_max.x),
        (polyboundary.uv_min.y, polyboundary.uv_max.y),
    );
    let (udiv, vdiv) = surface.parameter_division(range, tolerance);
    let (nu, nv) = (udiv.len(), vdiv.len());
    if nu < 2 || nv < 2 {
        return None;
    }
    let idx = |i: usize, j: usize| -> usize { i * nv + j };
    let sv = |k: usize| -> StandardVertex { [k, k, k].into() };
    let positions = udiv
        .iter()
        .flat_map(|u| vdiv.iter().map(move |v| surface.subs(*u, *v)))
        .collect::<Vec<_>>();
    let uv_coords = udiv
        .iter()
        .flat_map(|u| vdiv.iter().map(move |v| Vector2::new(*u, *v)))
        .collect::<Vec<_>>();
    let normals = udiv
        .iter()
        .flat_map(|u| vdiv.iter().map(move |v| surface.normal(*u, *v)))
        .collect::<Vec<_>>();
    let inside = udiv
        .iter()
        .flat_map(|u| {
            vdiv.iter()
                .map(move |v| polyboundary.include(Point2::new(*u, *v)))
        })
        .collect::<Vec<_>>();
    let nv_cells = nv - 1;
    let (udiv_ref, vdiv_ref, inside_ref) = (&udiv, &vdiv, &inside);
    let full_cells = (0..nu - 1)
        .flat_map(|i| {
            (0..nv - 1).map(move |j| {
                let corner_indices = [idx(i, j), idx(i, j + 1), idx(i + 1, j + 1), idx(i + 1, j)];
                let corners_inside = corner_indices.into_iter().all(|k| inside_ref[k]);
                if !corners_inside {
                    false
                } else {
                    let (u0, u1) = (udiv_ref[i], udiv_ref[i + 1]);
                    let (v0, v1) = (vdiv_ref[j], vdiv_ref[j + 1]);
                    let center = Point2::new((u0 + u1) * 0.5, (v0 + v1) * 0.5);
                    let edge_midpoints = [
                        Point2::new((u0 + u1) * 0.5, v0),
                        Point2::new(u1, (v0 + v1) * 0.5),
                        Point2::new((u0 + u1) * 0.5, v1),
                        Point2::new(u0, (v0 + v1) * 0.5),
                    ];
                    polyboundary.include(center)
                        && edge_midpoints
                            .into_iter()
                            .all(|midpoint| polyboundary.include(midpoint))
                }
            })
        })
        .collect::<Vec<_>>();
    let interior_quads = full_cells
        .iter()
        .enumerate()
        .filter_map(|(cell_index, is_full)| {
            if *is_full {
                let i = cell_index / nv_cells;
                let j = cell_index % nv_cells;
                let a = idx(i, j);
                let b = idx(i, j + 1);
                let c = idx(i + 1, j + 1);
                let d = idx(i + 1, j);
                Some([sv(a), sv(b), sv(c), sv(d)])
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if interior_quads.is_empty() {
        None
    } else {
        let mut interior_mesh = PolygonMesh::debug_new(
            StandardAttributes {
                positions,
                uv_coords,
                normals,
            },
            Faces::from_tri_and_quad_faces(Vec::new(), interior_quads),
        );
        let mut boundary_mesh = cdt_trimming_tessellation(surface, polyboundary, tolerance);
        let boundary_triangles = boundary_mesh
            .tri_faces()
            .iter()
            .copied()
            .filter(|triangle| {
                !triangle_is_in_full_cell(
                    *triangle,
                    boundary_mesh.uv_coords(),
                    &udiv,
                    &vdiv,
                    &full_cells,
                )
            })
            .collect::<Vec<_>>();
        let boundary_quads = boundary_mesh
            .quad_faces()
            .iter()
            .copied()
            .filter(|quad| {
                !quad_is_in_full_cell(*quad, boundary_mesh.uv_coords(), &udiv, &vdiv, &full_cells)
            })
            .collect::<Vec<_>>();
        {
            let editor = boundary_mesh.debug_editor();
            *editor.faces = Faces::from_tri_and_quad_faces(boundary_triangles, boundary_quads);
        }
        interior_mesh.merge(boundary_mesh);
        Some(interior_mesh)
    }
}

fn triangle_is_in_full_cell(
    triangle: [StandardVertex; 3],
    uv_coords: &[Vector2],
    udiv: &[f64],
    vdiv: &[f64],
    full_cells: &[bool],
) -> bool {
    triangle_center_uv(triangle, uv_coords)
        .is_some_and(|uv| uv_is_in_full_cell(uv, udiv, vdiv, full_cells))
}

fn quad_is_in_full_cell(
    quad: [StandardVertex; 4],
    uv_coords: &[Vector2],
    udiv: &[f64],
    vdiv: &[f64],
    full_cells: &[bool],
) -> bool {
    quad_center_uv(quad, uv_coords).is_some_and(|uv| uv_is_in_full_cell(uv, udiv, vdiv, full_cells))
}

fn triangle_center_uv(triangle: [StandardVertex; 3], uv_coords: &[Vector2]) -> Option<Vector2> {
    let uv0 = uv_coords[*triangle[0].uv.as_ref()?];
    let uv1 = uv_coords[*triangle[1].uv.as_ref()?];
    let uv2 = uv_coords[*triangle[2].uv.as_ref()?];
    Some((uv0 + uv1 + uv2) / 3.0)
}

fn quad_center_uv(quad: [StandardVertex; 4], uv_coords: &[Vector2]) -> Option<Vector2> {
    let uv0 = uv_coords[*quad[0].uv.as_ref()?];
    let uv1 = uv_coords[*quad[1].uv.as_ref()?];
    let uv2 = uv_coords[*quad[2].uv.as_ref()?];
    let uv3 = uv_coords[*quad[3].uv.as_ref()?];
    Some((uv0 + uv1 + uv2 + uv3) / 4.0)
}

fn uv_is_in_full_cell(uv: Vector2, udiv: &[f64], vdiv: &[f64], full_cells: &[bool]) -> bool {
    if vdiv.len() < 2 {
        false
    } else {
        let nv_cells = vdiv.len() - 1;
        if let (Some(i), Some(j)) = (
            parameter_cell_index(udiv, uv.x),
            parameter_cell_index(vdiv, uv.y),
        ) {
            full_cells.get(i * nv_cells + j).copied().unwrap_or(false)
        } else {
            false
        }
    }
}

fn parameter_cell_index(parameters: &[f64], value: f64) -> Option<usize> {
    if parameters.len() < 2 || value < parameters[0] || value > parameters[parameters.len() - 1] {
        None
    } else {
        let upper = parameters.partition_point(|sample| *sample <= value);
        if upper == 0 {
            None
        } else if upper >= parameters.len() {
            Some(parameters.len() - 2)
        } else {
            Some(upper - 1)
        }
    }
}

fn apply_quad_mode(mesh: &mut PolygonMesh, quad_config: QuadOptions) {
    if quad_config.mode != QuadMode::Triangles {
        mesh.quadrangulate(quad_config.plane_tolerance, quad_config.score_tolerance);
        if quad_config.mode == QuadMode::AllQuads {
            force_all_triangles_to_quads(mesh, quad_config);
        }
    }
}

fn force_all_triangles_to_quads(mesh: &mut PolygonMesh, quad_config: QuadOptions) {
    if mesh.tri_faces().is_empty() {
        return;
    }
    let normal_blend_angle = quad_config.normal_blend_angle;
    let triangles = mesh.tri_faces().clone();
    let mut quadrangles = mesh.quad_faces().clone();
    let mut midpoint_cache = HashMap::<(VertexKey, VertexKey), StandardVertex>::default();
    triangles.into_iter().for_each(|triangle| {
        let [vertex0, vertex1, vertex2] = triangle;
        let point0 = mesh.positions()[vertex0.pos];
        let point1 = mesh.positions()[vertex1.pos];
        let point2 = mesh.positions()[vertex2.pos];
        let point01 = point0.midpoint(point1);
        let point12 = point1.midpoint(point2);
        let point20 = point2.midpoint(point0);
        let center_point =
            Point3::from_vec((point0.to_vec() + point1.to_vec() + point2.to_vec()) / 3.0);
        let split_quads = [
            [point0, point01, center_point, point20],
            [point1, point12, center_point, point01],
            [point2, point20, center_point, point12],
        ];
        let quality_ok = split_quads
            .into_iter()
            .all(|quad| quad_passes_quality_gates(quad, quad_config));
        if quality_ok {
            let midpoint01 = midpoint_for_edge(
                mesh,
                &mut midpoint_cache,
                vertex0,
                vertex1,
                normal_blend_angle,
            );
            let midpoint12 = midpoint_for_edge(
                mesh,
                &mut midpoint_cache,
                vertex1,
                vertex2,
                normal_blend_angle,
            );
            let midpoint20 = midpoint_for_edge(
                mesh,
                &mut midpoint_cache,
                vertex2,
                vertex0,
                normal_blend_angle,
            );
            let center = create_centroid_vertex(mesh, triangle, normal_blend_angle);
            quadrangles.extend([
                [vertex0, midpoint01, center, midpoint20],
                [vertex1, midpoint12, center, midpoint01],
                [vertex2, midpoint20, center, midpoint12],
            ]);
        } else {
            quadrangles.push([vertex0, vertex1, vertex2, vertex2]);
        }
    });
    {
        let editor = mesh.debug_editor();
        *editor.faces = Faces::from_tri_and_quad_faces(Vec::new(), quadrangles);
    }
    mesh.make_face_compatible_to_normal();
}

type VertexKey = (usize, Option<usize>, Option<usize>);

fn midpoint_for_edge(
    mesh: &mut PolygonMesh,
    midpoint_cache: &mut HashMap<(VertexKey, VertexKey), StandardVertex>,
    vertex0: StandardVertex,
    vertex1: StandardVertex,
    normal_blend_angle: f64,
) -> StandardVertex {
    let key = edge_key(vertex0, vertex1);
    if let Some(vertex) = midpoint_cache.get(&key) {
        *vertex
    } else {
        let midpoint = create_midpoint_vertex(mesh, vertex0, vertex1, normal_blend_angle);
        midpoint_cache.insert(key, midpoint);
        midpoint
    }
}

fn edge_key(vertex0: StandardVertex, vertex1: StandardVertex) -> (VertexKey, VertexKey) {
    let key0 = (vertex0.pos, vertex0.uv, vertex0.nor);
    let key1 = (vertex1.pos, vertex1.uv, vertex1.nor);
    if key0 <= key1 {
        (key0, key1)
    } else {
        (key1, key0)
    }
}

fn quad_passes_quality_gates(quad: [Point3; 4], quad_config: QuadOptions) -> bool {
    let area = quad_area(quad);
    let convex = is_convex_quad(quad);
    let corner_angles_ok = quad.iter().enumerate().all(|(index, _)| {
        corner_angle(quad, index).is_some_and(|angle| angle <= quad_config.maximum_corner_angle)
    });
    area >= quad_config.minimum_area && convex && corner_angles_ok
}

fn quad_area(quad: [Point3; 4]) -> f64 {
    triangle_area(quad[0], quad[1], quad[2]) + triangle_area(quad[0], quad[2], quad[3])
}

fn triangle_area(point0: Point3, point1: Point3, point2: Point3) -> f64 {
    let vector0 = point1.to_vec() - point0.to_vec();
    let vector1 = point2.to_vec() - point0.to_vec();
    vector0.cross(vector1).magnitude() * 0.5
}

fn is_convex_quad(quad: [Point3; 4]) -> bool {
    let diagonal0 = quad[1].to_vec() - quad[0].to_vec();
    let diagonal1 = quad[2].to_vec() - quad[0].to_vec();
    let normal = diagonal0.cross(diagonal1);
    if normal.so_small() {
        false
    } else {
        let (has_positive, has_negative, has_nonzero) = quad
            .iter()
            .enumerate()
            .map(|(index, _)| corner_turn(quad, index, normal))
            .fold(
                (false, false, false),
                |(positive, negative, nonzero), turn| {
                    (
                        positive || turn > TOLERANCE,
                        negative || turn < -TOLERANCE,
                        nonzero || f64::abs(turn) > TOLERANCE,
                    )
                },
            );
        has_nonzero && !(has_positive && has_negative)
    }
}

fn corner_turn(quad: [Point3; 4], index: usize, normal: Vector3) -> f64 {
    let previous = quad[(index + 3) % 4];
    let current = quad[index];
    let next = quad[(index + 1) % 4];
    let edge0 = current.to_vec() - previous.to_vec();
    let edge1 = next.to_vec() - current.to_vec();
    edge0.cross(edge1).dot(normal)
}

fn corner_angle(quad: [Point3; 4], index: usize) -> Option<f64> {
    let previous = quad[(index + 3) % 4];
    let current = quad[index];
    let next = quad[(index + 1) % 4];
    let vector0 = previous.to_vec() - current.to_vec();
    let vector1 = next.to_vec() - current.to_vec();
    let length0 = vector0.magnitude();
    let length1 = vector1.magnitude();
    if length0.so_small() || length1.so_small() {
        None
    } else {
        let cosine = (vector0.dot(vector1) / (length0 * length1)).clamp(-1.0, 1.0);
        Some(cosine.acos())
    }
}

fn create_midpoint_vertex(
    mesh: &mut PolygonMesh,
    vertex0: StandardVertex,
    vertex1: StandardVertex,
    normal_blend_angle: f64,
) -> StandardVertex {
    let point0 = mesh.positions()[vertex0.pos];
    let point1 = mesh.positions()[vertex1.pos];
    let position = point0.midpoint(point1);
    let position_index = mesh.positions().len();
    mesh.push_position(position);

    let uv_coord = match (vertex0.uv, vertex1.uv) {
        (Some(index0), Some(index1)) => {
            Some((mesh.uv_coords()[index0] + mesh.uv_coords()[index1]) / 2.0)
        }
        _ => None,
    };
    let uv_index = uv_coord.map(|uv_coord| {
        let index = mesh.uv_coords().len();
        mesh.push_uv_coord(uv_coord);
        index
    });

    let normal = match (vertex0.nor, vertex1.nor) {
        (Some(index0), Some(index1)) => Some(blend_normals(
            mesh.normals()[index0],
            mesh.normals()[index1],
            normal_blend_angle,
        )),
        _ => None,
    };
    let normal_index = normal.map(|normal| {
        let index = mesh.normals().len();
        mesh.extend_normals([normal]);
        index
    });

    StandardVertex {
        pos: position_index,
        uv: uv_index,
        nor: normal_index,
    }
}

fn create_centroid_vertex(
    mesh: &mut PolygonMesh,
    triangle: [StandardVertex; 3],
    normal_blend_angle: f64,
) -> StandardVertex {
    let [vertex0, vertex1, vertex2] = triangle;
    let point0 = mesh.positions()[vertex0.pos];
    let point1 = mesh.positions()[vertex1.pos];
    let point2 = mesh.positions()[vertex2.pos];
    let position = Point3::from_vec((point0.to_vec() + point1.to_vec() + point2.to_vec()) / 3.0);
    let position_index = mesh.positions().len();
    mesh.push_position(position);

    let uv_coord = match (vertex0.uv, vertex1.uv, vertex2.uv) {
        (Some(index0), Some(index1), Some(index2)) => Some(
            (mesh.uv_coords()[index0] + mesh.uv_coords()[index1] + mesh.uv_coords()[index2]) / 3.0,
        ),
        _ => None,
    };
    let uv_index = uv_coord.map(|uv_coord| {
        let index = mesh.uv_coords().len();
        mesh.push_uv_coord(uv_coord);
        index
    });

    let normal = match (vertex0.nor, vertex1.nor, vertex2.nor) {
        (Some(index0), Some(index1), Some(index2)) => {
            let normal01 = blend_normals(
                mesh.normals()[index0],
                mesh.normals()[index1],
                normal_blend_angle,
            );
            Some(blend_normals(
                normal01,
                mesh.normals()[index2],
                normal_blend_angle,
            ))
        }
        _ => None,
    };
    let normal_index = normal.map(|normal| {
        let index = mesh.normals().len();
        mesh.extend_normals([normal]);
        index
    });

    StandardVertex {
        pos: position_index,
        uv: uv_index,
        nor: normal_index,
    }
}

fn blend_normals(normal0: Vector3, normal1: Vector3, normal_blend_angle: f64) -> Vector3 {
    let cosine = normal0.dot(normal1);
    let min_cosine = f64::cos(normal_blend_angle);
    let blended = if cosine < min_cosine {
        normal0
    } else {
        normal0 + normal1
    };
    let magnitude = blended.magnitude();
    if magnitude.so_small() {
        normal0
    } else {
        blended / magnitude
    }
}

/// Inserts parameter divisions into triangulation.
fn insert_surface(
    triangulation: &mut Cdt,
    surface: impl PreMeshableSurface,
    polyline: &PolyBoundary,
    tolerance: f64,
) {
    let range = (
        (polyline.uv_min.x, polyline.uv_max.x),
        (polyline.uv_min.y, polyline.uv_max.y),
    );
    let (udiv, vdiv) = surface.parameter_division(range, tolerance);
    let insert_res: Vec<Vec<Option<_>>> = udiv
        .into_iter()
        .map(|u| {
            vdiv.iter()
                .map(|v| match polyline.include(Point2::new(u, *v)) {
                    true => triangulation.insert(SPoint2::new(u, *v)).ok(),
                    false => None,
                })
                .collect()
        })
        .collect();
    insert_res.windows(2).for_each(|vec| {
        vec[0].windows(2).zip(&vec[1]).for_each(|(a, z)| {
            if let Some(x) = a[0] {
                if let Some(y) = a[1] {
                    if triangulation.can_add_constraint(x, y) {
                        triangulation.add_constraint(x, y);
                    }
                }
                if let Some(z) = z {
                    if triangulation.can_add_constraint(x, *z) {
                        triangulation.add_constraint(x, *z);
                    }
                }
            }
        });
        let idx = vec[0].len() - 1;
        if let (Some(x), Some(y)) = (vec[0][idx], vec[1][idx]) {
            if triangulation.can_add_constraint(x, y) {
                triangulation.add_constraint(x, y);
            }
        }
    });
}

/// Converts triangulation into `PolygonMesh`.
fn triangulation_into_polymesh<'a>(
    vertices: VertexIterator<'a, SPoint2, (), CdtEdge<()>, ()>,
    triangles: InnerFaceIterator<'a, SPoint2, (), CdtEdge<()>, ()>,
    surface: &impl ParametricSurface3D,
    polyline: &PolyBoundary,
    boundary_map: &HashMap<FixedVertexHandle, Point3>,
) -> PolygonMesh {
    let mut positions = Vec::<Point3>::new();
    let mut uv_coords = Vec::<Vector2>::new();
    let mut normals = Vec::<Vector3>::new();
    let mut surface_point_cache = HashMap::<UvKey, Point3>::default();
    let mut normal_cache = HashMap::<UvKey, Vector3>::default();
    let vmap: HashMap<_, _> = vertices
        .enumerate()
        .map(|(i, v)| {
            let p = *v.as_ref();
            let uv = Point2::new(p.x, p.y);
            let key = uv_key(uv);
            let idx = v.fix();
            let point = match boundary_map.get(&idx) {
                Some(point) => *point,
                None => *surface_point_cache
                    .entry(key)
                    .or_insert_with(|| surface.subs(p.x, p.y)),
            };
            let normal = *normal_cache
                .entry(key)
                .or_insert_with(|| surface.normal(p.x, p.y));
            positions.push(point);
            uv_coords.push(Vector2::new(p.x, p.y));
            normals.push(normal);
            (idx, i)
        })
        .collect();
    let tri_faces: Vec<[StandardVertex; 3]> = triangles
        .map(|tri| tri.vertices())
        .filter(|tri| {
            fn sp2cg(p: SPoint2) -> Point2 { Point2::new(p.x, p.y) }
            let tri = array![i => sp2cg(*tri[i].as_ref()); 3];
            let (a, b) = (tri[1] - tri[0], tri[2] - tri[0]);
            let c = tri[0] + (a + b) / 3.0;
            let area = a.x * b.y - a.y * b.x;
            polyline.include(c) && !area.so_small2()
        })
        .map(|tri| {
            let idcs = array![i => vmap[&tri[i].fix()]; 3];
            array![i => [idcs[i], idcs[i], idcs[i]].into(); 3]
        })
        .collect();
    PolygonMesh::debug_new(
        StandardAttributes {
            positions,
            uv_coords,
            normals,
        },
        Faces::from_tri_and_quad_faces(tri_faces, Vec::new()),
    )
}

fn polyline_on_surface(
    surface: impl PreMeshableSurface,
    p: SurfacePoint,
    q: SurfacePoint,
    tolerance: f64,
    point_cache: &mut HashMap<UvKey, Point3>,
) -> Vec<SurfacePoint> {
    use truck_geometry::prelude::*;
    let line = Line(p.uv, q.uv);
    let pcurve = PCurve::new(line, &surface);
    let (vec, _) = pcurve.parameter_division(pcurve.range_tuple(), tolerance);
    vec.into_iter()
        .map(|t| {
            let uv = line.subs(t);
            surface_point_with_cache(&surface, uv, point_cache)
        })
        .collect()
}

#[test]
#[ignore]
#[cfg(not(target_arch = "wasm32"))]
fn par_bench() {
    use std::time::Instant;
    use truck_modeling::*;
    const JSON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../resources/shape/bottle.json"
    ));
    let solid: Solid = serde_json::from_str(JSON).unwrap();
    let shell = solid.into_boundaries().pop().unwrap();

    let instant = Instant::now();
    (0..100).for_each(|_| {
        let _shell = shell_tessellation(&shell, 0.01, by_search_parameter, QuadOptions::default());
    });
    println!("{}ms", instant.elapsed().as_millis());

    let instant = Instant::now();
    (0..100).for_each(|_| {
        let _shell = shell_tessellation_single_thread(
            &shell,
            0.01,
            by_search_parameter,
            QuadOptions::default(),
        );
    });
    println!("{}ms", instant.elapsed().as_millis());
}
