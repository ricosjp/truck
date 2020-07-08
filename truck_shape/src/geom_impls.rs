use crate::*;
use geometry::{Vector3, KnotVec, Tolerance};
use std::f64::consts::PI;

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
    let tmp = point0.rational_projection();
    let pt0 = vector!(tmp[0], tmp[1], tmp[2]);
    let tmp = point1.rational_projection();
    let pt1 = vector!(tmp[0], tmp[1], tmp[2]);
    let origin = circum_center(&pt0, &pt1, transit);
    let vec0 = &pt0 - transit;
    let vec1 = &pt1 - transit;
    let angle = PI - vec0.angle(&vec1);
    let mut axis = &vec1 ^ &vec0;
    axis /= axis.norm();
    circle_arc(&point0, &origin, &axis, angle * 2.0)
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
    let tmp: Vector3 = point.rational_projection().into();
    let origin = origin + (axis * (tmp - origin)) * axis;
    let axis_trsf = if (axis[2] * axis[2]).near(&1.0) {
        Transform::identity()
    } else {
        let axis_angle = axis[2].acos();
        let mut axis_axis = vector!(-axis[1], axis[0], 0.0);
        axis_axis /= axis_axis.norm();
        Transform::rotate(&axis_axis, axis_angle) * Transform::translate(&origin)
    };
    let trsf_inverse = &axis_trsf.inverse().unwrap();
    let rotation = Transform::rotate(&vector!(0, 0, 1), angle / 2.0);
    let rotation2 = &rotation * &rotation * &axis_trsf;
    let cos = (angle / 2.0).cos();
    let pt = point * &trsf_inverse.0;
    let mut point1 = &pt * &rotation.0;
    point1[3] *= cos;
    point1 *= &axis_trsf;
    let mut curve = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![point.clone(), point1, pt * rotation2.0]
    );
    curve.add_knot(0.5);
    curve
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
    let knot_vec1 = KnotVec::try_from(vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0]).unwrap();
    let mut control_points = Vec::new();
    for point in curve.control_points() {
        let curve = circle_arc(point, origin, axis, angle);
        control_points.push(curve.control_points().clone());
    }
    BSplineSurface::new((knot_vec0, knot_vec1), control_points)
}

pub(super) fn bezier_curve(control_points: Vec<Vector3>) -> BSplineCurve {
    let knot_vec = KnotVec::bezier_knot(control_points.len() - 1);
    let control_points = control_points.into_iter().map(|pt| {
        rvector!(pt[0], pt[1], pt[2])
    }).collect();
    BSplineCurve::new(knot_vec, control_points)
}
