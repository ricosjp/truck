use crate::*;
use std::f64::consts::PI;
use cgmath::Rad;
pub(super) fn line(pt0: Vector4, pt1: Vector4) -> BSplineCurve {
    let knot_vec = KnotVec::bezier_knot(1);
    BSplineCurve::new_unchecked(knot_vec, vec![pt0, pt1])
}

pub(super) fn circle_arc_by_three_points(
    point0: Vector4,
    point1: Vector4,
    transit: Point3,
) -> BSplineCurve
{
    let pt0 = Point3::from_homogeneous(point0);
    let pt1 = Point3::from_homogeneous(point1);
    let origin = circum_center(pt0, pt1, transit);
    let vec0 = &pt0 - transit;
    let vec1 = &pt1 - transit;
    let angle = cgmath::Rad(PI) - vec0.angle(vec1);
    let mut axis = vec1.cross(vec0);
    axis /= axis.magnitude();
    circle_arc(point0, origin, axis, angle * 2.0)
}

fn circum_center(pt0: Point3, pt1: Point3, pt2: Point3) -> Point3 {
    let vec0 = pt1 - pt0;
    let vec1 = pt2 - pt0;
    let a2 = vec0.dot(vec0);
    let ab = vec0.dot(vec1);
    let b2 = vec1.dot(vec1);
    let det = a2 * b2 - ab * ab;
    let u = (b2 * a2 - ab * b2) / (2.0 * det);
    let v = (-ab * a2 + b2 * a2) / (2.0 * det);
    pt0 + u * vec0 + v * vec1
}

pub(super) fn circle_arc(
    point: Vector4,
    origin: Point3,
    axis: Vector3,
    angle: Rad<f64>,
) -> BSplineCurve
{
    let tmp = Point3::from_homogeneous(point);
    let origin = origin + (axis.dot(tmp - origin)) * axis;
    let axis_trsf = if (axis[2] * axis[2]).near(&1.0) {
        Matrix4::identity()
    } else {
        let axis_angle = cgmath::Rad(axis[2].acos());
        let mut axis_axis = Vector3::new(-axis[1], axis[0], 0.0);
        axis_axis /= axis_axis.magnitude();
        Matrix4::from_translation(origin.to_vec()) * Matrix4::from_axis_angle(axis_axis, axis_angle)
    };
    let trsf_inverse = axis_trsf.invert().unwrap();
    let rotation = Matrix4::from_angle_z(angle / 2.0);
    let rotation2 = &axis_trsf * &rotation * &rotation;
    let cos = (angle / 2.0).cos();
    let pt = &trsf_inverse * point;
    let mut point1 = &rotation * pt;
    point1[3] *= cos;
    point1 = &axis_trsf * point1;
    let mut curve = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![point.clone(), point1, rotation2 * pt]
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
    origin: Point3,
    axis: Vector3,
    angle: Rad<f64>,
) -> BSplineSurface
{
    let knot_vec0 = curve.knot_vec().clone();
    let knot_vec1 = KnotVec::try_from(vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0]).unwrap();
    let mut control_points = Vec::new();
    for point in curve.control_points() {
        let curve = circle_arc(*point, origin, axis, angle);
        control_points.push(curve.control_points().clone());
    }
    BSplineSurface::new((knot_vec0, knot_vec1), control_points)
}

pub(super) fn bezier_curve(control_points: Vec<Vector3>) -> BSplineCurve {
    let knot_vec = KnotVec::bezier_knot(control_points.len() - 1);
    let control_points = control_points.into_iter().map(|pt| {
        Vector4::new(pt[0], pt[1], pt[2], 1.0)
    }).collect();
    BSplineCurve::new(knot_vec, control_points)
}
