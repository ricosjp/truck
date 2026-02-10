use algo::curve::search_closest_parameter;
use itertools::Itertools;
use truck_geometry::prelude::*;

use super::geometry::*;
use super::params::FilletProfile;
use super::types::*;

pub(super) fn find_adjacent_edge(face: &Face, edge_id: EdgeID) -> Option<(Edge, Edge)> {
    face.boundary_iters()
        .into_iter()
        .flat_map(|boundary_iter| boundary_iter.circular_tuple_windows())
        .find(|(_, edge, _)| edge.id() == edge_id)
        .map(|(x, _, y)| (x, y))
}

#[allow(dead_code)]
pub(super) fn take_ori<T>(ori: bool, (a, b): (T, T)) -> T {
    match ori {
        true => a,
        false => b,
    }
}

pub(super) fn cut_face_by_bezier(
    face: &Face,
    mut bezier: NurbsCurve<Vector4>,
    filleted_edge_id: EdgeID,
) -> Option<(Face, Edge)> {
    let (front_edge, back_edge) = find_adjacent_edge(face, filleted_edge_id)?;

    let new_front_edge = {
        let curve = front_edge.curve();
        let hint = algo::curve::presearch_closest_point(
            &bezier,
            &curve,
            (bezier.range_tuple(), curve.range_tuple()),
            10,
        );
        let (t0, t1) = search_closest_parameter(&bezier, &curve, hint, 100)?;
        let v0 = Vertex::new(bezier.subs(t0));
        bezier = bezier.cut(t0);
        front_edge.not_strictly_cut_with_parameter(&v0, t1)?.0
    };

    let new_back_edge = {
        let curve = back_edge.curve();
        let hint = algo::curve::presearch_closest_point(
            &bezier,
            &curve,
            (bezier.range_tuple(), curve.range_tuple()),
            10,
        );
        let (t0, t1) = search_closest_parameter(&bezier, &curve, hint, 100)?;
        let v1 = Vertex::new(bezier.subs(t0));
        bezier.cut(t0);
        back_edge.not_strictly_cut_with_parameter(&v1, t1)?.1
    };

    let fillet_edge = Edge::new(new_front_edge.back(), new_back_edge.front(), bezier.into());
    let new_boundaries = face
        .absolute_boundaries()
        .iter()
        .cloned()
        .map(|mut boundary| {
            if let Some(idx) = boundary.iter().position(|edge0| edge0.is_same(&front_edge)) {
                let len = boundary.len();
                if face.orientation() {
                    boundary[idx] = new_front_edge.clone();
                    boundary[(idx + 1) % len] = fillet_edge.clone();
                    boundary[(idx + 2) % len] = new_back_edge.clone();
                } else {
                    boundary[(len + idx - 2) % len] = new_back_edge.inverse();
                    boundary[(len + idx - 1) % len] = fillet_edge.inverse();
                    boundary[idx] = new_front_edge.inverse();
                }
            }
            boundary
        })
        .collect::<Vec<_>>();
    let mut new_face = Face::new(new_boundaries, face.surface());
    if !face.orientation() {
        new_face.invert();
    }
    Some((new_face, fillet_edge))
}

pub(super) fn create_pcurve_edge(
    (v0, hint0): (&Vertex, (f64, f64)),
    (v1, hint1): (&Vertex, (f64, f64)),
    fillet_surface: &NurbsSurface<Vector4>,
) -> Option<Edge> {
    let uv0 = fillet_surface.search_parameter(v0.point(), hint0, 100)?;
    let uv1 = fillet_surface.search_parameter(v1.point(), hint1, 100)?;
    let curve = PCurve::new(Line(uv0.into(), uv1.into()), fillet_surface.clone());
    Some(Edge::new(v0, v1, curve.into()))
}

pub(super) fn create_new_side(
    side: &Face,
    fillet_edge: &Edge,
    corner_vertex_id: VertexID,
    left_face_front_edge: &Edge,
    right_face_back_edge: &Edge,
) -> Option<Face> {
    let (boundary_idx, edge_idx) = side.boundary_iters().into_iter().enumerate().find_map(
        |(boundary_idx, boundary_iter)| {
            boundary_iter
                .enumerate()
                .find(|(_, edge)| edge.back().id() == corner_vertex_id)
                .map(move |(edge_idx, _)| (boundary_idx, edge_idx))
        },
    )?;
    let new_boundaries = side
        .absolute_boundaries()
        .iter()
        .enumerate()
        .map(|(idx, boundary)| {
            let mut new_boundary = boundary.clone();
            if idx == boundary_idx {
                let len = new_boundary.len();
                if side.orientation() {
                    new_boundary[edge_idx] = right_face_back_edge.inverse();
                    new_boundary[(edge_idx + 1) % len] = left_face_front_edge.inverse();
                    new_boundary.insert(edge_idx + 1, fillet_edge.inverse());
                } else {
                    new_boundary[len - edge_idx - 1] = right_face_back_edge.clone();
                    new_boundary[(2 * len - edge_idx - 2) % len] = left_face_front_edge.clone();
                    new_boundary.insert(len - edge_idx, fillet_edge.clone());
                }
            }
            new_boundary
        })
        .collect();
    let side_surface = Box::new(side.oriented_surface());
    let Curve::PCurve(fillet_curve) = fillet_edge.curve() else {
        return None;
    };
    let fillet_surface = Box::new(fillet_curve.surface().clone());
    let new_curve = IntersectionCurve::new(side_surface, fillet_surface, fillet_curve);
    fillet_edge.set_curve(new_curve.into());
    let mut new_face = Face::new(new_boundaries, side.surface());
    if !side.orientation() {
        new_face.invert();
    }
    Some(new_face)
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FaceBoundaryEdgeIndex {
    pub(super) face_index: usize,
    pub(super) boundary_index: usize,
    pub(super) edge_index: usize,
}

impl From<(usize, usize, usize)> for FaceBoundaryEdgeIndex {
    fn from((face_index, boundary_index, edge_index): (usize, usize, usize)) -> Self {
        Self {
            face_index,
            boundary_index,
            edge_index,
        }
    }
}

pub(super) fn find_shared_face_with_front_edge(
    shell: &Shell,
    wire: &Wire,
) -> Option<FaceBoundaryEdgeIndex> {
    shell.iter().enumerate().find_map(|(face_idx, face)| {
        let mut boundary_iter = face.boundary_iters().into_iter().enumerate();
        boundary_iter.find_map(|(boundary_idx, boundary_iter)| {
            let mut edge_iter = boundary_iter.circular_tuple_windows().enumerate();
            edge_iter.find_map(|(edge_idx, (edge0, edge1))| {
                match edge0.is_same(&wire[0]) && edge1.is_same(&wire[1]) {
                    true => Some((face_idx, boundary_idx, edge_idx).into()),
                    false => None,
                }
            })
        })
    })
}

pub(super) fn enumerate_adjacent_faces(
    shell: &Shell,
    wire: &Wire,
    shared_face: FaceBoundaryEdgeIndex,
) -> Option<Vec<FaceBoundaryEdgeIndex>> {
    let iter = wire.edge_iter().map(|edge0| {
        shell.iter().enumerate().find_map(|(face_idx, face)| {
            if face_idx == shared_face.face_index {
                return None;
            }
            let mut boundary_iter = face.boundary_iters().into_iter().enumerate();
            boundary_iter.find_map(|(boundary_idx, boundary_iter)| {
                let mut edge_iter = boundary_iter.enumerate();
                edge_iter.find_map(|(edge_idx, edge)| match edge.is_same(edge0) {
                    true => Some((face_idx, boundary_idx, edge_idx).into()),
                    false => None,
                })
            })
        })
    });
    iter.collect()
}

pub(super) fn fillet_surfaces_along_wire(
    shell: &Shell,
    wire: &Wire,
    shared_face_index: FaceBoundaryEdgeIndex,
    adjacent_faces: &[FaceBoundaryEdgeIndex],
    radius: impl Fn(f64) -> f64,
    fillet_division: usize,
    profile: &FilletProfile,
) -> Option<Vec<NurbsSurface<Vector4>>> {
    let wire_faces_iter = wire.edge_iter().zip(adjacent_faces);
    let create_fillet_surface = |(edge, face_index): (&Edge, &FaceBoundaryEdgeIndex)| {
        let surface0 = &shell[shared_face_index.face_index].oriented_surface();
        let surface1 = &shell[face_index.face_index].oriented_surface();
        let curve = &edge.oriented_curve();
        let (first_wire, last_wire) = if wire.is_closed() {
            (false, false)
        } else {
            (
                edge.id() == wire.front_edge().unwrap().id(),
                edge.id() == wire.back_edge().unwrap().id(),
            )
        };
        let extend = first_wire || last_wire;
        let mut rs = relay_spheres(surface0, surface1, curve, fillet_division, &radius, extend)?;
        if first_wire {
            rs.pop();
        }
        if last_wire {
            rs.remove(0);
        }
        let surface = match profile {
            FilletProfile::Round => expand_fillet(&rs, surface0, surface1),
            FilletProfile::Chamfer => expand_chamfer(&rs, surface0, surface1),
            FilletProfile::Ridge => expand_ridge(&rs, surface0, surface1),
            FilletProfile::Custom(curve) => expand_custom(&rs, surface0, surface1, curve),
        };
        Some(surface)
    };
    wire_faces_iter.map(create_fillet_surface).collect()
}

pub(super) fn concat_fillet_surface(surfaces: &[NurbsSurface<Vector4>]) -> NurbsSurface<Vector4> {
    let len = surfaces[0].control_points().len();
    let concat_beziers = |i: usize| {
        let mut collector = CurveCollector::<NurbsCurve<Vector4>>::Singleton;
        (0..surfaces.len()).for_each(|n| {
            let mut curve = surfaces[n].column_curve(i);
            curve.knot_translate(n as f64);
            collector.concat(&curve);
        });
        collector.unwrap()
    };
    let long_beziers = (0..len).map(concat_beziers).collect::<Vec<_>>();

    let uknot_vec = surfaces[0].uknot_vec().clone();
    let vknot_vec = long_beziers[0].knot_vec().clone();
    let destruct_bezier = |bezier: NurbsCurve<Vector4>| bezier.into_non_rationalized().destruct().1;
    let control_points = long_beziers.into_iter().map(destruct_bezier).collect();
    NurbsSurface::new(BSplineSurface::new((uknot_vec, vknot_vec), control_points))
}

pub(super) fn create_free_edge(curve: Curve) -> Edge {
    let v0 = Vertex::new(curve.front());
    let v1 = Vertex::new(curve.back());
    Edge::new(&v0, &v1, curve)
}

pub(super) fn cut_face_by_last_bezier(
    shell: &mut Shell,
    face_index: FaceBoundaryEdgeIndex,
    fillet_surface: &NurbsSurface<Vector4>,
) -> Option<Edge> {
    let len = fillet_surface.control_points().len();
    let last_long_bezier = fillet_surface.column_curve(len - 1);
    let face = &shell[face_index.face_index];
    let filleted_edge = &face.boundaries()[face_index.boundary_index][face_index.edge_index];
    let (trimmed_face, edge1) =
        cut_face_by_bezier(face, last_long_bezier.inverse(), filleted_edge.id())?;
    shell[face_index.face_index] = trimmed_face;
    Some(edge1)
}
