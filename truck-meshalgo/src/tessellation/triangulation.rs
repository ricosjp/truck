#![allow(clippy::many_single_char_names)]

use super::*;
use crate::filters::NormalFilters;
use rustc_hash::FxHashMap as HashMap;

type Cdt<V, K> = ConstrainedDelaunayTriangulation<V, K>;
type MeshedShell = Shell<Point3, PolylineCurve, PolygonMesh>;

/// Tessellates faces
pub(super) fn tessellation<'a, C, S>(shell: &Shell<Point3, C, S>, tol: f64) -> Option<MeshedShell>
where
    C: PolylineableCurve + 'a,
    S: MeshableSurface + 'a, {
    let mut vmap = HashMap::default();
    let mut edge_map = HashMap::default();
    shell
        .face_iter()
        .map(|face| {
            let wires: Vec<_> = face
                .absolute_boundaries()
                .iter()
                .map(|wire| {
                    wire.edge_iter()
                        .map(|edge| {
                            let new_edge = edge_map.entry(edge.id()).or_insert_with(|| {
                                let vf = edge.absolute_front();
                                let v0 = vmap
                                    .entry(vf.id())
                                    .or_insert_with(|| vf.mapped(Point3::clone))
                                    .clone();
                                let vb = edge.absolute_back();
                                let v1 = vmap
                                    .entry(vb.id())
                                    .or_insert_with(|| vb.mapped(Point3::clone))
                                    .clone();
                                let curve = edge.get_curve();
                                let poly =
                                    PolylineCurve::from_curve(&curve, curve.parameter_range(), tol);
                                Edge::debug_new(&v0, &v1, poly)
                            });
                            match edge.orientation() {
                                true => new_edge.clone(),
                                false => new_edge.inverse(),
                            }
                        })
                        .collect()
                })
                .collect();
            let surface = face.get_surface();
            let mut polyline = Polyline::default();
            let polygon = match wires.iter().all(|wire| polyline.add_wire(&surface, wire)) {
                true => Some(trimming_tessellation(&surface, &polyline, tol)),
                false => None,
            }?;
            let mut new_face = Face::debug_new(wires, polygon);
            if !face.orientation() {
                new_face.invert();
            }
            Some(new_face)
        })
        .collect()
}

/// polyline, not always connected
#[derive(Debug, Default, Clone)]
struct Polyline {
    positions: Vec<Point2>,
    indices: Vec<[usize; 2]>,
}

impl Polyline {
    /// add an wire into polyline
    fn add_wire<S>(&mut self, surface: &S, wire: &Wire<Point3, PolylineCurve>) -> bool
    where S: MeshableSurface {
        let mut counter = 0;
        let len = self.positions.len();
        let res = wire.into_iter().all(|edge| {
            let mut poly_edge = edge.oriented_curve();
            poly_edge.pop();
            counter += poly_edge.len();
            let mut hint = None;
            Vec::from(poly_edge).into_iter().all(|pt| {
                hint = surface
                    .search_parameter(pt, hint, 100)
                    .or_else(|| surface.search_parameter(pt, None, 100));
                hint.map(|hint| self.positions.push(hint.into())).is_some()
            })
        });
        self.indices
            .extend((0..counter).map(|i| [len + i, len + (i + 1) % counter]));
        res
    }

    /// whether `c` is included in the domain with bounday = `self`.
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
    fn insert_to(&self, triangulation: &mut Cdt<[f64; 2], impl DelaunayKernel<f64>>) {
        let poly2tri: Vec<usize> = self
            .positions
            .iter()
            .map(|pt| triangulation.insert((*pt).into()))
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
where S: MeshableSurface {
    let mut triangulation = Cdt::<[f64; 2], FloatKernel>::new();
    polyline.insert_to(&mut triangulation);
    insert_surface(&mut triangulation, surface, polyline, tol);
    let mut mesh = triangulation_into_polymesh(
        triangulation.vertices(),
        triangulation.triangles(),
        surface,
        polyline,
    );
    mesh.make_face_compatible_to_normal();
    mesh
}

/// Inserts parameter divisions into triangulation.
fn insert_surface(
    triangulation: &mut Cdt<[f64; 2], impl DelaunayKernel<f64>>,
    surface: &impl MeshableSurface,
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
            triangulation.insert(pt.into());
        });
}

/// Converts triangulation into `PolygonMesh`.
fn triangulation_into_polymesh<'a>(
    vertices: impl Iterator<Item = VertexHandle<'a, [f64; 2], CdtEdge>>,
    triangles: impl Iterator<Item = FaceHandle<'a, [f64; 2], CdtEdge>>,
    surface: &impl ParametricSurface3D,
    polyline: &Polyline,
) -> PolygonMesh {
    let mut positions = Vec::<Point3>::new();
    let mut uv_coords = Vec::<Vector2>::new();
    let mut normals = Vec::<Vector3>::new();
    let vmap: HashMap<usize, usize> = vertices
        .enumerate()
        .map(|(i, v)| {
            let uv = Vector2::from(*v);
            positions.push(surface.subs(uv[0], uv[1]));
            uv_coords.push(uv);
            normals.push(surface.normal(uv[0], uv[1]));
            (v.fix(), i)
        })
        .collect();
    let tri_faces: Vec<[StandardVertex; 3]> = triangles
        .map(|tri| tri.as_triangle())
        .filter(|tri| {
            let c = Point2::new(
                (tri[0][0] + tri[1][0] + tri[2][0]) / 3.0,
                (tri[0][1] + tri[1][1] + tri[2][1]) / 3.0,
            );
            polyline.include(c)
        })
        .map(|tri| {
            let idcs = [
                vmap[&tri[0].fix()],
                vmap[&tri[1].fix()],
                vmap[&tri[2].fix()],
            ];
            [
                [idcs[0], idcs[0], idcs[0]].into(),
                [idcs[1], idcs[1], idcs[1]].into(),
                [idcs[2], idcs[2], idcs[2]].into(),
            ]
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
