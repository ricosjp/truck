use crate::Transform;
use geometry::*;

pub(super) fn line(pt0: Vector, pt1: Vector) -> BSplineCurve {
    let knot_vec = KnotVec::bezier_knot(1);
    BSplineCurve::new_unchecked(knot_vec, vec![pt0, pt1])
}

pub(super) fn circle_arc(pt0: Vector, pt1: Vector, transit: &Vector3) -> BSplineCurve {
    let pt0 = Vector3::new(pt0[0], pt0[1], pt0[2]);
    let pt1 = Vector3::new(pt1[0], pt1[1], pt1[2]);
    // circum_center
    let org = circum_center(&pt0, &pt1, transit);
    let trsf0 = Transform::translate(&org);

    // scalar component
    let scalar = (&pt0 - &org).norm();
    let trsf1 = Transform::scale(&Vector3::new(scalar, scalar, scalar));

    // the cosine of half of the angle of arc
    let cos = -(&pt0 - transit).cos_angle(&(&pt1 - transit));
    let mut curve = unit_circle_arc(cos);

    // orthogonal part
    let vec0 = &pt1 - &pt0;
    let mid = (&pt0 + &pt1) / 2.0;
    let vec1 = transit - mid;
    let mut radius = &vec1 - (&vec0 * &vec1) / (&vec0 * &vec0) * vec0;
    radius /= radius.norm();
    let mut n = (&pt1 - transit) ^ (&pt0 - transit);
    n /= n.norm();
    let trsf2 = Transform::by_axes(&radius, &(&n ^ &radius), &n);

    let trsf = trsf1 * trsf2 * trsf0;
    curve *= trsf.0;
    curve
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

fn unit_circle_arc(cos: f64) -> BSplineCurve {
    let knot_vec = KnotVec::bezier_knot(2);
    let sin = (1.0 - cos * cos).sqrt();
    let control_points = vec![
        Vector::new(cos, -sin, 0.0, 1.0),
        Vector::new(1.0, 0.0, 0.0, cos),
        Vector::new(cos, sin, 0.0, 1.0),
    ];
    BSplineCurve::new_unchecked(knot_vec, control_points)
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
) -> BSplineSurface {
    let trsf = Transform::translate(&-origin)
        * Transform::rotate(axis, angle / 2.0) 
        * Transform::translate(&origin);
    let trsf2 = &trsf * &trsf;
    let cos = (angle / 2.0).cos();
    let knot_vec0 = KnotVec::bezier_knot(3);
    let knot_vec1 = curve.knot_vec().clone();
    let mut control_points = Vec::new();
    for point in curve.control_points() {
        let mut row = vec![point.clone()];
        let mut point1 = point * &trsf.0;
        point1[3] *= cos;
        row.push(point1);
        row.push(point * &trsf2.0);
        control_points.push(row);
    }
    BSplineSurface::new((knot_vec0, knot_vec1), control_points)
}
