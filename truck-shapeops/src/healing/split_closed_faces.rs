use super::*;
use std::{convert::identity, ops::Range};

pub(super) fn split_closed_faces<C, S>(shell: &mut Shell<Point3, C, S>, tol: f64, sp: impl SP<S>)
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D, {
    let to_poly = closure_to_poly(tol);
    let mut poly_edges: Vec<_> = shell.edges.iter().map(to_poly).collect();
    let len = shell.faces.len();
    let split_closed_face0 = |i| {
        split_face_with_non_closed_boundary(i, shell, &mut poly_edges, &sp, tol);
    };
    (0..len).for_each(split_closed_face0);
    let len = shell.faces.len();
    let split_closed_face1 =
        |i| split_face_with_non_simple_wire(i, shell, &mut poly_edges, &sp, tol);
    let new_faces: Vec<_> = (0..len).filter_map(split_closed_face1).flatten().collect();
    shell.faces.extend(new_faces);
}

fn split_face_with_non_closed_boundary<C, S>(
    face_index: usize,
    shell: &mut Shell<Point3, C, S>,
    poly_edges: &mut Vec<PolylineCurve<Point3>>,
    sp: impl SP<S>,
    tol: f64,
) -> Option<()>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    let Face {
        boundaries,
        surface,
        ..
    } = &shell.faces[face_index];
    let (divisor, mut open, closed) = find_non_closed_wires_in_param_divisor(
        &boundaries,
        &shell.edges,
        poly_edges,
        surface,
        &sp,
    )?;
    take_vertices_to_intersections(divisor, face_index, shell, poly_edges, &sp, tol);
    let Shell {
        ref mut faces,
        ref mut edges,
        ..
    } = shell;
    let Face {
        ref mut boundaries,
        ref surface,
        ..
    } = faces[face_index];
    let param_boundaries = create_param_boundaries(boundaries, surface, poly_edges, &sp)?;
    let param_vertices = create_param_vertices(boundaries, &param_boundaries, edges);
    let vertices_on_divisor = enumerate_vertices_on_divisor(divisor, &param_vertices)?;
    let new_edges = create_new_edges(&vertices_on_divisor, poly_edges, surface, tol)?;
    let (new_edge_range, new_edge_indices) = signup_new_edges(edges, new_edges);
    let edge_iter = new_edge_indices.chain(closed.iter().flatten().copied());
    let vemap = create_vemap(edges, edge_iter);
    let mut new_boundaries = construct_boundaries(vemap, edges, new_edge_range);
    let (v0, v1) = divisor;
    let take_front = closure_take_front(edges);
    let (i, mut open_v0) = open.iter().find_map(|wire| {
        let index = wire
            .iter()
            .position(|edge_index| take_front(*edge_index) == v0);
        index.map(|index| (index, wire.clone()))
    })?;
    open_v0.rotate_left(i);
    let (j, mut open_v1) = open.iter_mut().find_map(|wire| {
        let index = wire
            .iter()
            .position(|edge_index| take_front(*edge_index) == v1);
        index.map(|index| (index, wire.clone()))
    })?;
    open_v1.rotate_left(j);
    let (k, added_wire) = new_boundaries.iter_mut().find_map(|wire| {
        let index = wire
            .iter()
            .position(|edge_index| take_front(*edge_index) == v0);
        index.map(|index| (index, wire))
    })?;
    added_wire.rotate_left(k);
    open_v0.append(added_wire);
    let l = open_v0
        .iter()
        .position(|edge_index| take_front(*edge_index) == v1)?;
    open_v0.rotate_left(l);
    open_v0.append(&mut open_v1);
    *added_wire = open_v0.clone();
    *boundaries = new_boundaries;
    Some(())
}

fn split_face_with_non_simple_wire<C, S>(
    face_index: usize,
    shell: &mut Shell<Point3, C, S>,
    poly_edges: &mut Vec<PolylineCurve<Point3>>,
    sp: impl SP<S>,
    tol: f64,
) -> Option<Vec<Face<S>>>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    let Face { boundaries, .. } = &shell.faces[face_index];
    let divisor = find_non_simple_wire_divisor(boundaries, &shell.edges)?;
    split_face_by_divisor(face_index, divisor, shell, poly_edges, sp, tol)
}

// --- find_non_simple_wire_divisor ---

fn find_non_simple_wire_divisor<C>(
    boundaries: &[Wire],
    edges: &[Edge<C>],
) -> Option<(usize, usize)> {
    let closure = |boundary| non_simple_wire_divisor(boundary, edges);
    boundaries.iter().cloned().find_map(closure)
}

fn non_simple_wire_divisor<C>(mut boundary: Wire, edges: &[Edge<C>]) -> Option<(usize, usize)> {
    let (i0, j0) = boundary.iter().enumerate().find_map(find_loop())?;
    boundary.rotate_left(i0);
    let j0 = j0 - i0;

    let (i1, j1) = boundary
        .iter()
        .enumerate()
        .skip(j0 + 1)
        .find_map(find_loop())
        .unwrap_or((j0, boundary.len()));

    let (k0, k1) = ((j0 + 1) / 2, (i1 + j1 + 1) / 2);
    let f = closure_take_front(edges);
    Some((f(boundary[k0]), f(boundary[k1])))
}

fn find_loop() -> impl FnMut((usize, &EdgeIndex)) -> Option<(usize, usize)> {
    let mut map = HashMap::<usize, usize>::default();
    move |(i, edge)| map.insert(edge.index, i).map(move |j| (j, i))
}

// --- find_non_closed_wires_divisor ---

fn find_non_closed_wires_in_param_divisor<'a, C, S>(
    boundaries: &'a [Wire],
    edges: &'a [Edge<C>],
    poly_edges: &'a [PolylineCurve<Point3>],
    surface: &'a S,
    sp: impl SP<S>,
) -> Option<((usize, usize), Vec<Wire>, Vec<Wire>)>
where
    C: ParametricCurve3D + BoundedCurve + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    let param_boundaries = create_param_boundaries(boundaries, surface, poly_edges, sp)?;
    let mut open = Vec::new();
    let mut closed = Vec::new();
    boundaries
        .iter()
        .zip(&param_boundaries)
        .for_each(|(boundary, param_boundary)| {
            let poly = PolylineCurve(param_boundary.iter().flatten().copied().collect());
            match boundary_type(&poly) {
                BoundaryType::NotClosed => open.push((boundary, param_boundary)),
                _ => closed.push((boundary, param_boundary)),
            }
        });
    if open.len() != 2 {
        return None;
    }
    let divisor_vec = open
        .iter()
        .map(|(wire, poly_wire)| {
            let (urange, vrange) = surface.try_range_tuple();
            let (up, vp) = (surface.u_period(), surface.v_period());
            let take_front = closure_take_front(edges);
            let p = poly_wire[0][0];
            let q = *poly_wire.last().unwrap().last().unwrap();
            let vertices: Vec<(usize, f64)> = if p.x.near(&q.x) {
                if let (Some(vp), Some((v0, _))) = (vp, vrange) {
                    Some(
                        wire.iter()
                            .zip(*poly_wire)
                            .map(|(edge_index, poly)| {
                                let v = poly[0].y;
                                let closure = move |i| {
                                    let v = v + i as f64 * vp;
                                    f64::abs(v - v0)
                                };
                                let dist = (-2..=2)
                                    .map(closure)
                                    .min_by(|x, y| x.partial_cmp(&y).unwrap())
                                    .unwrap();
                                (take_front(*edge_index), dist)
                            })
                            .collect(),
                    )
                } else {
                    None
                }
            } else if p.y.near(&q.y) {
                if let (Some(up), Some((u0, _))) = (up, urange) {
                    Some(
                        wire.iter()
                            .zip(*poly_wire)
                            .map(|(edge_index, poly)| {
                                let u = poly[0].x;
                                let closure = move |i| {
                                    let u = u + i as f64 * up;
                                    f64::abs(u - u0)
                                };
                                let dist = (-2..=2)
                                    .map(closure)
                                    .min_by(|x, y| x.partial_cmp(&y).unwrap())
                                    .unwrap();
                                (take_front(*edge_index), dist)
                            })
                            .collect(),
                    )
                } else {
                    None
                }
            } else {
                None
            }?;
            vertices
                .into_iter()
                .min_by(|(_, x), (_, y)| x.partial_cmp(&y).unwrap())
                .map(|(v, _)| v)
        })
        .collect::<Option<Vec<_>>>()?;
    let open = open
        .into_iter()
        .map(|(w, _)| w.iter().copied().collect())
        .collect();
    let closed = closed
        .into_iter()
        .map(|(w, _)| w.iter().copied().collect())
        .collect();
    Some(((divisor_vec[0], divisor_vec[1]), open, closed))
}

// --- split_face_by_divisor ---

fn split_face_by_divisor<C, S>(
    face_index: usize,
    divisor: (usize, usize),
    shell: &mut Shell<Point3, C, S>,
    poly_edges: &mut Vec<PolylineCurve<Point3>>,
    sp: impl SP<S>,
    tol: f64,
) -> Option<Vec<Face<S>>>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + ParameterDivision1D<Point = Point3>
        + SearchNearestParameter<D1, Point = Point3>
        + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    take_vertices_to_intersections(divisor, face_index, shell, poly_edges, &sp, tol);
    sub_split_face_by_divisor(face_index, divisor, shell, poly_edges, sp, tol)
}

// --- take_vertices_to_intersections ---

fn take_vertices_to_intersections<C, S>(
    divisor: (usize, usize),
    face_index: usize,
    Shell {
        vertices,
        edges,
        faces,
    }: &mut Shell<Point3, C, S>,
    poly_edges: &mut Vec<PolylineCurve<Point3>>,
    sp: impl SP<S>,
    tol: f64,
) -> Option<()>
where
    C: ParametricCurve3D
        + BoundedCurve
        + Cut
        + SearchNearestParameter<D1, Point = Point3>
        + ParameterDivision1D<Point = Point3>,
    S: ParametricSurface3D,
{
    let Face {
        ref boundaries,
        ref surface,
        ..
    } = faces[face_index];
    let param_boundaries = create_param_boundaries(boundaries, surface, poly_edges, &sp)?;
    let (v0, v1) = divisor;
    let p0 = zip_boundaries(boundaries, &param_boundaries)
        .filter_map(closure_find_vertex_parameter(v0, edges))
        .collect::<Vec<_>>();
    let p1 = zip_boundaries(boundaries, &param_boundaries)
        .filter_map(closure_find_vertex_parameter(v1, edges))
        .collect::<Vec<_>>();
    let nearest_points = p0
        .into_iter()
        .map(|p| {
            p1.iter()
                .min_by(|q, r| p.distance2(**q).partial_cmp(&p.distance2(**r)).unwrap())
                .map(|q| (p, *q))
        })
        .collect::<Option<Vec<_>>>()?;
    nearest_points.into_iter().try_for_each(move |(p, q)| {
        let Face {
            ref boundaries,
            ref surface,
            ..
        } = faces[face_index];
        let pcurve = PCurve::new(Line(p, q), surface.clone());
        let cut_edge = |(edge_index, param_edge): ZippedEdge<'_>| {
            let index = edge_index.index;
            let vec = enumerate_intersections(&edges[index], param_edge, &pcurve)?;
            if vec.is_empty() {
                return Some(None);
            }
            let erange = cut_edge_by_intersections(index, vec, edges, poly_edges, vertices, tol);
            Some(Some((index, erange)))
        };
        let new_edges = zip_boundaries(boundaries, &param_boundaries)
            .map(cut_edge)
            .collect::<Option<Vec<_>>>()?;
        let insert = |(index, erange): (usize, Range<usize>)| {
            faces
                .iter_mut()
                .flat_map(|face| &mut face.boundaries)
                .for_each(|wire| insert_new_edges(wire, index, erange.clone()));
        };
        new_edges.into_iter().filter_map(identity).for_each(insert);
        Some(())
    })
}

fn closure_find_vertex_parameter<C>(
    v: usize,
    edges: &[Edge<C>],
) -> impl Fn(ZippedEdge<'_>) -> Option<Point2> + '_ {
    let take_front = closure_take_front(edges);
    move |(edge_index, param_edge): ZippedEdge<'_>| match v == take_front(*edge_index) {
        true => Some(param_edge[0]),
        false => None,
    }
}

fn enumerate_intersections<'a, C, S>(
    edge: &Edge<C>,
    param_edge: &PolylineCurve<Point2>,
    pcurve: &'a PCurve<Line<Point2>, S>,
) -> Option<Vec<(f64, Point3)>>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S: ParametricSurface3D,
{
    let rough_intersections = intersections_between_line_polyline(*pcurve.curve(), param_edge);
    let intersections = exact_intersections(rough_intersections, pcurve, &edge.curve)?;
    Some(intersections)
}

fn intersections_between_line_polyline(
    line: Line<Point2>,
    param_edge: &PolylineCurve<Point2>,
) -> Vec<f64> {
    let filter = |p: &[Point2]| {
        let (s, t, _) = line.intersection(Line(p[0], p[1]))?;
        match 0.0 < s && s < 1.0 && 0.0 <= t && t < 1.0 {
            true => Some(s),
            false => None,
        }
    };
    let mut vec: Vec<_> = param_edge.windows(2).filter_map(filter).collect();
    if !vec.is_empty() {
        let first = *param_edge.first().unwrap();
        let last = *param_edge.last().unwrap();
        if line.distance_to_point(first).so_small() {
            vec.remove(0);
        }
        if line.distance_to_point(last).so_small() {
            vec.pop();
        }
    }
    vec
}

fn exact_intersections<C, S>(
    naive_intersections: Vec<f64>,
    pcurve: &PCurve<Line<Point2>, S>,
    curve: &C,
) -> Option<Vec<(f64, Point3)>>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S: ParametricSurface3D,
{
    let closure = |t| {
        let (_, t, p) = search_intersection(&pcurve, curve, t)?;
        Some((t, p))
    };
    let mut res = naive_intersections
        .into_iter()
        .map(closure)
        .collect::<Option<Vec<_>>>()?;
    res.sort_by(|(s, _), (t, _)| s.partial_cmp(&t).unwrap());
    Some(res)
}

fn search_intersection<C, S>(
    pcurve: &PCurve<Line<Point2>, S>,
    curve: &C,
    mut t0: f64,
) -> Option<(f64, f64, Point3)>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S: ParametricSurface3D,
{
    let mut previous0 = t0;
    let mut previous1 = None;
    for _ in 0..100 {
        let p = pcurve.subs(t0);
        let t1 = curve.search_nearest_parameter(p, previous1, 100)?;
        let q = curve.subs(t1);
        t0 = pcurve.search_nearest_parameter(q, t0, 100)?;
        if let Some(previous1) = previous1 {
            if previous0.near(&t0) && previous1.near(&t1) {
                return Some((t0, t1, pcurve.subs(t0)));
            }
        }
        previous0 = t0;
        previous1 = Some(t1);
    }
    None
}

fn cut_edge_by_intersections<C>(
    edge_index: usize,
    intersections: Vec<(f64, Point3)>,
    edges: &mut Vec<Edge<C>>,
    poly_edges: &mut Vec<PolylineCurve<Point3>>,
    vertices: &mut Vec<Point3>,
    tol: f64,
) -> Range<usize>
where
    C: Cut<Point = Point3> + ParameterDivision1D<Point = Point3>,
{
    let edge = &mut edges[edge_index];
    let vfirst = vertices.len();
    vertices.extend(intersections.iter().map(|(_, p)| *p));
    let intersections_iter = intersections.into_iter().enumerate().rev();
    let closure = |(i, (t, _))| Edge {
        vertices: (vfirst + i, vfirst + i + 1),
        curve: edge.curve.cut(t),
    };
    let mut new_edges: Vec<_> = intersections_iter.map(closure).collect();
    let to_poly = closure_to_poly(tol);
    poly_edges[edge_index] = to_poly(edge);
    poly_edges.extend(new_edges.iter().map(to_poly));
    new_edges.last_mut().unwrap().vertices.1 = edge.vertices.1;
    edge.vertices.1 = vfirst;
    let efirst = edges.len();
    edges.extend(new_edges);
    efirst..edges.len()
}

fn insert_new_edges(wire: &mut Wire, pivot_edge_index: usize, inserted_range: Range<usize>) {
    let positions = wire
        .iter()
        .enumerate()
        .filter_map(
            |(i, edge_index)| match edge_index.index == pivot_edge_index {
                true => Some(i),
                false => None,
            },
        )
        .collect::<Vec<usize>>();
    positions.into_iter().rev().for_each(|i| {
        let orientation = wire[i].orientation;
        let ei = |index| EdgeIndex { index, orientation };
        if orientation {
            let tmp_wire = wire.split_off(i + 1);
            wire.extend(inserted_range.clone().map(ei));
            wire.extend(tmp_wire);
        } else {
            let tmp_wire = wire.split_off(i);
            wire.extend(inserted_range.clone().rev().map(ei));
            wire.extend(tmp_wire);
        }
    });
}

// --- sub_split_face_by_divisor ---

fn sub_split_face_by_divisor<C, S>(
    face_index: usize,
    divisor: (usize, usize),
    Shell { faces, edges, .. }: &mut Shell<Point3, C, S>,
    poly_edges: &mut Vec<PolylineCurve<Point3>>,
    sp: impl SP<S>,
    tol: f64,
) -> Option<Vec<Face<S>>>
where
    C: ParametricCurve3D + BoundedCurve + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    let Face {
        ref mut boundaries,
        ref surface,
        orientation: ori,
    } = faces[face_index];
    let param_boundaries = create_param_boundaries(boundaries, surface, poly_edges, &sp)?;
    let param_vertices = create_param_vertices(boundaries, &param_boundaries, edges);
    let vertices_on_divisor = enumerate_vertices_on_divisor(divisor, &param_vertices)?;
    let new_edges = create_new_edges(&vertices_on_divisor, poly_edges, surface, tol)?;
    let (new_edge_range, new_edge_indices) = signup_new_edges(edges, new_edges);
    let edge_iter = boundaries.iter().flatten().copied().chain(new_edge_indices);
    let vemap = create_vemap(edges, edge_iter);
    let new_boundaries = construct_boundaries(vemap, edges, new_edge_range);
    divide_face(boundaries, surface, ori, new_boundaries, poly_edges, sp)
}

// --- create_param_boundaries ---

fn create_param_boundaries<S: ParametricSurface3D>(
    boundaries: &[Wire],
    surface: &S,
    poly_edges: &[PolylineCurve<Point3>],
    sp: impl SP<S>,
) -> Option<Vec<Vec<PolylineCurve<Point2>>>> {
    let closure = |boundary: &Wire| boundary_to_param_polys(boundary, surface, poly_edges, &sp);
    boundaries.iter().map(closure).collect()
}

fn boundary_to_param_polys<S: ParametricSurface3D>(
    boundary: &Wire,
    surface: &S,
    poly_edges: &[PolylineCurve<Point3>],
    sp: impl SP<S>,
) -> Option<Vec<PolylineCurve<Point2>>> {
    let get_poly = closure_get_poly(poly_edges);
    let poly_boundary: Vec<_> = boundary.iter().copied().map(get_poly).collect();
    let poly_wire_iter = PolyWireIter::try_new(&poly_boundary)?;
    let closure = poly_project_to_uv(surface, sp);
    let mut long_poly = poly_wire_iter.map(closure).collect::<Option<Vec<_>>>()?;
    boundary_into_domain(&mut long_poly, surface);
    let mut vec = vec![long_poly.remove(0)];
    let split_into_2dpoly = move |poly: &PolylineCurve<_>| {
        let latter = long_poly.split_off(poly.len() - 1);
        vec.append(&mut long_poly);
        let mut res = Vec::new();
        res.append(&mut vec);
        vec.push(*res.last().unwrap());
        long_poly = latter;
        PolylineCurve(res)
    };
    Some(poly_boundary.iter().map(split_into_2dpoly).collect())
}

struct PolyWireIter<'a, P> {
    current: std::slice::Iter<'a, P>,
    stock: std::vec::IntoIter<std::slice::Iter<'a, P>>,
}

impl<'a, P> PolyWireIter<'a, P> {
    fn try_new<I: AsRef<[P]> + 'a>(vec: &'a [I]) -> Option<Self> {
        let iters: Vec<_> = vec.iter().map(|v| v.as_ref().iter()).collect();
        let mut stock = iters.into_iter();
        Some(Self {
            current: stock.next()?,
            stock,
        })
    }
}

impl<'a, P: Copy> Iterator for PolyWireIter<'a, P> {
    type Item = P;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.current.next() {
            Some(*next)
        } else {
            self.current = self.stock.next()?;
            self.current.next()?;
            self.current.next().cloned()
        }
    }
}

fn poly_project_to_uv<'a, S: ParametricSurface3D>(
    surface: &'a S,
    sp: impl SP<S> + 'a,
) -> impl FnMut(Point3) -> Option<Point2> + 'a {
    let (up, vp) = (surface.u_period(), surface.v_period());
    let mut previous = None;
    move |pt| {
        let (mut u, mut v) = sp(surface, pt, previous)?;
        if let (Some(up), Some((u0, _))) = (up, previous) {
            u = get_mindiff(u, u0, up);
        }
        if let (Some(vp), Some((_, v0))) = (vp, previous) {
            v = get_mindiff(v, v0, vp);
        }
        previous = Some((u, v));
        Some(Point2::new(u, v))
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

fn boundary_into_domain<S: ParametricSurface3D>(vec: &mut Vec<Point2>, surface: &S) {
    let (up, vp) = (surface.u_period(), surface.v_period());
    let (urange, vrange) = surface.try_range_tuple();
    let grav = vec.iter().fold(Point2::origin(), |g, p| g + p.to_vec()) / vec.len() as f64;
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
        let Point2 { x: u0, y: v0 } = last;
        if surface.uder(u0, v0).so_small() || surface.vder(u0, v0).so_small() {
            vec.push(vec[0]);
        }
    }
}

// --- create_param_vertices ---

fn create_param_vertices<C>(
    boundaries: &[Wire],
    param_boundaries: &[Vec<PolylineCurve<Point2>>],
    edges: &[Edge<C>],
) -> HashMap<usize, Vec<Point2>> {
    let take_front = closure_take_front(edges);
    let mut vmap = HashMap::default();
    zip_boundaries(boundaries, param_boundaries).for_each(|(edge_index, param_edge)| {
        let v = take_front(*edge_index);
        vmap.entry(v).or_insert_with(Vec::new).push(param_edge[0]);
    });
    boundaries.iter().zip(param_boundaries).for_each(|(b, pb)| {
        let last = pb.last().unwrap().last().unwrap();
        if pb[0][0].distance(*last) > 1.0e-3 {
            let v = take_front(b[0]);
            vmap.entry(v).or_insert_with(Vec::new).push(*last);
        }
    });
    vmap
}

// --- enumerate_vertices_on_divisor ---

fn enumerate_vertices_on_divisor(
    divisor: (usize, usize),
    param_vertices: &HashMap<usize, Vec<Point2>>,
) -> Option<Vec<(f64, (usize, Vec<Point2>))>> {
    let (v0, v1) = divisor;
    let (p0, p1) = (param_vertices.get(&v0)?, param_vertices.get(&v1)?);
    let nearest_points = p0
        .iter()
        .map(|p| {
            p1.iter()
                .min_by(|q, r| p.distance2(**q).partial_cmp(&p.distance2(**r)).unwrap())
                .map(|q| (*p, *q))
        })
        .collect::<Option<Vec<_>>>()?;
    let mut vertices_on_divisor = nearest_points
        .into_iter()
        .flat_map(|(p, q)| {
            let line = Line(p, q);
            param_vertices
                .iter()
                .filter(move |(_, uv)| line.distance_to_point_as_segment(uv[0]).so_small())
                .map(move |(v, uv)| {
                    Some((
                        line.search_nearest_parameter(uv[0], None, 1)?,
                        (*v, uv.clone()),
                    ))
                })
        })
        .collect::<Option<Vec<_>>>()?;
    vertices_on_divisor.sort_by(|(s, _), (t, _)| s.partial_cmp(t).unwrap());
    Some(vertices_on_divisor)
}

// --- create_new_edges ---

fn create_new_edges<C, S>(
    vertices_on_divisor: &[(f64, (usize, Vec<Point2>))],
    poly_edges: &mut Vec<PolylineCurve<Point3>>,
    surface: &S,
    tol: f64,
) -> Option<Vec<Edge<C>>>
where
    C: TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    vertices_on_divisor
        .chunks(2)
        .map(|p| {
            let ((_, (v0, vec0)), (_, (v1, vec1))) = (&p[0], &p[1]);
            let uv0 = vec0[0];
            let uv1 = *vec1
                .iter()
                .min_by(|q, r| uv0.distance2(**q).partial_cmp(&uv0.distance2(**r)).unwrap())?;
            let pcurve = PCurve::new(Line(uv0, uv1), surface.clone());
            poly_edges.push(PolylineCurve::from_curve(&pcurve, (0.0, 1.0), tol));
            Some(Edge {
                vertices: (*v0, *v1),
                curve: C::try_from(pcurve).ok()?,
            })
        })
        .collect()
}

// --- signup new edges ---

fn signup_new_edges<C>(
    edges: &mut Vec<Edge<C>>,
    new_edges: Vec<Edge<C>>,
) -> (Range<usize>, impl Iterator<Item = EdgeIndex>) {
    let len = edges.len();
    edges.extend(new_edges);
    let new_edge_indices = (len..edges.len()).flat_map(move |index| {
        let ei = |orientation: bool| EdgeIndex { index, orientation };
        [ei(true), ei(false)]
    });
    (len..edges.len(), new_edge_indices)
}

// --- create_vemap ---

fn create_vemap<C>(
    edges: &[Edge<C>],
    edge_iter: impl Iterator<Item = EdgeIndex>,
) -> HashMap<usize, Vec<EdgeIndex>> {
    let mut vemap = HashMap::<usize, Vec<EdgeIndex>>::default();
    let take_front = closure_take_front(edges);
    edge_iter.for_each(|edge_index| {
        let v = take_front(edge_index);
        vemap.entry(v).or_insert_with(Vec::new).push(edge_index);
    });
    vemap
}

// --- construct boundaries ---

fn construct_boundaries<C>(
    mut vemap: HashMap<usize, Vec<EdgeIndex>>,
    edges: &[Edge<C>],
    new_edge_range: Range<usize>,
) -> Vec<Wire> {
    let take_back = closure_take_back(edges);
    let mut new_boundaries = Vec::new();
    while let Some((start, vec)) = vemap.iter_mut().next() {
        let start = *start;
        let mut edge_index = vec.pop().unwrap();
        if vec.is_empty() {
            vemap.remove(&start);
        }
        let mut wire = Vec::new();
        loop {
            wire.push(edge_index);
            let v = take_back(edge_index);
            if v == start {
                break;
            }
            let Some(vec) = vemap.get_mut(&v) else {
                unreachable!();
            };
            match vec.len() {
                1 => {
                    edge_index = vec.pop().unwrap();
                    vemap.remove(&v);
                }
                2 => {
                    let is_new0 = new_edge_range.contains(&edge_index.index);
                    let is_new1 = new_edge_range.contains(&vec[0].index);
                    let i = if is_new0 != is_new1 { 0 } else { 1 };
                    edge_index = vec.remove(i);
                }
                _ => panic!("something wrong!"),
            }
        }
        new_boundaries.push(wire);
    }
    new_boundaries
}

// --- divide_face ---

fn divide_face<S: ParametricSurface3D>(
    boundaries: &mut Vec<Wire>,
    surface: &S,
    orientation: bool,
    new_boundaries: Vec<Wire>,
    poly_edges: &[PolylineCurve<Point3>],
    sp: impl SP<S>,
) -> Option<Vec<Face<S>>> {
    let mut face_boundaries = assort_boundary(surface, new_boundaries, poly_edges, sp)?.into_iter();
    *boundaries = face_boundaries.next().unwrap();
    let create_face = |boundaries| Face {
        boundaries,
        surface: surface.clone(),
        orientation,
    };
    Some(face_boundaries.map(create_face).collect())
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
    let (mut positives, mut negatives) = (Vec::new(), Vec::new());
    let param_boundaries = create_param_boundaries(&boundaries, surface, poly_edges, sp)?;
    boundaries
        .into_iter()
        .zip(param_boundaries)
        .try_for_each(|(boundary, param_boundary)| {
            let poly_boundary = PolylineCurve(param_boundary.into_iter().flatten().collect());
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

// --- common functions ---

fn closure_take_front<C>(edges: &[Edge<C>]) -> impl Fn(EdgeIndex) -> usize + '_ {
    move |edge_index: EdgeIndex| {
        let index = edge_index.index;
        let (v0, v1) = edges[index].vertices;
        match edge_index.orientation {
            true => v0,
            false => v1,
        }
    }
}

fn closure_take_back<C>(edges: &[Edge<C>]) -> impl Fn(EdgeIndex) -> usize + '_ {
    move |edge_index: EdgeIndex| {
        let (v0, v1) = edges[edge_index.index].vertices;
        match edge_index.orientation {
            true => v1,
            false => v0,
        }
    }
}

fn closure_get_poly<P: Clone>(
    poly_edges: &[PolylineCurve<P>],
) -> impl Fn(EdgeIndex) -> PolylineCurve<P> + '_ {
    move |edge_index: EdgeIndex| {
        let poly = &poly_edges[edge_index.index];
        match edge_index.orientation {
            true => poly.clone(),
            false => poly.inverse(),
        }
    }
}

fn closure_to_poly<P, C>(tol: f64) -> impl Fn(&Edge<C>) -> PolylineCurve<P> + 'static
where C: BoundedCurve + ParameterDivision1D<Point = P> {
    move |Edge { curve, .. }: &Edge<C>| PolylineCurve::from_curve(curve, curve.range_tuple(), tol)
}

type ZippedEdge<'a> = (&'a EdgeIndex, &'a PolylineCurve<Point2>);

fn zip_boundaries<'a>(
    boundaries: &'a [Wire],
    param_boundaries: &'a [Vec<PolylineCurve<Point2>>],
) -> impl Iterator<Item = ZippedEdge<'a>> {
    boundaries
        .iter()
        .zip(param_boundaries)
        .flat_map(move |(b, pb)| b.iter().zip(pb))
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
