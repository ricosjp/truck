use crate::*;
use std::f64::consts::PI;

pub(super) fn circle_arc_by_three_points(
    point0: Vector4,
    point1: Vector4,
    transit: Point3,
) -> BSplineCurve<Vector4> {
    let pt0 = Point3::from_homogeneous(point0);
    let pt1 = Point3::from_homogeneous(point1);
    let origin = circum_center(pt0, pt1, transit);
    let vec0 = pt0 - transit;
    let vec1 = pt1 - transit;
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
) -> BSplineCurve<Vector4> {
    let tmp = Point3::from_homogeneous(point);
    let origin = origin + (axis.dot(tmp - origin)) * axis;
    let axis_trsf = if !Tolerance::near(&(axis[2] * axis[2]), &1.0) {
        let axis_angle = Rad(axis[2].acos());
        let mut axis_axis = Vector3::new(-axis[1], axis[0], 0.0);
        axis_axis /= axis_axis.magnitude();
        Matrix4::from_translation(origin.to_vec()) * Matrix4::from_axis_angle(axis_axis, axis_angle)
    } else if axis[2] > 0.0 {
        Matrix4::from_translation(origin.to_vec())
    } else {
        Matrix4::from_translation(origin.to_vec())
            * Matrix4::from_axis_angle(Vector3::unit_y(), Rad(PI))
    };
    let trsf_inverse = axis_trsf.invert().unwrap();
    let rotation = Matrix4::from_angle_z(angle / 2.0);
    let rotation2 = axis_trsf * rotation * rotation;
    let cos = (angle / 2.0).cos();
    let pt = trsf_inverse * point;
    let mut point1 = rotation * pt;
    point1[3] *= cos;
    point1 = axis_trsf * point1;
    let mut curve = BSplineCurve::new(KnotVec::bezier_knot(2), vec![point, point1, rotation2 * pt]);
    curve.add_knot(0.25);
    curve.add_knot(0.5);
    curve.add_knot(0.75);
    curve
}

fn closed_polyline_orientation<'a>(pts: impl IntoIterator<Item = &'a Vec<Point3>>) -> bool {
    pts.into_iter()
        .flat_map(|vec| vec.windows(2))
        .fold(0.0, |sum, p| {
            sum + (p[1][0] + p[0][0]) * (p[1][1] - p[0][1])
        })
        >= 0.0
}

pub(super) fn attach_plane(mut pts: Vec<Vec<Point3>>) -> Option<Plane> {
    let center = pts
        .iter()
        .flatten()
        .fold(Point3::origin(), |sum, pt| sum + pt.to_vec())
        / pts.len() as f64;
    let normal = pts
        .iter()
        .flat_map(|vec| vec.windows(2))
        .fold(Vector3::zero(), |sum, p| {
            sum + (p[0] - center).cross(p[1] - center)
        });
    let n = match normal.so_small() {
        true => return None,
        false => normal.normalize(),
    };
    let a = match (n[2].abs() - 1.0).so_small() {
        true => Vector3::new(0.0, n[2], -n[1]).normalize(),
        false => Vector3::new(n[1], -n[0], 0.0).normalize(),
    };
    let mat: Matrix4 = Matrix3::from_cols(a, n.cross(a), n).into();
    pts.iter_mut()
        .flatten()
        .for_each(|pt| *pt = mat.invert().unwrap().transform_point(*pt));
    let bnd_box: BoundingBox<Point3> = pts.iter().flatten().collect();
    let diag = bnd_box.diagonal();
    if !diag[2].so_small() {
        return None;
    }
    let (max, min) = match closed_polyline_orientation(&pts) {
        true => (bnd_box.max(), bnd_box.min()),
        false => (bnd_box.min(), bnd_box.max()),
    };
    let plane = Plane::new(
        Point3::new(min[0], min[1], min[2]),
        Point3::new(max[0], min[1], min[2]),
        Point3::new(min[0], max[1], min[2]),
    )
    .transformed(mat);
    Some(plane)
}

#[cfg(test)]
mod geom_impl_test {
    use super::*;
    use rand::random;

    fn random_array<T: Default + AsMut<[f64]>>(inf: f64, sup: f64) -> T {
        let mut a = T::default();
        for s in a.as_mut() {
            *s = inf + (sup - inf) * random::<f64>();
        }
        a
    }

    #[test]
    fn circle_arc_test0() {
        use rand::random;
        let origin = Point3::from(random_array::<[f64; 3]>(-1.0, 1.0));
        let axis = Vector3::from(random_array::<[f64; 3]>(-1.0, 1.0)).normalize();
        let angle = Rad(random::<f64>() * 1.5 * PI);
        let pt0 = Point3::from(random_array::<[f64; 3]>(-1.0, 1.0));
        let curve = circle_arc(pt0.to_homogeneous(), origin, axis, angle);
        const N: usize = 100;
        let vec0 = pt0 - origin;
        for i in 0..=N {
            let t = i as f64 / N as f64;
            let pt = Point3::from_homogeneous(curve.subs(t));
            let vec = pt - origin;
            assert!(
                Tolerance::near2(&vec.dot(axis), &vec0.dot(axis)),
                "origin: {:?}\naxis: {:?}\nangle: {:?}\npt0: {:?}",
                origin,
                axis,
                angle,
                pt0
            );
        }
    }

    #[test]
    fn circle_arc_test1() {
        let origin = Point3::from(random_array::<[f64; 3]>(-1.0, 1.0));
        let axis = Vector3::unit_z();
        let angle = Rad(random::<f64>() * 1.5 * PI);
        let pt0 = Point3::from(random_array::<[f64; 3]>(-1.0, 1.0));
        let curve = circle_arc(pt0.to_homogeneous(), origin, axis, angle);
        const N: usize = 100;
        let vec0 = pt0 - origin;
        for i in 0..=N {
            let t = i as f64 / N as f64;
            let pt = Point3::from_homogeneous(curve.subs(t));
            let vec = pt - origin;
            assert!(
                Tolerance::near2(&vec.dot(axis), &vec0.dot(axis)),
                "origin: {:?}\naxis: {:?}\nangle: {:?}\npt0: {:?}",
                origin,
                axis,
                angle,
                pt0
            );
        }
    }

    #[test]
    fn circle_arc_test2() {
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
            assert_near2!(vec.dot(axis), vec0.dot(axis));
            assert!(pt[1] >= 0.0, "angle: {:?}", angle);
        }
    }

    #[test]
    fn attach_plane_test0() {
        const N: usize = 10;
        let pt = Point3::new(1.0, 0.0, 0.0);
        let c = Point3::new(0.0, 2.0 * random::<f64>() - 1.0, 0.0);
        let axis = Vector3::new(c[1], 1.0, 0.0).normalize();

        let pts0 = (0..=N)
            .map(|i| {
                let angle = Rad(2.0 * PI * i as f64 / N as f64);
                let rot = Matrix3::from_axis_angle(axis, angle);
                pt + rot * (pt - c)
            })
            .collect();
        let mid = pt.midpoint(c);
        let pts1 = (0..=N)
            .map(|i| {
                let angle = Rad(2.0 * PI * (N * 3 / 2 - i) as f64 / N as f64);
                let rot = Matrix3::from_axis_angle(axis, angle);
                pt + rot * (mid - c)
            })
            .collect();
        let mut pts = vec![pts0, pts1];
        let surface = attach_plane(pts.clone()).unwrap();
        let n = surface.normal();
        assert!(
            n.near(&axis),
            "rotation axis: {:?}\nsurface normal: {:?}",
            axis,
            n
        );
        pts.iter_mut().for_each(|vec| vec.reverse());
        let surface = attach_plane(pts).unwrap();
        let n = surface.normal();
        assert!(
            (-n).near(&axis),
            "inversed failed: rotation axis: {:?}\nsurface normal: {:?}",
            axis,
            n
        );
    }

    #[test]
    fn attach_plane_test1() {
        const N: usize = 10;
        let pt = Point3::new(1.0, 0.0, 0.0);
        let c = Point3::new(0.0, 0.0, 0.0);
        let axis = Vector3::unit_z();

        let pts0 = (0..=N)
            .map(|i| {
                let angle = Rad(2.0 * PI * i as f64 / N as f64);
                let rot = Matrix3::from_axis_angle(axis, angle);
                pt + rot * (pt - c)
            })
            .collect();
        let mid = pt.midpoint(c);
        let pts1 = (0..=N)
            .map(|i| {
                let angle = Rad(2.0 * PI * (N * 3 / 2 - i) as f64 / N as f64);
                let rot = Matrix3::from_axis_angle(axis, angle);
                mid + rot * (mid - c)
            })
            .collect();
        let mut pts = vec![pts0, pts1];
        let surface = attach_plane(pts.clone()).unwrap();
        let n = surface.normal();
        assert!(
            n.near(&axis),
            "rotation axis: {:?}\nsurface normal: {:?}",
            axis,
            n
        );
        pts.iter_mut().for_each(|vec| vec.reverse());
        let surface = attach_plane(pts).unwrap();
        let n = surface.normal();
        assert!(
            (-n).near(&axis),
            "inversed failed: rotation axis: {:?}\nsurface normal: {:?}",
            axis,
            n
        );
    }

    #[test]
    fn attach_plane_test2() {
        const N: usize = 10;
        let pt = Point3::new(1.0, 0.0, 0.0);
        let c = Point3::new(0.0, 0.0, 0.0);
        let axis = -Vector3::unit_z();

        let pts0 = (0..=N)
            .map(|i| {
                let angle = Rad(2.0 * PI * i as f64 / N as f64);
                let rot = Matrix3::from_axis_angle(axis, angle);
                pt + rot * (pt - c)
            })
            .collect();
        let mid = pt.midpoint(c);
        let pts1 = (0..=N)
            .map(|i| {
                let angle = Rad(2.0 * PI * (N * 3 / 2 - i) as f64 / N as f64);
                let rot = Matrix3::from_axis_angle(axis, angle);
                mid + rot * (mid - c)
            })
            .collect();
        let mut pts = vec![pts0, pts1];
        let surface = attach_plane(pts.clone()).unwrap();
        let n = surface.normal();
        assert!(
            n.near(&axis),
            "rotation axis: {:?}\nsurface normal: {:?}",
            axis,
            n
        );
        pts.iter_mut().for_each(|vec| vec.reverse());
        let surface = attach_plane(pts).unwrap();
        let n = surface.normal();
        assert!(
            (-n).near(&axis),
            "inversed failed: rotation axis: {:?}\nsurface normal: {:?}",
            axis,
            n
        );
    }
}
