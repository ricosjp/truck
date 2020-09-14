use crate::*;

pub fn vertex(pt: Point3) -> Vertex { Vertex::new(pt.to_homogeneous()) }

pub fn line(vertex0: &Vertex, vertex1: &Vertex) -> Edge {
    let curve = geom_impls::line(
        *vertex0.lock_point().unwrap(),
        *vertex1.lock_point().unwrap(),
    );
    Edge::new(vertex0, vertex1, curve)
}

pub fn circle_arc(vertex0: &Vertex, vertex1: &Vertex, transit: Point3) -> Edge {
    let curve = geom_impls::circle_arc_by_three_points(
        *vertex0.lock_point().unwrap(),
        *vertex1.lock_point().unwrap(),
        transit,
    );
    Edge::new(vertex0, vertex1, curve)
}

pub fn bezier(vertex0: &Vertex, vertex1: &Vertex, mut inter_points: Vec<Point3>) -> Edge {
    let pt0 = Point3::from_homogeneous(*vertex0.lock_point().unwrap());
    let pt1 = Point3::from_homogeneous(*vertex1.lock_point().unwrap());
    let mut pre_ctrl_pts = vec![pt0];
    pre_ctrl_pts.append(&mut inter_points);
    pre_ctrl_pts.push(pt1);
    let ctrl_pts: Vec<_> = pre_ctrl_pts
        .into_iter()
        .map(|pt| pt.to_homogeneous())
        .collect();
    let knot_vec = KnotVec::bezier_knot(ctrl_pts.len() - 1);
    let curve = BSplineCurve::new(knot_vec, ctrl_pts);
    Edge::new(vertex0, vertex1, curve)
}

pub fn homotopy(edge0: &Edge, edge1: &Edge) -> Face {
    let wire: Wire = vec![
        edge0.clone(),
        line(edge0.back(), edge1.front()),
        edge1.inverse(),
        line(edge1.front(), edge1.back()),
    ].into();
    let surface = BSplineSurface::homotopy(
        edge0.oriented_curve(),
        edge1.oriented_curve(),
    );
    Face::new(vec![wire], surface)
}


