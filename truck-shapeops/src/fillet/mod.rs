use algo::curve::search_intersection_parameter;
use itertools::Itertools;
use truck_geometry::prelude::{rbf_surface::*, *};
use truck_topology::*;

/// condition of curves to attach fillet
pub trait FilletedCurve<S, R>: ParametricCurve3D + BoundedCurve + Cut + Invertible + std::fmt::Debug {}
impl<C: ParametricCurve3D + BoundedCurve + Cut + Invertible + std::fmt::Debug, S, R> FilletedCurve<S, R> for C {}

/// condition of sufaces to attach fillet
pub trait FilletedSurface<C, R>:
    ParametricSurface3D + SearchParameter<D2, Point = Point3> + Invertible {
}
impl<C, S: ParametricSurface3D + SearchParameter<D2, Point = Point3> + Invertible, R>
    FilletedSurface<C, R> for S
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

fn cut_face_by_curve<C, S, R>(
    face: &Face<Point3, C, S>,
    mut curve: C,
    filleted_edge_id: EdgeID<C>,
) -> Option<(Face<Point3, C, S>, Edge<Point3, C>)>
where
    C: FilletedCurve<S, R>,
    S: Clone,
    R: RadiusFunction,
{
    let (front_edge, back_edge) = find_adjacent_edge(face, filleted_edge_id)?;

    let new_front_edge = {
        let front_curve = front_edge.curve();
        let hint = algo::curve::presearch_closest_point(
            &curve,
            &front_curve,
            (curve.range_tuple(), front_curve.range_tuple()),
            10,
        );
        let (t0, t1) = search_intersection_parameter(&curve, &front_curve, hint, 100)?;
        let v0 = Vertex::new(curve.subs(t0));
        curve = curve.cut(t0);
        front_edge.cut_with_parameter(&v0, t1)?.0
    };

    let new_back_edge = {
        let back_curve = back_edge.curve();
        let hint = algo::curve::presearch_closest_point(
            &curve,
            &back_curve,
            (curve.range_tuple(), back_curve.range_tuple()),
            10,
        );
        let (t0, t1) = search_intersection_parameter(&curve, &back_curve, hint, 100)?;
        let v1 = Vertex::new(curve.subs(t0));
        curve.cut(t0);
        back_edge.cut_with_parameter(&v1, t1)?.1
    };

    let fillet_edge = Edge::new(new_front_edge.back(), new_back_edge.front(), curve.into());
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

fn create_pcurve_edge<C, S>(
    (v0, hint0): (&Vertex<Point3>, (f64, f64)),
    (v1, hint1): (&Vertex<Point3>, (f64, f64)),
    fillet_surface: S,
) -> Option<Edge<Point3, C>>
where
    PCurve<Line<Point2>, S>: ToSameGeometry<C>,
    S: SearchParameter<D2, Point = Point3> + Clone,
{
    let uv0 = fillet_surface.search_parameter(v0.point(), hint0, 100)?;
    let uv1 = fillet_surface.search_parameter(v1.point(), hint1, 100)?;
    let curve = PCurve::new(Line(uv0.into(), uv1.into()), fillet_surface);
    Some(Edge::new(v0, v1, curve.to_same_geometry()))
}

/// create simple fillet
pub fn simple_fillet<C, S, R>(
    face0: &Face<Point3, C, S>,
    face1: &Face<Point3, C, S>,
    filleted_edge_id: EdgeID<C>,
    radius: R,
) -> Option<(Face<Point3, C, S>, Face<Point3, C, S>, Face<Point3, C, S>)>
where
    C: FilletedCurve<S, R>,
    S: FilletedSurface<C, R>,
    R: RadiusFunction,
    RbfContactCurve<C, S, S, R>: ToSameGeometry<C>,
    PCurve<Line<Point2>, S>: ToSameGeometry<C>,
    RbfSurface<C, S, S, R>: ToSameGeometry<S>,
{
    let is_filleted_edge = move |edge: &Edge<Point3, C>| edge.id() == filleted_edge_id;
    let filleted_edge = face0.edge_iter().find(is_filleted_edge)?;

    let fillet_surface = {
        let surface0 = face0.oriented_surface();
        let surface1 = face1.oriented_surface();
        let curve = filleted_edge.oriented_curve();
        RbfSurface::new(curve, surface0, surface1, radius)
    };

    let (new_face0, fillet_edge0) = {
        let contact_curve = fillet_surface.contact_curve0().to_same_geometry();
        cut_face_by_curve(face0, contact_curve, filleted_edge.id())?
    };
    let (new_face1, fillet_edge1) = {
        let contact_curve = fillet_surface.contact_curve1().to_same_geometry();
        cut_face_by_curve(face1, contact_curve.inverse(), filleted_edge.id())?
    };

    let surface = fillet_surface.to_same_geometry();
    let ((vertex0, vertex1), (vertex2, vertex3)) = (fillet_edge0.ends(), fillet_edge1.ends());
    let ((u0, u1), (v0, v1)) = fillet_surface.range_tuple();
    let edge0 = create_pcurve_edge((vertex0, (u0, v0)), (vertex3, (u1, v0)), surface.clone())?;
    let edge1 = create_pcurve_edge((vertex2, (u1, v1)), (vertex1, (u0, v1)), surface.clone())?;
    let fillet = {
        let fillet_boundary = [fillet_edge0.inverse(), edge0, fillet_edge1.inverse(), edge1];
        Face::new(vec![fillet_boundary.into()], surface)
    };

    Some((new_face0, new_face1, fillet))
}

//mod experiment;

#[cfg(test)]
mod tests;
