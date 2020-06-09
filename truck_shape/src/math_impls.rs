use crate::Transform;
use geometry::*;

pub(super) fn line(pt0: Vector, pt1: Vector) -> BSplineCurve {
    let knot_vec = KnotVec::bezier_knot(1);
    BSplineCurve::new_unchecked(knot_vec, vec![pt0, pt1])
}

pub(super) fn circle_arc_by_three_points(
    point0: Vector,
    point1: Vector,
    transit: &Vector3,
) -> BSplineCurve
{
    let pt0 = Vector3::new(point0[0], point0[1], point0[2]);
    let pt1 = Vector3::new(point1[0], point1[1], point1[2]);
    let origin = circum_center(&pt0, &pt1, transit);
    let vec0 = &pt0 - transit;
    let vec1 = &pt1 - transit;
    let angle = std::f64::consts::PI - vec0.angle(&vec1);
    let mut axis = &vec1 ^ &vec0;
    axis /= axis.norm();
    let control_points = circle_arc_control_pts(&point0, &origin, &axis, angle * 2.0);
    BSplineCurve::new(KnotVec::bezier_knot(2), control_points)
}

fn circum_center(pt0: &Vector3, pt1: &Vector3, pt2: &Vector3) -> Vector3 {
    let vec0 = pt1 - pt0;
    let vec1 = pt2 - pt0;
    let a2 = &vec0 * &vec0;
    let ab = &vec0 * &vec1;
    let b2 = &vec1 * &vec1;
    let det = a2 * b2 - ab * ab;
    let u = (b2 * a2 - ab * b2) / (2.0 * det);
    let v = (-ab * a2 + b2 * a2) / (2.0 * det);
    pt0 + u * vec0 + v * vec1
}

pub(super) fn circle_arc(
    point: &Vector,
    origin: &Vector3,
    axis: &Vector3,
    angle: f64,
) -> BSplineCurve
{
    BSplineCurve::new(
        KnotVec::bezier_knot(2),
        circle_arc_control_pts(point, origin, axis, angle),
    )
}

fn circle_arc_control_pts(
    point: &Vector,
    origin: &Vector3,
    axis: &Vector3,
    angle: f64,
) -> Vec<Vector>
{
    let axis_trsf = if (axis[2] * axis[2]).near(&1.0) {
        Transform::identity()
    } else {
        let axis_angle = axis[2].acos();
        let mut axis_axis = Vector3::new(-axis[1], axis[0], 0.0);
        axis_axis /= axis_axis.norm();
        Transform::rotate(&axis_axis, axis_angle) * Transform::translate(origin)
    };
    let point = point * axis_trsf.inverse().unwrap().0;
    let rotation = Transform::rotate(&Vector3::new(0, 0, 1), angle / 2.0);
    let mut res = vec![&point * &axis_trsf.0];
    let mut point1 = &point * &rotation.0;
    point1[3] *= (angle / 2.0).cos();
    res.push(point1 * &axis_trsf.0);
    res.push(point * (&rotation * &rotation).0 * axis_trsf.0);
    res
}

pub(super) fn plane_by_two_curves(
    mut curve0: BSplineCurve,
    mut curve2: BSplineCurve,
) -> BSplineSurface
{
    let t = curve0.knot_vec()[0] + curve0.knot_vec().range_length() / 2.0;
    let curve1 = curve0.cut(t);
    let t = curve2.knot_vec()[0] + curve2.knot_vec().range_length() / 2.0;
    let curve3 = curve2.cut(t);
    BSplineSurface::by_boundary(curve0, curve1, curve2, curve3)
}

pub(super) fn plane_by_three_curves(
    curve0: BSplineCurve,
    curve1: BSplineCurve,
    curve3: BSplineCurve,
) -> BSplineSurface
{
    let curve2 = line(curve1.end_points().1, curve3.end_points().0);
    BSplineSurface::by_boundary(curve0, curve1, curve2, curve3)
}

pub(super) fn rsweep_surface(
    curve: &BSplineCurve,
    origin: &Vector3,
    axis: &Vector3,
    angle: f64,
) -> BSplineSurface
{
    let knot_vec0 = curve.knot_vec().clone();
    let knot_vec1 = KnotVec::bezier_knot(3);
    let mut control_points = Vec::new();
    for point in curve.control_points() {
        control_points.push(circle_arc_control_pts(point, origin, axis, angle));
    }
    BSplineSurface::new((knot_vec0, knot_vec1), control_points)
}
