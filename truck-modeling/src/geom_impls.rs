use crate::*;
use geometry::KnotVec;
use std::f64::consts::PI;
type BSplineCurve = truck_geometry::BSplineCurve<Vector4>;
type BSplineSurface = truck_geometry::BSplineSurface<Vector4>;

pub(super) fn line(pt0: Vector4, pt1: Vector4) -> BSplineCurve {
    let knot_vec = KnotVec::bezier_knot(1);
    BSplineCurve::new_unchecked(knot_vec, vec![pt0, pt1])
}

pub(super) fn circle_arc_by_three_points(
    point0: Vector4,
    point1: Vector4,
    transit: Point3,
) -> BSplineCurve {
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
) -> BSplineCurve {
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
    let rotation2 = &axis_trsf * &rotation * &rotation;
    let cos = (angle / 2.0).cos();
    let pt = &trsf_inverse * point;
    let mut point1 = &rotation * pt;
    point1[3] *= cos;
    point1 = &axis_trsf * point1;
    let mut curve = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![point.clone(), point1, rotation2 * pt],
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
) -> BSplineSurface {
    let knot_vec0 = curve.knot_vec().clone();
    let knot_vec1 = KnotVec::try_from(vec![0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0]).unwrap();
    let mut control_points = Vec::new();
    for point in curve.control_points() {
        let curve = circle_arc(*point, origin, axis, angle);
        control_points.push(curve.control_points().clone());
    }
    BSplineSurface::new((knot_vec0, knot_vec1), control_points)
}

fn closed_polyline_orientation(pts: &Vec<Point3>) -> bool {
    pts.windows(2).fold(0.0, |sum, pt| {
        sum + (pt[1][0] + pt[0][0]) * (pt[1][1] - pt[0][1])
    }) >= 0.0
}

pub(super) fn attach_plane(mut pts: Vec<Point3>) -> Option<BSplineSurface> {
    let pt0 = pts[0];
    let pt1 = match pts.iter().find(|pt| !pt0.near(&pt)) {
        Some(got) => got,
        None => return None,
    };
    let pt2 = match pts
        .iter()
        .find(|pt| !(*pt - pt0).cross(pt1 - pt0).so_small())
    {
        Some(got) => got,
        None => return None,
    };
    let n = (pt2 - pt0).cross(pt1 - pt0).normalize();
    let mat = match n.cross(Vector3::unit_z()).so_small() {
        true => Matrix4::identity(),
        false => {
            let a = Vector3::new(n[1], -n[0], 0.0).normalize();
            let b = n.cross(a);
            Matrix3::from_cols(a, b, n).into()
        }
    };
    pts.iter_mut()
        .for_each(|pt| *pt = mat.invert().unwrap().transform_point(*pt));
    let bnd_box: BoundingBox<Point3> = pts.iter().collect();
    let diag = bnd_box.diagonal();
    if !diag[2].so_small() {
        return None;
    }
    let (max, min) = match closed_polyline_orientation(&pts) {
        true => (bnd_box.max(), bnd_box.min()),
        false => (bnd_box.min(), bnd_box.max()),
    };
    let ctrl_pts = vec![
        vec![
            mat * Vector4::new(min[0], min[1], min[2], 1.0),
            mat * Vector4::new(max[0], min[1], min[2], 1.0),
        ],
        vec![
            mat * Vector4::new(min[0], max[1], min[2], 1.0),
            mat * Vector4::new(max[0], max[1], min[2], 1.0),
        ],
    ];
    let knot_vecs = (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1));
    Some(BSplineSurface::new(knot_vecs, ctrl_pts))
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
            Tolerance::assert_near2(&vec.dot(axis), &vec0.dot(axis));
            assert!(pt[1] >= 0.0, "angle: {:?}", angle);
        }
    }

    #[test]
    fn rsweep_surface_test0() {
        let knot_vec = KnotVec::bezier_knot(3);
        let ctrl_pts = vec![
            Vector4::from(random_array::<[f64; 4]>(0.1, 100.0)),
            Vector4::from(random_array::<[f64; 4]>(0.1, 100.0)),
            Vector4::from(random_array::<[f64; 4]>(0.1, 100.0)),
            Vector4::from(random_array::<[f64; 4]>(0.1, 100.0)),
        ];
        let curve = BSplineCurve::new(knot_vec, ctrl_pts);
        let origin = Point3::from(random_array::<[f64; 3]>(-1.0, 1.0));
        let axis = Vector3::from(random_array::<[f64; 3]>(-1.0, 1.0)).normalize();
        let angle = Rad(random::<f64>() * 1.5 * PI);
        let surface = rsweep_surface(&curve, origin, axis, angle);
        let curve = NURBSCurve::new(curve);
        let surface = NURBSSurface::new(surface);
        const N: usize = 100;
        for i in 0..=N {
            let s = i as f64 / N as f64;
            for j in 0..=N {
                let t = j as f64 / N as f64;
                let vec0 = curve.subs(s) - origin;
                let vec1 = surface.subs(s, t) - origin;
                let h0 = vec0 - vec0.dot(axis) * axis;
                let h1 = vec1 - vec1.dot(axis) * axis;
                assert!(
                    f64::near(&h0.magnitude2(), &h1.magnitude2()),
                    "origin\n{:?}\naxis: {:?}\nangle: {:?}\ncurve: {:?}",
                    origin,
                    axis,
                    angle,
                    curve.non_rationalized(),
                );
            }
            let vec0 = curve.subs(s) - origin;
            let vec1 = surface.subs(s, 1.0) - origin;
            let h0 = (vec0 - vec0.dot(axis) * axis).normalize();
            let h1 = (vec1 - vec1.dot(axis) * axis).normalize();
            let axis0 = h0.cross(h1);
            let cos0 = h0.dot(h1);
            assert!(
                f64::near(&cos0, &angle.cos()) && axis0.cross(axis).so_small(),
                "origin\n{:?}\naxis: {:?}\nangle: {:?}\ncurve: {:?}",
                origin,
                axis,
                angle,
                curve.non_rationalized(),
            );
        }
    }
}
