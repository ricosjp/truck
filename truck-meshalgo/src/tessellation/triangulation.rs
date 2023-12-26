#![allow(clippy::many_single_char_names)]

use super::*;
use crate::filters::NormalFilters;
use crate::Point2;
use array_macro::array;
use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

type SPoint2 = spade::Point2<f64>;
type Cdt = ConstrainedDelaunayTriangulation<SPoint2>;
type MeshedShell = Shell<Point3, PolylineCurve, Option<PolygonMesh>>;
type MeshedCShell = CompressedShell<Point3, PolylineCurve, Option<PolygonMesh>>;

pub(super) fn by_search_parameter<S>(
    surface: &S,
    point: Point3,
    hint: Option<(f64, f64)>,
) -> Option<(f64, f64)>
where
    S: MeshableSurface,
{
    surface
        .search_parameter(point, hint, 100)
        .or_else(|| surface.search_parameter(point, None, 100))
}

pub(super) fn by_search_nearest_parameter<S>(
    surface: &S,
    point: Point3,
    hint: Option<(f64, f64)>,
) -> Option<(f64, f64)>
where
    S: RobustMeshableSurface,
{
    surface
        .search_parameter(point, hint, 100)
        .or_else(|| surface.search_parameter(point, None, 100))
        .or_else(|| surface.search_nearest_parameter(point, hint, 100))
        .or_else(|| surface.search_nearest_parameter(point, None, 100))
}

/// Tessellates faces
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn shell_tessellation<'a, C, S, F>(
    shell: &Shell<Point3, C, S>,
    tol: f64,
    sp: F,
) -> MeshedShell
where
    C: PolylineableCurve + 'a,
    S: PreMeshableSurface + 'a,
    F: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> + Parallelizable,
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
            let poly = PolylineCurve::from_curve(&curve, curve.range_tuple(), tol);
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
        shell_create_polygon(&face.surface(), wires, face.orientation(), tol, &sp)
    };
    shell.face_par_iter().map(create_face).collect()
}

/// Tessellates faces
#[cfg(any(target = "wasm32", test))]
pub(super) fn shell_tessellation_single_thread<'a, C, S, F>(
    shell: &'a Shell<Point3, C, S>,
    tol: f64,
    sp: F,
) -> MeshedShell
where
    C: PolylineableCurve + 'a,
    S: PreMeshableSurface + 'a,
    F: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)>,
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
            let poly = PolylineCurve::from_curve(&curve, curve.range_tuple(), tol);
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
        shell_create_polygon(&face.surface(), wires, face.orientation(), tol, &sp)
    };
    shell.face_iter().map(create_face).collect()
}

/// Tessellates faces
pub(super) fn cshell_tessellation<'a, C, S, F>(
    shell: &CompressedShell<Point3, C, S>,
    tol: f64,
    sp: F,
) -> MeshedCShell
where
    C: PolylineableCurve + 'a,
    S: PreMeshableSurface + 'a,
    F: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> + Parallelizable,
{
    let vertices = shell.vertices.clone();
    let tessellate_edge = |edge: &CompressedEdge<C>| {
        let curve = &edge.curve;
        CompressedEdge {
            vertices: edge.vertices,
            curve: PolylineCurve::from_curve(curve, curve.range_tuple(), tol),
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let edges: Vec<_> = shell.edges.par_iter().map(tessellate_edge).collect();
    #[cfg(target_arch = "wasm32")]
    let edges: Vec<_> = shell.edges.iter().map(tessellate_edge).collect();
    let tessellate_face = |face: &CompressedFace<S>| {
        // vertex loop case
        if face.boundaries.is_empty() {
            let surface = &face.surface;
            let polygon = if let (Some(urange), Some(vrange)) = surface.try_range_tuple() {
                Some(StructuredMesh::from_surface(surface, (urange, vrange), tol).destruct())
            } else {
                None
            };
            CompressedFace {
                boundaries: Vec::new(),
                orientation: face.orientation,
                surface: polygon,
            }
        } else {
            let boundaries = face.boundaries.clone();
            let surface = &face.surface;
            let create_edge = |edge_idx: &CompressedEdgeIndex| match edge_idx.orientation {
                true => Some(edges.get(edge_idx.index)?.curve.clone()),
                false => Some(edges.get(edge_idx.index)?.curve.inverse()),
            };
            let create_boundary = |wire: &Vec<CompressedEdgeIndex>| {
                let wire_iter = wire.iter().filter_map(create_edge);
                PolyBoundaryPiece::try_new(surface, wire_iter, &sp)
            };
            let preboundary: Option<Vec<_>> = boundaries.iter().map(create_boundary).collect();
            let polygon: Option<PolygonMesh> = (|| {
                let boundary = PolyBoundary::new(preboundary?, &surface, tol);
                Some(trimming_tessellation(&surface, &boundary, tol))
            })();
            CompressedFace {
                boundaries,
                orientation: face.orientation,
                surface: polygon,
            }
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
    tol: f64,
    sp: impl Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)>,
) -> Face<Point3, PolylineCurve, Option<PolygonMesh>> {
    let preboundary = wires
        .iter()
        .map(|wire: &Wire<_, _>| {
            let wire_iter = wire.iter().map(Edge::oriented_curve);
            PolyBoundaryPiece::try_new(surface, wire_iter, &sp)
        })
        .collect::<Option<Vec<_>>>();
    let polygon: Option<PolygonMesh> = (|| {
        let boundary = PolyBoundary::new(preboundary?, &surface, tol);
        Some(trimming_tessellation(surface, &boundary, tol))
    })();
    let mut new_face = Face::debug_new(wires, polygon);
    if !orientation {
        new_face.invert();
    }
    new_face
}

#[derive(Debug, Default, Clone)]
struct PolyBoundaryPiece(Vec<Point2>);

impl PolyBoundaryPiece {
    fn try_new<S: PreMeshableSurface>(
        surface: &S,
        wire: impl Iterator<Item = PolylineCurve>,
        sp: impl Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)>,
    ) -> Option<Self> {
        let mut previous: Option<(f64, f64)> = None;
        let up = surface.u_period();
        let vp = surface.v_period();
        let mut bdry3d: Vec<Point3> = wire
            .flat_map(|poly_edge| {
                let n = poly_edge.len() - 1;
                poly_edge.0.into_iter().take(n)
            })
            .collect();
        bdry3d.push(bdry3d[0]);
        let mut hint = None;
        let vec = bdry3d
            .into_iter()
            .map(|pt| {
                hint = sp(surface, pt, hint);
                fn abs_diff(previous: f64) -> impl Fn(&f64, &f64) -> std::cmp::Ordering {
                    let f = move |x: &f64| f64::abs(x - previous);
                    move |x: &f64, y: &f64| f(x).partial_cmp(&f(y)).unwrap()
                }
                if let (Some((ref mut hint, _)), Some(up), Some((previous, _))) =
                    (&mut hint, up, previous)
                {
                    *hint = (-2..=2)
                        .map(|i| *hint + i as f64 * up)
                        .min_by(abs_diff(previous))
                        .unwrap();
                }
                if let (Some((_, ref mut hint)), Some(vp), Some((_, previous))) =
                    (&mut hint, vp, previous)
                {
                    *hint = (-2..=2)
                        .map(|i| *hint + i as f64 * vp)
                        .min_by(abs_diff(previous))
                        .unwrap();
                }
                previous = hint;
                hint.map(Into::<Point2>::into)
            })
            .collect::<Option<Vec<Point2>>>()?;
        Some(Self(vec))
    }
}

#[derive(Debug, Default, Clone)]
struct PolyBoundary(Vec<Vec<Point2>>);

fn noramlize_range(curve: &mut Vec<Point2>, compidx: usize, (u0, u1): (f64, f64)) {
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

impl PolyBoundary {
    fn new(pieces: Vec<PolyBoundaryPiece>, surface: &impl PreMeshableSurface, tol: f64) -> Self {
        let (mut closed, mut open) = (Vec::new(), Vec::new());
        pieces.into_iter().for_each(|PolyBoundaryPiece(mut vec)| {
            match vec[0].distance(vec[vec.len() - 1]) < TOLERANCE * 100.0 {
                true => {
                    vec.pop();
                    closed.push(vec)
                }
                false => open.push(vec),
            }
        });
        fn connect_edges(vecs: impl IntoIterator<Item = Vec<Point2>>) -> Vec<Point2> {
            let closure = |vec: Vec<Point2>| {
                let len = vec.len();
                vec.into_iter().take(len - 1)
            };
            vecs.into_iter().flat_map(closure).collect()
        }
        match open.len() {
            1 => {
                let mut curve = open.pop().unwrap();
                let p = curve[0];
                let q = curve[curve.len() - 1];
                if let (Some((u0, u1)), Some((v0, v1))) = surface.try_range_tuple() {
                    if p.x < q.x - TOLERANCE {
                        noramlize_range(&mut curve, 0, (u0, u1));
                        let p = curve[0];
                        let q = curve[curve.len() - 1];
                        let x = Point2::new(u0, v1);
                        let y = Point2::new(u1, v1);
                        let vec0 = polyline_on_surface(surface, q, y, tol);
                        let vec1 = polyline_on_surface(surface, y, x, tol);
                        let vec2 = polyline_on_surface(surface, x, p, tol);
                        closed.push(connect_edges([vec0, vec1, vec2, curve]));
                    } else if q.x < p.x - TOLERANCE {
                        noramlize_range(&mut curve, 0, (u0, u1));
                        let p = curve[0];
                        let q = curve[curve.len() - 1];
                        let x = Point2::new(u1, v0);
                        let y = Point2::new(u0, v0);
                        let vec0 = polyline_on_surface(surface, q, y, tol);
                        let vec1 = polyline_on_surface(surface, y, x, tol);
                        let vec2 = polyline_on_surface(surface, x, p, tol);
                        closed.push(connect_edges([vec0, vec1, vec2, curve]));
                    } else if p.y < q.y - TOLERANCE {
                        noramlize_range(&mut curve, 1, (v0, v1));
                        let p = curve[0];
                        let q = curve[curve.len() - 1];
                        let x = Point2::new(u0, v0);
                        let y = Point2::new(u0, v1);
                        let vec0 = polyline_on_surface(surface, q, y, tol);
                        let vec1 = polyline_on_surface(surface, y, x, tol);
                        let vec2 = polyline_on_surface(surface, x, p, tol);
                        closed.push(connect_edges([vec0, vec1, vec2, curve]));
                    } else if q.y < p.y - TOLERANCE {
                        noramlize_range(&mut curve, 1, (v0, v1));
                        let p = curve[0];
                        let q = curve[curve.len() - 1];
                        let x = Point2::new(u1, v1);
                        let y = Point2::new(u1, v0);
                        let vec0 = polyline_on_surface(surface, q, y, tol);
                        let vec1 = polyline_on_surface(surface, y, x, tol);
                        let vec2 = polyline_on_surface(surface, x, p, tol);
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
                        noramlize_range(&mut curve0, 0, urange);
                        noramlize_range(&mut curve1, 0, urange);
                    }
                } else if !p0.y.near(&p1.y) && !q0.y.near(&q1.y) {
                    if let (_, Some(vrange)) = surface.try_range_tuple() {
                        noramlize_range(&mut curve0, 1, vrange);
                        noramlize_range(&mut curve1, 1, vrange);
                    }
                }
                let ((p0, p1), (q0, q1)) = (end_pts(&curve0), end_pts(&curve1));
                let vec0 = polyline_on_surface(surface, p1, q0, tol);
                let vec1 = polyline_on_surface(surface, q1, p0, tol);
                closed.push(connect_edges([curve0, vec0, curve1, vec1]));
            }
            _ => {}
        }
        Self(closed)
    }

    /// whether `c` is included in the domain with boundary = `self`.
    fn include(&self, c: Point2) -> bool {
        let t = 2.0 * std::f64::consts::PI * HashGen::hash1(c);
        let r = Vector2::new(f64::cos(t), f64::sin(t));
        self.0
            .iter()
            .flat_map(|vec| vec.iter().circular_tuple_windows())
            .try_fold(0_i32, move |counter, (p0, p1)| {
                let a = p0 - c;
                let b = p1 - c;
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
    fn insert_to(&self, triangulation: &mut Cdt) {
        let poly2tri: Vec<_> = self
            .0
            .iter()
            .flatten()
            .filter_map(|pt| triangulation.insert(SPoint2::from([pt.x, pt.y])).ok())
            .collect();
        let mut prev: Option<usize> = None;
        let mut counter = 0;
        self.0
            .iter()
            .map(Vec::len)
            .flat_map(|len| {
                let range = counter..counter + len;
                counter += len;
                range.circular_tuple_windows()
            })
            .for_each(|(i, j)| {
                if let Some(p) = prev {
                    if triangulation.can_add_constraint(poly2tri[p], poly2tri[j]) {
                        triangulation.add_constraint(poly2tri[p], poly2tri[j]);
                        prev = None;
                    }
                } else if triangulation.can_add_constraint(poly2tri[i], poly2tri[j]) {
                    triangulation.add_constraint(poly2tri[i], poly2tri[j]);
                } else {
                    prev = Some(i);
                }
            });
    }
}

/// Tessellates one surface trimmed by polyline.
fn trimming_tessellation<S>(surface: &S, polyboundary: &PolyBoundary, tol: f64) -> PolygonMesh
where S: PreMeshableSurface {
    let mut triangulation = Cdt::new();
    polyboundary.insert_to(&mut triangulation);
    insert_surface(&mut triangulation, surface, polyboundary, tol);
    let mut mesh = triangulation_into_polymesh(
        triangulation.vertices(),
        triangulation.inner_faces(),
        surface,
        polyboundary,
    );
    mesh.make_face_compatible_to_normal();
    mesh
}

/// Inserts parameter divisions into triangulation.
fn insert_surface(
    triangulation: &mut Cdt,
    surface: impl PreMeshableSurface,
    polyline: &PolyBoundary,
    tol: f64,
) {
    let bdb: BoundingBox<Point2> = polyline.0.iter().flatten().collect();
    let range = ((bdb.min()[0], bdb.max()[0]), (bdb.min()[1], bdb.max()[1]));
    let (udiv, vdiv) = surface.parameter_division(range, tol);
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
) -> PolygonMesh {
    let mut positions = Vec::<Point3>::new();
    let mut uv_coords = Vec::<Vector2>::new();
    let mut normals = Vec::<Vector3>::new();
    let vmap: HashMap<_, _> = vertices
        .enumerate()
        .map(|(i, v)| {
            let p = *v.as_ref();
            let uv = Vector2::new(p.x, p.y);
            positions.push(surface.subs(uv[0], uv[1]));
            uv_coords.push(uv);
            normals.push(surface.normal(uv[0], uv[1]));
            (v.fix(), i)
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
    p: Point2,
    q: Point2,
    tol: f64,
) -> Vec<Point2> {
    use truck_geometry::prelude::*;
    let line = Line(p, q);
    let pcurve = PCurve::new(line, surface);
    let (vec, _) = pcurve.parameter_division(pcurve.range_tuple(), tol);
    vec.into_iter().map(|t| line.subs(t)).collect()
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
        let _shell = shell_tessellation(&shell, 0.01, by_search_parameter);
    });
    println!("{}ms", instant.elapsed().as_millis());

    let instant = Instant::now();
    (0..100).for_each(|_| {
        let _shell = shell_tessellation_single_thread(&shell, 0.01, by_search_parameter);
    });
    println!("{}ms", instant.elapsed().as_millis());
}
