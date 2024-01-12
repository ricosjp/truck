#![allow(dead_code)]

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use truck_geometry::prelude::*;
use truck_meshalgo::rexport_polymesh::*;
use truck_topology::compress::*;

type Edge<C> = CompressedEdge<C>;
type EdgeIndex = CompressedEdgeIndex;
type Wire = Vec<EdgeIndex>;
type Face<S> = CompressedFace<S>;
type Shell<P, C, S> = CompressedShell<P, C, S>;
type Solid<P, C, S> = CompressedSolid<P, C, S>;

trait SP<S>: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> {}
impl<S, F> SP<S> for F where F: Fn(&S, Point3, Option<(f64, f64)>) -> Option<(f64, f64)> {}

/// Cuts the edge at the middle point of the bounded curve `curve`.
fn cut_edge<P, C>(
    vertices: &mut Vec<P>,
    Edge {
        vertices: (_, v1),
        curve,
    }: &mut Edge<C>,
) -> Edge<C>
where
    C: BoundedCurve<Point = P> + Cut,
{
    let (t0, t1) = curve.range_tuple();
    let t = (t0 + t1) / 2.0;
    vertices.push(curve.subs(t));

    let v2 = *v1;
    *v1 = vertices.len() - 1;
    let v1 = *v1;
    let curve0 = curve.cut(t);
    Edge {
        vertices: (v1, v2),
        curve: curve0,
    }
}

/// Adds cut edges to the vector `edges`.
fn add_edges<P, C>(vertices: &mut Vec<P>, edges: &mut Vec<Edge<C>>) -> HashMap<usize, usize>
where C: BoundedCurve<Point = P> + Cut {
    let len = edges.len();
    let sub_add_edges = move |i: usize| {
        let (v0, v1) = edges[i].vertices;
        if v0 == v1 {
            let new_edge = cut_edge(vertices, &mut edges[i]);
            edges.push(new_edge);
            Some((i, edges.len() - 1))
        } else {
            None
        }
    };
    (0..len).filter_map(sub_add_edges).collect()
}

fn replace_edges(wire: &mut Wire, added: &HashMap<usize, usize>) {
    let insert_one_edge = |edge: EdgeIndex| {
        let Some(new_edge_index) = added.get(&edge.index) else {
            return vec![edge];
        };
        let new_edge = EdgeIndex {
            index: *new_edge_index,
            orientation: edge.orientation,
        };
        match edge.orientation {
            true => vec![edge, new_edge],
            false => vec![new_edge, edge],
        }
    };
    let new_wire = wire.iter().copied().flat_map(insert_one_edge).collect();
    *wire = new_wire;
}

fn split_closed_edges<P, C, S>(shell: &mut Shell<P, C, S>)
where C: BoundedCurve<Point = P> + Cut {
    let added = add_edges(&mut shell.vertices, &mut shell.edges);
    let wire_iter = shell.faces.iter_mut().flat_map(|face| &mut face.boundaries);
    wire_iter.for_each(|wire| replace_edges(wire, &added));
}

fn is_simple(wire: &Wire) -> bool {
    let mut set = HashSet::default();
    wire.iter().all(|edge| set.insert(edge.index))
}

fn split_nonsimple_wire<C>(
    boundary: &mut Wire,
    edges: &mut Vec<Edge<C>>,
    mut new_edge: impl FnMut(usize, usize) -> Option<Edge<C>>,
) -> Option<Vec<EdgeIndex>> {
    fn find_loop() -> impl FnMut((usize, &EdgeIndex)) -> Option<(usize, usize)> {
        let mut map = HashMap::<usize, usize>::default();
        move |(i, edge)| map.insert(edge.index, i).map(move |j| (j, i))
    }
    let (i0, j0) = boundary.iter().enumerate().find_map(find_loop())?;
    boundary.rotate_left(i0);
    let j0 = j0 - i0;

    let (i1, j1) = boundary
        .iter()
        .enumerate()
        .skip(j0 + 1)
        .find_map(find_loop())
        .unwrap_or((j0, boundary.len()));

    let (k0, k1) = ((i0 + j0 + 1) / 2, (i1 + j1 + 1) / 2);
    let edge_index0 = boundary[k0].index;
    let v0 = match boundary[k0].orientation {
        true => edges[edge_index0].vertices.0,
        false => edges[edge_index0].vertices.1,
    };
    let edge_index1 = boundary[k1].index;
    let v1 = match boundary[k1].orientation {
        true => edges[edge_index1].vertices.0,
        false => edges[edge_index1].vertices.1,
    };
    edges.push((&mut new_edge)(v0, v1)?);

    let back = boundary.split_off(k1);
    let mut middle = boundary.split_off(k0);

    let create_edge = |orientation: bool| EdgeIndex {
        index: edges.len() - 1,
        orientation,
    };
    boundary.push(create_edge(true));
    boundary.extend(back);
    middle.push(create_edge(false));
    Some(middle)
}

fn to_parametric_polyline<S: ParametricSurface3D>(
    surface: &S,
    wire: impl Iterator<Item = PolylineCurve<Point3>>,
    sp: impl SP<S>,
) -> Option<PolylineCurve<Point2>> {
    let up = surface.u_period();
    let vp = surface.v_period();
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
            fn abs_diff(previous: f64) -> impl Fn(&f64, &f64) -> std::cmp::Ordering {
                let f = move |x: &f64| f64::abs(x - previous);
                move |x: &f64, y: &f64| f(x).partial_cmp(&f(y)).unwrap()
            }
            if let (Some(up), Some((u0, _))) = (up, previous) {
                u = (-2..=2)
                    .map(|i| u + i as f64 * up)
                    .min_by(abs_diff(u0))
                    .unwrap();
            }
            if let (Some(vp), Some((_, v0))) = (vp, previous) {
                v = (-2..=2)
                    .map(|i| v + i as f64 * vp)
                    .min_by(abs_diff(v0))
                    .unwrap();
            }
            let res = (|| {
                if let Some((u0, v0)) = previous {
                    if !u0.near(&u) && surface.uder(u0, v0).so_small() {
                        return vec![Some(Point2::new(u, v0)), Some(Point2::new(u, v))];
                    } else if !v0.near(&v) && surface.vder(u0, v0).so_small() {
                        return vec![Some(Point2::new(u0, v)), Some(Point2::new(u, v))];
                    }
                }
                vec![Some(Point2::new(u, v))]
            })();
            previous = Some((u, v));
            res
        })
        .collect::<Option<Vec<Point2>>>()?;
    let last = *vec.last().unwrap();
    if !vec[0].near(&last) {
        let Point2 { x: u0, y: v0 } = last;
        if surface.uder(u0, v0).so_small() || surface.vder(u0, v0).so_small() {
            vec.push(vec[0]);
        }
    }
    Some(PolylineCurve(vec))
}

#[derive(Clone, Copy, Debug)]
enum BoundaryType {
    Positive,
    Negative,
    NotClosed,
}

fn boundary_type(poly: &PolylineCurve<Point2>) -> BoundaryType {
    if poly[0].distance2(poly[poly.len() - 1]) < 1.0e-3 {
        match poly.area() > 0.0 {
            true => BoundaryType::Positive,
            false => BoundaryType::Negative,
        }
    } else {
        BoundaryType::NotClosed
    }
}

fn assort_boundary<S>(
    surface: &S,
    boundaries: Vec<Wire>,
    poly_edges: &[PolylineCurve<Point3>],
    sp: impl SP<S>,
) -> Option<Vec<Vec<Wire>>>
where
    S: ParametricSurface3D,
{
    let get_polyline = move |EdgeIndex { index, orientation }: &EdgeIndex| match orientation {
        true => poly_edges[*index].clone(),
        false => poly_edges[*index].inverse(),
    };
    let (mut positives, mut negatives) = (Vec::new(), Vec::new());
    boundaries.into_iter().try_for_each(|boundary| {
        let wire = boundary.iter().map(get_polyline);
        let poly_boundary = to_parametric_polyline(surface, wire, &sp)?;
        match boundary_type(&poly_boundary) {
            BoundaryType::Positive => positives.push((boundary, poly_boundary)),
            BoundaryType::Negative => negatives.push((boundary, poly_boundary)),
            _ => return None,
        }
        Some(())
    })?;
    let mut assorted = positives.into_iter().map(|x| vec![x]).collect::<Vec<_>>();
    negatives
        .into_iter()
        .try_for_each(|(boundary, poly_boundary)| {
            let p = poly_boundary[0];
            let vec = assorted.iter_mut().find(|vec| vec[0].1.include(p))?;
            vec.push((boundary, poly_boundary));
            Some(())
        })?;
    let res = assorted
        .into_iter()
        .map(|vec| vec.into_iter().map(|(x, _)| x).collect())
        .collect();
    Some(res)
}

fn split_closed_face<C, S>(
    Face {
        boundaries,
        surface,
        orientation,
    }: &mut Face<S>,
    vertices: &[Point3],
    edges: &mut Vec<Edge<C>>,
    poly_edges: &mut Vec<PolylineCurve<Point3>>,
    sp: impl SP<S>,
    tol: f64,
) -> Option<Vec<Face<S>>>
where
    C: TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    let new_boundary = boundaries
        .iter_mut()
        .filter_map(|boundary| {
            let new_edge = |v0: usize, v1: usize| {
                let (p0, p1) = (vertices[v0], vertices[v1]);
                let (u0, u1) = (sp(surface, p0, None)?, sp(surface, p1, None)?);
                let pcurve = PCurve::new(Line(u0.into(), u1.into()), surface.clone());
                poly_edges.push(PolylineCurve::from_curve(&pcurve, (0.0, 1.0), tol));
                Some(Edge {
                    vertices: (v0, v1),
                    curve: C::try_from(pcurve).ok()?,
                })
            };
            split_nonsimple_wire(boundary, edges, new_edge)
        })
        .collect::<Vec<_>>();
    boundaries.extend(new_boundary);
    let mut vec = Vec::new();
    vec.append(boundaries);
    let mut assorted = assort_boundary(surface, vec, poly_edges, sp)?.into_iter();
    *boundaries = assorted.next()?;
    Some(
        assorted
            .map(|boundaries| Face {
                boundaries,
                surface: surface.clone(),
                orientation: *orientation,
            })
            .collect(),
    )
}

fn split_closed_faces<C, S>(
    Shell {
        vertices,
        edges,
        faces,
    }: &mut Shell<Point3, C, S>,
    tol: f64,
    sp: impl SP<S>,
) where
    C: BoundedCurve + ParameterDivision1D<Point = Point3> + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    let mut poly_edges = edges
        .iter()
        .map(|Edge { curve, .. }| PolylineCurve::from_curve(curve, curve.range_tuple(), tol))
        .collect::<Vec<_>>();
    let new_faces = faces
        .iter_mut()
        .filter_map(|face| split_closed_face(face, vertices, edges, &mut poly_edges, &sp, tol))
        .flatten()
        .collect::<Vec<_>>();
    faces.extend(new_faces);
}

fn closing_boundary<C, S>(
    Face {
        boundaries,
        surface,
        ..
    }: &mut Face<S>,
    poly_edges: &[PolylineCurve<Point3>],
    sp: impl SP<S>,
) -> Option<()>
where
    C: BoundedCurve<Point = Point3> + ParameterDivision1D<Point = Point3>,
    S: ParametricSurface3D,
{
    let get_polyline = move |EdgeIndex { index, orientation }: &EdgeIndex| match orientation {
        true => poly_edges[*index].clone(),
        false => poly_edges[*index].inverse(),
    };
    let _poly_boundaries = boundaries
        .iter()
        .map(|boundary| {
            let wire = boundary.iter().map(get_polyline);
            to_parametric_polyline(surface, wire, &sp)
        })
        .collect::<Vec<_>>();
    Some(())
}

#[cfg(test)]
mod tests;
