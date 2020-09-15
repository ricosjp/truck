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
    ]
    .into();
    let surface = BSplineSurface::homotopy(edge0.oriented_curve(), edge1.oriented_curve());
    Face::new(vec![wire], surface)
}

pub fn clone<T: Mapped<Vector4, BSplineCurve, BSplineSurface>>(elem: &T) -> T {
    elem.topological_clone()
}

pub fn transformed<T: Mapped<Vector4, BSplineCurve, BSplineSurface>>(elem: &T, mat: Matrix4) -> T {
    elem.mapped(
        &move |pt: &Vector4| mat * pt,
        &move |curve: &BSplineCurve| mat * curve,
        &move |surface: &BSplineSurface| mat * surface,
    )
}

pub fn translated<T: Mapped<Vector4, BSplineCurve, BSplineSurface>>(
    elem: &T,
    vector: Vector3,
) -> T
{
    builder::transformed(elem, Matrix4::from_translation(vector))
}

pub fn rotated<T: Mapped<Vector4, BSplineCurve, BSplineSurface>>(
    elem: &T,
    origin: Point3,
    axis: Vector3,
    angle: f64,
) -> T
{
    let mat0 = Matrix4::from_translation(-origin.to_vec());
    let mat1 = Matrix4::from_axis_angle(axis, cgmath::Rad(angle));
    let mat2 = Matrix4::from_translation(origin.to_vec());
    builder::transformed(elem, mat2 * mat1 * mat0)
}

pub fn scaled<T: Mapped<Vector4, BSplineCurve, BSplineSurface>>(
    elem: &T,
    origin: Point3,
    scalars: Vector3,
) -> T
{
    let mat0 = Matrix4::from_translation(-origin.to_vec());
    let mat1 = Matrix4::from_nonuniform_scale(scalars[0], scalars[1], scalars[2]);
    let mat2 = Matrix4::from_translation(origin.to_vec());
    builder::transformed(elem, mat2 * mat1 * mat0)
}

pub fn tsweep<T: Sweep<Vector4, BSplineCurve, BSplineSurface>>(
    elem: &T,
    vector: Vector3,
) -> T::Sweeped
{
    let trsl = Matrix4::from_translation(vector);
    elem.sweep(
        &move |pt| trsl * pt,
        &move |curve| trsl * curve,
        &move |surface| trsl * surface,
        &move |pt0, pt1| geom_impls::line(*pt0, *pt1),
        &move |curve0, curve1| BSplineSurface::homotopy(curve0.clone(), curve1.clone()),
    )
}

pub fn partial_rsweep<T: Sweep<Vector4, BSplineCurve, BSplineSurface>>(
    elem: &T,
    origin: Point3,
    axis: Vector3,
    angle: f64,
) -> T::Sweeped
{
    let angle = cgmath::Rad(angle);
    let mat0 = Matrix4::from_translation(-origin.to_vec());
    let mat1 = Matrix4::from_axis_angle(axis, angle);
    let mat2 = Matrix4::from_translation(origin.to_vec());
    let trsl = mat2 * mat1 * mat0;
    elem.sweep(
        &move |pt| trsl * pt,
        &move |curve| trsl * curve,
        &move |surface| trsl * surface,
        &move |pt, _| geom_impls::circle_arc(*pt, origin, axis, angle),
        &move |curve, _| geom_impls::rsweep_surface(curve, origin, axis, angle),
    )
}
