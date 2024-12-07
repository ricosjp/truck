use super::*;
use std::ops::Range;

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
    (0..len).for_each(|i| {
        split_face_with_non_closed_boundary(i, shell, &mut poly_edges, &sp, tol);
    });
    let len = shell.faces.len();
    let closure = |i| split_face_with_non_simple_wire(i, shell, &mut poly_edges, &sp, tol);
    let new_faces: Vec<_> = (0..len).filter_map(closure).flatten().collect();
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
    let (divisor, open, closed) =
        non_closed_wires_in_param_divisor(face_index, shell, poly_edges, &sp)?;
    debug_assert_eq!(open.len(), 2);
    take_vertices_to_intersections(divisor, face_index, shell, poly_edges, &sp, tol);
    let Shell { faces, edges, .. } = shell;
    let new_boundaries = split_boundaries_by_divisor(
        &faces[face_index],
        &closed,
        divisor,
        edges,
        poly_edges,
        &sp,
        tol,
    )?;
    let boundaries = &mut faces[face_index].boundaries;
    connect_open_boundaries(boundaries, new_boundaries, divisor, edges, open)
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
    let divisor = find_non_simple_wire_divisor(boundaries, &shell.edges, &shell.vertices)?;
    split_face_by_divisor(face_index, divisor, shell, poly_edges, sp, tol)
}

// --- find_non_simple_wire_divisor ---

fn find_non_simple_wire_divisor<C>(
    boundaries: &[Wire],
    edges: &[Edge<C>],
    vertices: &[Point3],
) -> Option<(usize, usize)> {
    let closure = |boundary| non_simple_wire_divisor(boundary, edges, vertices);
    boundaries.iter().cloned().find_map(closure)
}

fn non_simple_wire_divisor<C>(
    mut boundary: Wire,
    edges: &[Edge<C>],
    vertices: &[Point3],
) -> Option<(usize, usize)> {
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
    let pre_divisor = (f(boundary[k0]), f(boundary[k1]));
    nearest_correction(
        pre_divisor,
        &boundary[1..j0],
        &boundary[i1 + 1..j1],
        vertices,
        closure_take_front(edges),
    )
}

fn find_loop() -> impl FnMut((usize, &EdgeIndex)) -> Option<(usize, usize)> {
    let mut map = HashMap::<usize, usize>::default();
    move |(i, edge)| map.insert(edge.index, i).map(move |j| (j, i))
}

// --- find_non_closed_wires_in_param_divisor ---

type FindNonClosedWiresInParamDivisorResult = Option<((usize, usize), Vec<Wire>, Vec<Wire>)>;
fn non_closed_wires_in_param_divisor<C, S>(
    face_index: usize,
    Shell {
        edges,
        faces,
        ref vertices,
    }: &mut Shell<Point3, C, S>,
    poly_edges: &[PolylineCurve<Point3>],
    sp: impl SP<S>,
) -> FindNonClosedWiresInParamDivisorResult
where
    C: ParametricCurve3D + BoundedCurve + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    let Face {
        boundaries,
        surface,
        ..
    } = &faces[face_index];
    let param_boundaries = create_param_boundaries(boundaries, surface, poly_edges, sp)?;
    let (mut open, mut closed) = (Vec::new(), Vec::new());
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
    let take_front = &closure_take_front(edges);
    let find_nearest_to_ends = move |i: usize, u0: f64, up: f64| {
        move |(edge_index, poly): ZippedEdge<'_>| {
            let u = poly[0][i];
            let closure = move |i| {
                let u = u + i as f64 * up;
                f64::abs(u - u0)
            };
            let dist = (-2..=2)
                .map(closure)
                .min_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap();
            (take_front(*edge_index), dist)
        }
    };
    let create_pre_divisor = move |(wire, poly_wire): ZippedWire<'_>| {
        let (urange, vrange) = surface.try_range_tuple();
        let (up, vp) = (surface.u_period(), surface.v_period());
        let p = poly_wire[0][0];
        let q = *poly_wire.last().unwrap().last().unwrap();
        let vertices: Vec<(usize, f64)> = if p.x.near(&q.x) {
            if let (Some(vp), Some((v0, _))) = (vp, vrange) {
                let closure = find_nearest_to_ends(1, v0, vp);
                Some(wire.iter().zip(poly_wire).map(closure).collect())
            } else {
                None
            }
        } else if p.y.near(&q.y) {
            if let (Some(up), Some((u0, _))) = (up, urange) {
                let closure = find_nearest_to_ends(0, u0, up);
                Some(wire.iter().zip(poly_wire).map(closure).collect())
            } else {
                None
            }
        } else {
            None
        }?;
        vertices
            .into_iter()
            .min_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap())
            .map(|(v, _)| v)
    };
    let pre_divisor_vec = open
        .iter()
        .copied()
        .map(create_pre_divisor)
        .collect::<Option<Vec<_>>>()?;
    let open: Vec<_> = open.into_iter().map(|(w, _)| w.to_vec()).collect();
    let closed: Vec<_> = closed.into_iter().map(|(w, _)| w.to_vec()).collect();
    let divisor = nearest_correction(
        (pre_divisor_vec[0], pre_divisor_vec[1]),
        &open[0],
        &open[1],
        vertices,
        take_front,
    )?;
    Some((divisor, open, closed))
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
    let face = &shell.faces[face_index];
    let new_boundaries = split_boundaries_by_divisor(
        face,
        &face.boundaries,
        divisor,
        &mut shell.edges,
        poly_edges,
        &sp,
        tol,
    )?;
    divide_face(
        &mut shell.faces[face_index],
        new_boundaries,
        poly_edges,
        &sp,
    )
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
    let p = zip_boundaries(boundaries, &param_boundaries)
        .find_map(closure_find_vertex_parameter(v0, edges))?;
    let q = zip_boundaries(boundaries, &param_boundaries)
        .find_map(closure_find_vertex_parameter(v1, edges))?;
    let Face {
        ref boundaries,
        ref surface,
        ..
    } = faces[face_index];
    let periods = (surface.u_period(), surface.v_period());
    let q = periodic_iterator(q, periods)
        .min_by(|q, r| p.distance2(*q).partial_cmp(&p.distance2(*r)).unwrap())?;
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
    new_edges.into_iter().flatten().for_each(insert);
    Some(())
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

fn enumerate_intersections<C, S>(
    edge: &Edge<C>,
    param_edge: &PolylineCurve<Point2>,
    pcurve: &PCurve<Line<Point2>, S>,
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
        let unit = 0.0..1.0;
        match unit.contains(&s) && unit.contains(&t) {
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
        let (_, t, p) = search_intersection(pcurve, curve, t)?;
        Some((t, p))
    };
    let mut res = naive_intersections
        .into_iter()
        .map(closure)
        .collect::<Option<Vec<_>>>()?;
    res.sort_by(|(s, _), (t, _)| s.partial_cmp(t).unwrap());
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

// --- split boundaries ---

fn split_boundaries_by_divisor<C, S>(
    Face {
        boundaries,
        surface,
        ..
    }: &Face<S>,
    closed: &[Wire],
    divisor: (usize, usize),
    edges: &mut Vec<Edge<C>>,
    poly_edges: &mut Vec<PolylineCurve<Point3>>,
    sp: impl SP<S>,
    tol: f64,
) -> Option<Vec<Wire>>
where
    C: ParametricCurve3D + BoundedCurve + TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    let param_boundaries = create_param_boundaries(boundaries, surface, poly_edges, &sp)?;
    let param_vertices = create_param_vertices(boundaries, &param_boundaries, edges);
    let mut duplicated_edges = duplicated_edges(boundaries.iter().flatten().map(|ei| ei.index));
    let vertices_on_divisor = enumerate_vertices_on_divisor(divisor, &param_vertices, surface)?;
    let new_edges = create_new_edges(&vertices_on_divisor, poly_edges, surface, tol)?;
    let (new_edge_range, new_edge_indices) = signup_new_edges(edges, new_edges);
    duplicated_edges.extend(new_edge_range);
    let edge_iter = closed.iter().flatten().copied().chain(new_edge_indices);
    let vemap = create_vemap(edges, edge_iter);
    Some(construct_boundaries(vemap, edges, &duplicated_edges))
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

impl<P: Copy> Iterator for PolyWireIter<'_, P> {
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
) -> HashMap<usize, Point2> {
    let take_front = closure_take_front(edges);
    zip_boundaries(boundaries, param_boundaries)
        .map(|(edge_index, param_edge)| (take_front(*edge_index), param_edge[0]))
        .collect()
}

// --- duplicated_edges ---

fn duplicated_edges(edges: impl Iterator<Item = usize>) -> HashSet<usize> {
    let mut done = HashSet::default();
    edges.filter(move |index| !done.insert(*index)).collect()
}

// --- enumerate_vertices_on_divisor ---

fn enumerate_vertices_on_divisor<S: ParametricSurface>(
    divisor: (usize, usize),
    param_vertices: &HashMap<usize, Point2>,
    surface: &S,
) -> Option<Vec<(f64, (usize, Point2))>> {
    let (v0, v1) = divisor;
    let (p, q) = (*param_vertices.get(&v0)?, *param_vertices.get(&v1)?);
    let periods = (surface.u_period(), surface.v_period());
    let q = periodic_iterator(q, periods)
        .min_by(|q, r| p.distance2(*q).partial_cmp(&p.distance2(*r)).unwrap())?;
    let line = Line(p, q);
    let iter = param_vertices.iter().filter_map(move |(v, uv)| {
        periodic_iterator(*uv, periods)
            .find(move |uv| line.distance_to_point_as_segment(*uv).so_small())
            .map(move |uv| Some((line.search_nearest_parameter(uv, None, 1)?, (*v, uv))))
    });
    let mut vertices_on_divisor = iter.collect::<Option<Vec<_>>>()?;
    vertices_on_divisor.sort_by(|(s, _), (t, _)| s.partial_cmp(t).unwrap());
    Some(vertices_on_divisor)
}

// --- create_new_edges ---

fn create_new_edges<C, S>(
    vertices_on_divisor: &[(f64, (usize, Point2))],
    poly_edges: &mut Vec<PolylineCurve<Point3>>,
    surface: &S,
    tol: f64,
) -> Option<Vec<Edge<C>>>
where
    C: TryFrom<PCurve<Line<Point2>, S>>,
    S: ParametricSurface3D,
{
    vertices_on_divisor
        .iter()
        .for_each(|(_, (_, uv))| print!("{:?} ", surface.subs(uv.x, uv.y)));
    println!();
    let make_edge = move |p: &[(f64, (usize, Point2))]| {
        let ((_, (v0, uv0)), (_, (v1, uv1))) = (p[0], p[1]);
        let pcurve = PCurve::new(Line(uv0, uv1), surface.clone());
        poly_edges.push(PolylineCurve::from_curve(&pcurve, (0.0, 1.0), tol));
        Some(Edge {
            vertices: (v0, v1),
            curve: C::try_from(pcurve).ok()?,
        })
    };
    vertices_on_divisor.chunks(2).map(make_edge).collect()
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
    new_edge_range: &HashSet<usize>,
) -> Vec<Wire> {
    let take_back = closure_take_back(edges);
    let mut new_boundaries = Vec::new();
    while !vemap.is_empty() {
        let (start, vec) = vemap.iter_mut().min_by_key(|(idx, _)| *idx).unwrap();
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
    Face {
        ref mut boundaries,
        ref surface,
        ref orientation,
    }: &mut Face<S>,
    new_boundaries: Vec<Wire>,
    poly_edges: &[PolylineCurve<Point3>],
    sp: impl SP<S>,
) -> Option<Vec<Face<S>>> {
    let mut face_boundaries = assort_boundary(surface, new_boundaries, poly_edges, sp)?.into_iter();
    *boundaries = face_boundaries.next().unwrap();
    let create_face = |boundaries| Face {
        boundaries,
        surface: surface.clone(),
        orientation: *orientation,
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

// --- connect_open_boundaries ---

fn connect_open_boundaries<C>(
    boundaries: &mut Vec<Wire>,
    mut new_boundaries: Vec<Wire>,
    divisor: (usize, usize),
    edges: &[Edge<C>],
    mut open: Vec<Wire>,
) -> Option<()> {
    let (v0, v1) = divisor;
    let take_front = closure_take_front(edges);
    let find_index = move |wire: &Wire, v: usize| {
        wire.iter()
            .position(|edge_index| take_front(*edge_index) == v)
    };
    let (i, index) = open
        .iter()
        .enumerate()
        .find_map(|(i, wire)| find_index(wire, v0).map(|index| (i, index)))?;
    if index != 0 {
        open.swap(0, 1);
    }
    let (mut open_v1, mut open_v0) = (open.pop()?, open.pop()?);
    let j = find_index(&open_v1, v1)?;
    open_v0.rotate_left(i);
    open_v1.rotate_left(j);
    let (k, added_wire) = new_boundaries
        .iter_mut()
        .find_map(|wire| find_index(wire, v0).map(|index| (index, wire)))?;
    added_wire.rotate_left(k);
    open_v0.append(added_wire);
    let l = find_index(&open_v0, v1)?;
    open_v0.rotate_left(l);
    open_v0.append(&mut open_v1);
    *added_wire = open_v0;
    *boundaries = new_boundaries;
    Some(())
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
type ZippedWire<'a> = (&'a Wire, &'a Vec<PolylineCurve<Point2>>);

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

fn periodic_iterator(
    p: Point2,
    (up, vp): (Option<f64>, Option<f64>),
) -> impl Iterator<Item = Point2> {
    let up_range = match up {
        Some(up) => vec![-2.0 * up, -up, 0.0, up, 2.0 * up],
        None => vec![0.0],
    };
    let vp_range = match vp {
        Some(vp) => vec![-2.0 * vp, -vp, 0.0, vp, 2.0 * vp],
        None => vec![0.0],
    };
    itertools::iproduct!(up_range, vp_range).map(move |(dx, dy)| p + Vector2::new(dx, dy))
}

fn nearest_correction(
    (v0, _v1): (usize, usize),
    wire0: &[EdgeIndex],
    wire1: &[EdgeIndex],
    vertices: &[Point3],
    take_front: impl Fn(EdgeIndex) -> usize,
) -> Option<(usize, usize)> {
    let p0 = vertices[v0];
    let v1 = wire1.iter().copied().map(&take_front).min_by(|v, w| {
        p0.distance2(vertices[*v])
            .partial_cmp(&p0.distance2(vertices[*w]))
            .unwrap()
    })?;
    let p1 = vertices[v1];
    let v0 = wire0.iter().copied().map(&take_front).min_by(|v, w| {
        p1.distance2(vertices[*v])
            .partial_cmp(&p1.distance2(vertices[*w]))
            .unwrap()
    })?;
    Some((v0, v1))
}
