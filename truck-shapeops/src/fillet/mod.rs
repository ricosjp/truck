use algo::curve::search_intersection_parameter;
use itertools::Itertools;
use truck_geometry::prelude::{rbf_surface::*, *};
use truck_topology::*;

/// condition of curves to attach fillet
pub trait FilletedCurve<S>:
    ParametricCurve3D
    + SearchNearestParameter<D1, Point = Point3>
    + BoundedCurve
    + Cut
    + Invertible
    + std::fmt::Debug {
}
impl<
        C: ParametricCurve3D
            + SearchNearestParameter<D1, Point = Point3>
            + BoundedCurve
            + Cut
            + Invertible
            + std::fmt::Debug,
        S,
    > FilletedCurve<S> for C
{
}

/// condition of sufaces to attach fillet
pub trait FilletedSurface<C>:
    ParametricSurface3D + SearchParameter<D2, Point = Point3> + Invertible {
}
impl<C, S: ParametricSurface3D + SearchParameter<D2, Point = Point3> + Invertible>
    FilletedSurface<C> for S
{
}

fn find_adjacent_edge<P: Clone, C: Clone, S>(
    face: &Face<P, C, S>,
    edge_id: EdgeID<C>,
) -> Option<(Edge<P, C>, Edge<P, C>)> {
    face.boundary_iters()
        .into_iter()
        .flat_map(|boundary_iter| boundary_iter.circular_tuple_windows())
        .find(|(_, edge, _)| edge.id() == edge_id)
        .map(|(x, _, y)| (x, y))
}

#[allow(clippy::type_complexity)]
fn cut_face_by_curve<C, S>(
    face: &Face<Point3, C, S>,
    mut curve: C,
    filleted_edge_id: EdgeID<C>,
) -> Option<(Face<Point3, C, S>, Edge<Point3, C>, (Vector3, Vector3))>
where
    C: FilletedCurve<S>,
    S: Clone,
{
    let (front_edge, back_edge) = find_adjacent_edge(face, filleted_edge_id)?;

    let (new_front_edge, front_der) = {
        let front_curve = front_edge.curve();
        let front_curve_hint = match front_edge.orientation() {
            true => front_curve.range_tuple().1,
            false => front_curve.range_tuple().0,
        };
        let curve_hint = curve.range_tuple().0;
        let hint = (curve_hint, front_curve_hint);
        let (t0, t1) = search_intersection_parameter(&curve, &front_curve, hint, 100)?;
        let p = curve.subs(t0).midpoint(front_curve.subs(t1));
        let v0 = Vertex::new(p);
        if !curve_hint.near(&t0) {
            curve = curve.cut(t0);
        }
        let der = front_curve.der(t1);
        let new_front_edge = front_edge.cut_with_parameter(&v0, t1)?.0;
        match front_edge.orientation() {
            true => (new_front_edge, der),
            false => (new_front_edge, -der),
        }
    };

    let (new_back_edge, back_der) = {
        let back_curve = back_edge.curve();
        let back_curve_hint = match back_edge.orientation() {
            true => back_curve.range_tuple().0,
            false => back_curve.range_tuple().1,
        };
        let curve_hint = curve.range_tuple().1;
        let hint = (curve_hint, back_curve_hint);
        let (t0, t1) = search_intersection_parameter(&curve, &back_curve, hint, 100)?;
        let p = curve.subs(t0).midpoint(back_curve.subs(t1));
        let v1 = Vertex::new(p);
        if !t0.near(&curve_hint) {
            curve.cut(t0);
        }
        let der = back_curve.der(t1);
        let new_back_edge = back_edge.cut_with_parameter(&v1, t1)?.1;
        match back_edge.orientation() {
            true => (new_back_edge, der),
            false => (new_back_edge, -der),
        }
    };

    let fillet_edge = Edge::new(new_front_edge.back(), new_back_edge.front(), curve);
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
    Some((new_face, fillet_edge, (front_der, back_der)))
}

fn create_pcurve_edge<C, S>(
    (v0, hint0, der0): (&Vertex<Point3>, (f64, f64), Vector3),
    (v1, hint1, der1): (&Vertex<Point3>, (f64, f64), Vector3),
    fillet_surface: S,
) -> Option<Edge<Point3, C>>
where
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3> + Clone,
{
    let uv0: Point2 = fillet_surface
        .search_parameter(v0.point(), hint0, 100)?
        .into();
    let uv1: Point2 = fillet_surface
        .search_parameter(v1.point(), hint1, 100)?
        .into();
    let dist = uv0.distance(uv1);

    let uder0 = fillet_surface.uder(uv0.x, uv0.y);
    let vder0 = fillet_surface.vder(uv0.x, uv0.y);
    let n0 = uder0.cross(vder0);
    let uvder0 = Matrix3::from_cols(uder0, vder0, n0).invert().unwrap() * der0;
    //debug_assert!(uvder0.z.so_small(), "{:?}", uvder0);
    let cp1 = uv0 + dist / 3.0 * uvder0.truncate().normalize();

    let uder1 = fillet_surface.uder(uv1.x, uv1.y);
    let vder1 = fillet_surface.vder(uv1.x, uv1.y);
    let n1 = uder1.cross(vder1).normalize();
    let uvder1 = Matrix3::from_cols(uder1, vder1, n1).invert().unwrap() * der1;
    //debug_assert!(uvder1.z.so_small(), "{:?}", uvder1);
    let cp2 = uv1 - dist / 3.0 * uvder1.truncate().normalize();

    let bsp = BSplineCurve::new(KnotVec::bezier_knot(3), vec![uv0, cp1, cp2, uv1]);
    let curve = PCurve::new(bsp, fillet_surface);
    Some(Edge::new(v0, v1, curve.to_same_geometry()))
}

/// result of [`simple_fillet`]
#[derive(Clone, Debug)]
pub struct SimpleFillet<C, S> {
    /// the trimmed first face
    pub face0: Face<Point3, C, S>,
    /// the trimmed second face
    pub face1: Face<Point3, C, S>,
    /// the added new fillet face
    pub fillet: Face<Point3, C, S>,
}

/// create simple fillet
pub fn simple_fillet<C, S, R>(
    face0: &Face<Point3, C, S>,
    face1: &Face<Point3, C, S>,
    filleted_edge_id: EdgeID<C>,
    radius: R,
    tol: f64,
) -> Option<SimpleFillet<C, S>>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
    R: RadiusFunction,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    ApproxFilletSurface<S, S>: ToSameGeometry<S>,
{
    let is_filleted_edge = move |edge: &Edge<Point3, C>| edge.id() == filleted_edge_id;
    let filleted_edge = face0.edge_iter().find(is_filleted_edge)?;

    let strict_surface = {
        let surface0 = face0.oriented_surface();
        let surface1 = face1.oriented_surface();
        let curve = filleted_edge.oriented_curve();
        RbfSurface::new(curve, surface0, surface1, radius)
    };

    let vrange = {
        let (t0, t1) = strict_surface.edge_curve().range_tuple();

        let (front_edge0, back_edge0) = find_adjacent_edge(face0, filleted_edge_id)?;

        let curve00 = front_edge0.oriented_curve();
        let (_, s00_hint) = curve00.range_tuple();
        let (_, _, v00, _) = strict_surface
            .search_contact_curve0_cross_point_with_adjacent_edge(t0, &curve00, s00_hint, 100)
            .unwrap();

        let curve10 = back_edge0.oriented_curve();
        let (s10_hint, _) = curve10.range_tuple();
        let (_, _, v10, _) = strict_surface
            .search_contact_curve0_cross_point_with_adjacent_edge(t1, &curve10, s10_hint, 100)
            .unwrap();

        let (front_edge1, back_edge1) = find_adjacent_edge(face1, filleted_edge_id)?;

        let curve11 = front_edge1.oriented_curve();
        let (_, s11_hint) = curve11.range_tuple();
        let (_, _, v11, _) = strict_surface
            .search_contact_curve1_cross_point_with_adjacent_edge(t1, &curve11, s11_hint, 100)
            .unwrap();

        let curve01 = back_edge1.oriented_curve();
        let (s01_hint, _) = curve01.range_tuple();
        let (_, _, v01, _) = strict_surface
            .search_contact_curve1_cross_point_with_adjacent_edge(t0, &curve01, s01_hint, 100)
            .unwrap();

        (v00.min(v01), v10.max(v11))
    };

    let fillet_surface =
        ApproxFilletSurface::approx_rolling_ball_fillet(&strict_surface, vrange, tol)?;

    let (new_face0, fillet_edge0, (front_der0, back_der0)) = {
        let contact_curve = fillet_surface.side_pcurve0().to_same_geometry();
        cut_face_by_curve(face0, contact_curve, filleted_edge.id())?
    };
    let (new_face1, fillet_edge1, (front_der1, back_der1)) = {
        let contact_curve = fillet_surface.side_pcurve1().to_same_geometry();
        cut_face_by_curve(face1, contact_curve.inverse(), filleted_edge.id())?
    };

    let surface = fillet_surface.to_same_geometry();
    let ((vertex0, vertex1), (vertex2, vertex3)) = (fillet_edge0.ends(), fillet_edge1.ends());
    let ((u0, u1), (v0, v1)) = fillet_surface.range_tuple();
    let edge0 = create_pcurve_edge(
        (vertex0, (u0, v0), front_der0),
        (vertex3, (u1, v0), back_der1),
        surface.clone(),
    )?;
    let edge1 = create_pcurve_edge(
        (vertex2, (u1, v1), front_der1),
        (vertex1, (u0, v1), back_der0),
        surface.clone(),
    )?;
    let fillet = {
        let fillet_boundary = [fillet_edge0.inverse(), edge0, fillet_edge1.inverse(), edge1];
        Face::new(vec![fillet_boundary.into()], surface)
    };

    Some(SimpleFillet {
        face0: new_face0,
        face1: new_face1,
        fillet,
    })
}

/// result of [`fillet_with_side`].
#[derive(Clone, Debug)]
pub struct FilletWithSide<C, S> {
    /// simple fillet part
    pub simple_fillet: SimpleFillet<C, S>,
    /// the trimmed first side face
    pub side0: Option<Face<Point3, C, S>>,
    /// the trimmed second side face
    pub side1: Option<Face<Point3, C, S>>,
}

fn create_new_side<C, S>(
    side: &Face<Point3, C, S>,
    fillet_edge: &Edge<Point3, C>,
    fillet_surface: &S,
    corner_vertex_id: VertexID<Point3>,
    left_face_front_edge: &Edge<Point3, C>,
    right_face_back_edge: &Edge<Point3, C>,
) -> Option<Face<Point3, C, S>>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
    IntersectionCurve<C, S, S>: ToSameGeometry<C>,
{
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
    let side_surface = side.oriented_surface();
    let fillet_surface = fillet_surface.clone();
    let new_curve = IntersectionCurve::new(side_surface, fillet_surface, fillet_edge.curve());
    fillet_edge.set_curve(new_curve.to_same_geometry());
    let mut new_face = Face::new(new_boundaries, side.surface());
    if !side.orientation() {
        new_face.invert();
    }
    Some(new_face)
}

/// Create fillet with side faces
pub fn fillet_with_side<C, S, R>(
    face0: &Face<Point3, C, S>,
    face1: &Face<Point3, C, S>,
    filleted_edge_id: EdgeID<C>,
    side0: Option<&Face<Point3, C, S>>,
    side1: Option<&Face<Point3, C, S>>,
    radius: R,
    tol: f64,
) -> Option<FilletWithSide<C, S>>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
    R: RadiusFunction,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    IntersectionCurve<C, S, S>: ToSameGeometry<C>,
    ApproxFilletSurface<S, S>: ToSameGeometry<S>,
{
    let simple_fillet = simple_fillet(face0, face1, filleted_edge_id, radius, tol)?;

    let (front_edge0, back_edge0) = {
        let fillet_edge_id = simple_fillet.fillet.absolute_boundaries()[0][0].id();
        find_adjacent_edge(&simple_fillet.face0, fillet_edge_id)?
    };
    let (front_edge1, back_edge1) = {
        let fillet_edge_id = simple_fillet.fillet.absolute_boundaries()[0][2].id();
        find_adjacent_edge(&simple_fillet.face1, fillet_edge_id)?
    };

    let is_filleted_edge = |edge: &Edge<Point3, C>| edge.id() == filleted_edge_id;
    let filleted_edge = face0.edge_iter().find(is_filleted_edge)?;
    let (v0, v1) = filleted_edge.ends();

    let fillet_surface = simple_fillet.fillet.surface();
    let new_side0 = side0.and_then(|side0| {
        let fillet_edge = &simple_fillet.fillet.absolute_boundaries()[0][1];
        create_new_side(
            side0,
            fillet_edge,
            &fillet_surface,
            v0.id(),
            &front_edge0,
            &back_edge1,
        )
    });
    let new_side1 = side1.and_then(|side1| {
        let fillet_edge = &simple_fillet.fillet.absolute_boundaries()[0][3];
        create_new_side(
            side1,
            fillet_edge,
            &fillet_surface,
            v1.id(),
            &front_edge1,
            &back_edge0,
        )
    });
    Some(FilletWithSide {
        simple_fillet,
        side0: new_side0,
        side1: new_side1,
    })
}

#[derive(Clone, Debug, PartialEq)]
struct EdgesConnectingToVertex<P, C> {
    side0: Option<Edge<P, C>>,
    side1: Option<Edge<P, C>>,
}
type Ectv<P, C> = EdgesConnectingToVertex<P, C>;

#[derive(Clone, Debug, PartialEq)]
struct FacesConnectingToEdge<'a, P, C, S> {
    side0: &'a Face<P, C, S>,
    side1: &'a Face<P, C, S>,
}
type Fcte<'a, P, C, S> = FacesConnectingToEdge<'a, P, C, S>;

struct CharactersOrganization<'a, P, C, S> {
    ectvs: Vec<EdgesConnectingToVertex<P, C>>,
    fctes: Vec<FacesConnectingToEdge<'a, P, C, S>>,
}

impl<'a, P, C, S> CharactersOrganization<'a, P, C, S> {
    fn with_capacity(n: usize) -> Self {
        Self {
            ectvs: Vec::with_capacity(n + 1),
            fctes: Vec::with_capacity(n),
        }
    }
}

fn organize_characters<'a, P, C, S>(
    shell: &'a Shell<P, C, S>,
    wire: &Wire<P, C>,
) -> Option<CharactersOrganization<'a, P, C, S>> {
    if wire.is_empty() || !wire.is_simple() {
        eprintln!("wire must be non-empty and simple");
        return None;
    }

    let mut co = CharactersOrganization::with_capacity(wire.len());
    let CharactersOrganization { ectvs, fctes } = &mut co;

    for (prev_edge, edge, next_edge) in wire.edge_iter().circular_tuple_windows() {
        let mut prev_side0_edge = None;
        let mut prev_side1_edge = None;
        let mut next_side0_edge = None;
        let mut next_side1_edge = None;
        let mut side0_face = None;
        let mut side1_face = None;
        for face in shell.face_iter() {
            for (prev, current, next) in face
                .boundaries()
                .iter()
                .flat_map(|vec| vec.iter().circular_tuple_windows())
            {
                if edge == current {
                    side0_face = Some(face);
                    if !prev_edge.is_same(&prev) || prev_edge.back() != edge.front() {
                        prev_side0_edge = Some(prev.clone());
                    }
                    if !next_edge.is_same(&next) || next_edge.front() != edge.back() {
                        next_side0_edge = Some(next.inverse());
                    }
                } else if edge.is_same(&current) {
                    side1_face = Some(face);
                    if !prev_edge.is_same(&next) || prev_edge.back() != edge.front() {
                        prev_side1_edge = Some(next.clone());
                    }
                    if !next_edge.is_same(&prev) || next_edge.front() != edge.back() {
                        next_side1_edge = Some(prev.inverse());
                    }
                }
            }
        }
        ectvs.push(Ectv {
            side0: prev_side0_edge,
            side1: prev_side1_edge,
        });
        if next_edge.front() != edge.back() {
            ectvs.push(Ectv {
                side0: next_side0_edge,
                side1: next_side1_edge,
            });
        }
        fctes.push(Fcte {
            side0: side0_face?,
            side1: side1_face?,
        });
    }
    ectvs.rotate_right(1);
    fctes.rotate_right(1);

    Some(co)
}

fn fillet_faces<C, S, R>(
    wire: &Wire<Point3, C>,
    fctes: &[Fcte<'_, Point3, C, S>],
    radius: R,
    tol: f64,
) -> Option<Shell<Point3, C, S>>
where
    C: FilletedCurve<S>,
    S: FilletedSurface<C>,
    R: RadiusFunction,
    PCurve<BSplineCurve<Point2>, S>: ToSameGeometry<C>,
    NurbsCurve<Vector4>: ToSameGeometry<C>,
    ApproxFilletSurface<S, S>: ToSameGeometry<S>,
{
    let mut prev_edge: Option<Edge<Point3, C>> = None;
    let mut prev_radius: Option<f64> = None;
    let closure = |(edge, &Fcte { side0, side1 }): (&Edge<Point3, C>, _)| {
        let curve = edge.oriented_curve();
        let range = curve.range_tuple();
        let surface0 = side0.oriented_surface();
        let surface1 = side1.oriented_surface();

        if let Some(r0) = prev_radius {
            let (t0, t1) = curve.range_tuple();
            if !r0.near(&radius.subs(t0)) {
                return None;
            }
            prev_radius = Some(radius.subs(t1));
        }

        let strict_fillet = RbfSurface::new(curve, surface0, surface1, &radius);
        let surface = ApproxFilletSurface::approx_rolling_ball_fillet(&strict_fillet, range, tol)?;

        let ((u0, u1), (v0, v1)) = surface.range_tuple();

        let edge0 = prev_edge.take().unwrap_or_else(|| {
            let nurbs0 = surface.fillet_bezier(v0);
            let v0 = &Vertex::new(nurbs0.subs(u0));
            let v1 = &Vertex::new(nurbs0.subs(u1));
            Edge::new(v0, v1, nurbs0.to_same_geometry())
        });
        let mut edge2 = {
            let nurbs1 = surface.fillet_bezier(v1);
            let v0 = &Vertex::new(nurbs1.subs(u0));
            let v1 = &Vertex::new(nurbs1.subs(u1));
            Edge::new(v0, v1, nurbs1.to_same_geometry())
        };
        prev_edge = Some(edge2.clone());

        let pcurve1 = surface.side_pcurve1().to_same_geometry();
        let edge1 = Edge::new(edge0.back(), edge2.back(), pcurve1);
        let pcurve0 = surface.side_pcurve0().to_same_geometry();
        let mut edge3 = Edge::new(edge0.front(), edge2.front(), pcurve0);

        edge2.invert();
        edge3.invert();
        let boundary = wire![edge0, edge1, edge2, edge3];
        Some(Face::new(vec![boundary], surface.to_same_geometry()))
    };
    wire.edge_iter().zip(fctes).map(closure).collect()
}

//mod experiment;
mod extend_intersection_curve;

#[cfg(test)]
mod tests;
