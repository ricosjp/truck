use crate::*;
use geometry::KnotVec;
use std::f64::consts::PI;

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
    let angle = Rad(PI) - vec0.angle(vec1);
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
    let axis_trsf = if !Tolerance::near(&(axis[2] * axis[2]), &1.0) {
        let axis_angle = cgmath::Rad(axis[2].acos());
        let mut axis_axis = Vector3::new(-axis[1], axis[0], 0.0);
        axis_axis /= axis_axis.magnitude();
        Matrix4::from_translation(origin.to_vec()) * Matrix4::from_axis_angle(axis_axis, axis_angle)
    } else if axis[2] > 0.0 {
        Matrix4::from_translation(origin.to_vec())
    } else {
        Matrix4::from_translation(origin.to_vec()) * Matrix4::from_axis_angle(Vector3::unit_y(), Rad(PI))
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
    curve.add_knot(0.25);
    curve.add_knot(0.5);
    curve.add_knot(0.75);
    curve
}

pub(super) fn rsweep_surface(
    curve: &BSplineCurve,
    origin: Point3,
    axis: Vector3,
    angle: Rad<f64>,
) -> BSplineSurface
{
    let knot_vec0 = curve.knot_vec().clone();
    let knot_vec1 = KnotVec::try_from(vec![0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0]).unwrap();
    let mut control_points = Vec::new();
    for point in curve.control_points() {
        let curve = circle_arc(*point, origin, axis, angle);
        control_points.push(curve.control_points().clone());
    }
    BSplineSurface::new((knot_vec0, knot_vec1), control_points)
}

#[test]
fn circle_arc_test0() {
    use rand::random;
    let origin = Point3::new(
        2.0 * random::<f64>() - 1.0,
        2.0 * random::<f64>() - 1.0,
        2.0 * random::<f64>() - 1.0,
    );
    let axis = Vector3::new(
        2.0 * random::<f64>() - 1.0,
        2.0 * random::<f64>() - 1.0,
        2.0 * random::<f64>() - 1.0,
    ).normalize();
    let angle = Rad(random::<f64>() * 2.0 * PI);
    let pt0 = Point3::new(
        2.0 * random::<f64>() - 1.0,
        2.0 * random::<f64>() - 1.0,
        2.0 * random::<f64>() - 1.0,
    );
    let curve = circle_arc(pt0.to_homogeneous(), origin, axis, angle);
    
    const N: usize = 100;
    let vec0 = pt0 - origin;
    for i in 0..=N {
        let t = i as f64 / N as f64;
        let pt = Point3::from_homogeneous(curve.subs(t));
        let vec = pt - origin;
        assert!(Tolerance::near2(&vec.dot(axis), &vec0.dot(axis)));
    }
}

#[test]
fn circle_arc_test1() {
    use rand::random;
    let origin = Point3::new(
        2.0 * random::<f64>() - 1.0,
        2.0 * random::<f64>() - 1.0,
        2.0 * random::<f64>() - 1.0,
    );
    let axis = Vector3::unit_z();
    let angle = Rad(random::<f64>() * 2.0 * PI);
    let pt0 = Point3::new(
        2.0 * random::<f64>() - 1.0,
        2.0 * random::<f64>() - 1.0,
        2.0 * random::<f64>() - 1.0,
    );
    let curve = circle_arc(pt0.to_homogeneous(), origin, axis, angle);
    
    const N: usize = 100;
    let vec0 = pt0 - origin;
    for i in 0..=N {
        let t = i as f64 / N as f64;
        let pt = Point3::from_homogeneous(curve.subs(t));
        let vec = pt - origin;
        Tolerance::assert_near2(&vec.dot(axis), &vec0.dot(axis));
    }
}

#[test]
fn circle_arc_test2() {
    use rand::random;
    let origin = Point3::origin();
    let axis = Vector3::unit_z();
    let angle = Rad(random::<f64>() * PI);
    let pt0 = Point3::new(1.4, 0.0, 0.0);
    let curve = circle_arc(pt0.to_homogeneous(), origin, axis, angle);
    
    const N: usize = 100;
    let vec0 = pt0 - origin;
    for i in 0..=N {
        let t = i as f64 / N as f64;
        let pt = Point3::from_homogeneous(curve.subs(t));
        let vec = pt - origin;
        Tolerance::assert_near2(&vec.dot(axis), &vec0.dot(axis));
        assert!(pt[1] >= 0.0);
    }
}