#![allow(clippy::many_single_char_names)]

use super::*;
use crate::filters::NormalFilters;
use crate::Point2;
use array_macro::array;
use rustc_hash::FxHashMap as HashMap;
use truck_base::entry_map::FxEntryMap as EntryMap;
use truck_topology::Vertex as TVertex;

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
        let surface = face.surface();
        let mut polyline = Polyline::default();
        let polygon = match wires.iter().all(|wire: &Wire<_, _>| {
            polyline.add_wire(&surface, wire.iter().map(Edge::oriented_curve), &sp)
        }) {
            true => Some(trimming_tessellation(&surface, &polyline, tol)),
            false => None,
        };
        let mut new_face = Face::debug_new(wires, polygon);
        if !face.orientation() {
            new_face.invert();
        }
        new_face
    };
    shell.face_par_iter().map(create_face).collect()
}

/// Tessellates faces
#[allow(dead_code)]
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
        let surface = face.surface();
        let mut polyline = Polyline::default();
        let polygon = match wires.iter().all(|wire: &Wire<_, _>| {
            polyline.add_wire(&surface, wire.iter().map(|edge| edge.oriented_curve()), &sp)
        }) {
            true => Some(trimming_tessellation(&surface, &polyline, tol)),
            false => None,
        };
        let mut new_face = Face::debug_new(wires, polygon);
        if !face.orientation() {
            new_face.invert();
        }
        new_face
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
            use std::ops::Bound;
            let surface = &face.surface;
            fn range_tuple<T>(range: (Bound<T>, Bound<T>)) -> Option<(T, T)> {
                match range {
                    (Bound::Included(x), Bound::Included(y)) => Some((x, y)),
                    (Bound::Excluded(x), Bound::Included(y)) => Some((x, y)),
                    (Bound::Included(x), Bound::Excluded(y)) => Some((x, y)),
                    (Bound::Excluded(x), Bound::Excluded(y)) => Some((x, y)),
                    _ => None,
                }
            }
            let (urange, vrange) = surface.parameter_range();
            let surface = match (range_tuple(urange), range_tuple(vrange)) {
                (Some(urange), Some(vrange)) => {
                    Some(StructuredMesh::from_surface(surface, (urange, vrange), tol).destruct())
                }
                _ => None,
            };
            CompressedFace {
                boundaries: Vec::new(),
                orientation: face.orientation,
                surface,
            }
        } else {
            let boundaries = face.boundaries.clone();
            let surface = &face.surface;
            let mut polyline = Polyline::default();
            let polygon = match boundaries.iter().all(|wire| {
                let wire_iter = wire
                    .iter()
                    .filter_map(|edge_idx| match edge_idx.orientation {
                        true => Some(edges.get(edge_idx.index)?.curve.clone()),
                        false => Some(edges.get(edge_idx.index)?.curve.inverse()),
                    });
                polyline.add_wire(surface, wire_iter, &sp)
            }) {
                true => Some(trimming_tessellation(surface, &polyline, tol)),
                false => None,
            };
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

/// polyline, not always connected
#[derive(Debug, Default, Clone)]
struct Polyline {
    positions: Vec<Point2>,
    indices: Vec<[usize; 2]>,
}

impl Polyline {
    /// add an wire into polyline
    fn add_wire<S>(
        &mut self,
        surface: &S,
        mut wire: impl Iterator<Item = PolylineCurve>,
        sp: impl Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)>,
    ) -> bool
    where
        S: PreMeshableSurface,
    {
        let mut counter = 0;
        let mut previous: Option<(f64, f64)> = None;
        let len = self.positions.len();
        let up = surface.u_period();
        let vp = surface.v_period();
        let res = wire.all(|mut poly_edge| {
            poly_edge.pop();
            counter += poly_edge.len();
            let mut hint = None;
            Vec::from(poly_edge).into_iter().all(|pt| {
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
                hint.map(|hint| self.positions.push(hint.into())).is_some()
            })
        });
        self.indices
            .extend((0..counter).map(|i| [len + i, len + (i + 1) % counter]));
        res
    }

    /// whether `c` is included in the domain with boundary = `self`.
    fn include(&self, c: Point2) -> bool {
        let t = 2.0 * std::f64::consts::PI * HashGen::hash1(c);
        let r = Vector2::new(f64::cos(t), f64::sin(t));
        self.indices
            .iter()
            .try_fold(0_i32, move |counter, edge| {
                let a = self.positions[edge[0]] - c;
                let b = self.positions[edge[1]] - c;
                let s0 = r[0] * a[1] - r[1] * a[0]; // v times a
                let s1 = r[0] * b[1] - r[1] * b[0]; // v times b
                let s2 = a[0] * b[1] - a[1] * b[0]; // a times b
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
            .positions
            .iter()
            .filter_map(|pt| triangulation.insert(SPoint2::from([pt.x, pt.y])).ok())
            .collect();
        let mut prev: Option<usize> = None;
        self.indices.iter().for_each(|a| {
            if let Some(p) = prev {
                if triangulation.can_add_constraint(poly2tri[p], poly2tri[a[1]]) {
                    triangulation.add_constraint(poly2tri[p], poly2tri[a[1]]);
                    prev = None;
                }
            } else if triangulation.can_add_constraint(poly2tri[a[0]], poly2tri[a[1]]) {
                triangulation.add_constraint(poly2tri[a[0]], poly2tri[a[1]]);
            } else {
                prev = Some(a[0]);
            }
        });
    }
}

/// Tessellates one surface trimmed by polyline.
fn trimming_tessellation<S>(surface: &S, polyline: &Polyline, tol: f64) -> PolygonMesh
where S: PreMeshableSurface {
    let mut triangulation = Cdt::new();
    polyline.insert_to(&mut triangulation);
    insert_surface(&mut triangulation, surface, polyline, tol);
    let mut mesh = triangulation_into_polymesh(
        triangulation.vertices(),
        triangulation.inner_faces(),
        surface,
        polyline,
    );
    mesh.make_face_compatible_to_normal();
    mesh
}

/// Inserts parameter divisions into triangulation.
fn insert_surface(
    triangulation: &mut Cdt,
    surface: &impl PreMeshableSurface,
    polyline: &Polyline,
    tol: f64,
) {
    let bdb: BoundingBox<Point2> = polyline.positions.iter().collect();
    let range = ((bdb.min()[0], bdb.max()[0]), (bdb.min()[1], bdb.max()[1]));
    let (udiv, vdiv) = surface.parameter_division(range, tol);
    udiv.into_iter()
        .flat_map(|u| vdiv.iter().map(move |v| Point2::new(u, *v)))
        .filter(|pt| polyline.include(*pt))
        .for_each(|pt| {
            let _ = triangulation.insert(SPoint2::from([pt.x, pt.y]));
        });
}

/// Converts triangulation into `PolygonMesh`.
fn triangulation_into_polymesh<'a>(
    vertices: VertexIterator<'a, SPoint2, (), CdtEdge<()>, ()>,
    triangles: InnerFaceIterator<'a, SPoint2, (), CdtEdge<()>, ()>,
    surface: &impl ParametricSurface3D,
    polyline: &Polyline,
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
            fn sp2cg(x: SPoint2) -> Point2 { Point2::from(<[f64; 2]>::from(x)) }
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
