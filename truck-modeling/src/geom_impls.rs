use crate::*;
use itertools::Itertools;
use std::f64::consts::PI;

pub(super) fn circle_arc_by_three_points(
    point0: Point3,
    point1: Point3,
    transit: Point3,
) -> Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4> {
    let origin = circum_center(point0, point1, transit);
    let (vec0, vec1) = (point0 - transit, point1 - transit);
    let axis = vec1.cross(vec0).normalize();
    let angle = Rad(PI) - vec0.angle(vec1);
    circle_arc(point0, origin, axis, angle * 2.0)
}

fn circum_center(pt0: Point3, pt1: Point3, pt2: Point3) -> Point3 {
    let (vec0, vec1) = (pt1 - pt0, pt2 - pt0);
    let (a2, ab, b2) = (vec0.dot(vec0), vec0.dot(vec1), vec1.dot(vec1));
    let (det, u, v) = (a2 * b2 - ab * ab, a2 * b2 - ab * b2, a2 * b2 - ab * a2);
    pt0 + u / (2.0 * det) * vec0 + v / (2.0 * det) * vec1
}

pub(super) fn circle_arc(
    point: Point3,
    origin: Point3,
    axis: Vector3,
    angle: Rad<f64>,
) -> Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4> {
    let origin = origin + (axis.dot(point - origin)) * axis;
    let diag = point - origin;
    let axis_trsf = Matrix4::from_cols(
        diag.extend(0.0),
        axis.cross(diag).extend(0.0),
        axis.extend(0.0),
        origin.to_homogeneous(),
    );
    let unit_arc = TrimmedCurve::new(UnitCircle::new(), (0.0, angle.0));
    Processor::with_transform(unit_arc, axis_trsf)
}

fn closed_polyline_orientation<'a>(pts: impl IntoIterator<Item = &'a Vec<Point3>>) -> bool {
    pts.into_iter()
        .flat_map(|vec| vec.iter().circular_tuple_windows())
        .map(|(p0, p1)| (p1[0] + p0[0]) * (p1[1] - p0[1]))
        .sum::<f64>()
        >= 0.0
}

fn take_one_axis_by_normal(n: Vector3) -> Vector3 {
    let a = n.map(f64::abs);
    if a.x > a.z || a.y > a.z {
        Vector3::new(-n.y, n.x, 0.0).normalize()
    } else {
        Vector3::new(-n.z, 0.0, n.x).normalize()
    }
}

pub(super) fn attach_plane(mut pts: Vec<Vec<Point3>>) -> Option<Plane> {
    let center = pts
        .iter()
        .flatten()
        .fold(Point3::origin(), |sum, pt| sum + pt.to_vec())
        / pts.len() as f64;
    let normal = pts
        .iter()
        .flat_map(|vec| vec.iter().circular_tuple_windows())
        .fold(Vector3::zero(), |sum, (p0, p1)| {
            sum + (p0 - center).cross(p1 - center)
        });
    let n = match normal.so_small() {
        true => return None,
        false => normal.normalize(),
    };
    let a = take_one_axis_by_normal(n);
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
mod test_geom_impl {
    use super::*;
    use proptest::*;

    fn pole_to_normal(pole: [f64; 2]) -> Vector3 {
        let theta = PI * pole[0];
        let z = pole[1];
        let zi = f64::sqrt(f64::max(1.0 - z * z, 0.0));
        Vector3::new(f64::cos(theta) * zi, f64::sin(theta) * zi, z)
    }

    fn complex_boundary(angles: [f64; 10]) -> Vec<Point3> {
        let mut angle_store = 0.0;
        angles
            .into_iter()
            .enumerate()
            .flat_map(move |(i, angle)| {
                let prev_angle = angle_store;
                angle_store = angle;
                let r = 10.0 - i as f64;
                let min_theta = f64::acos(1.0 - 0.01 / r);
                let divs = 1 + (f64::abs(angle - prev_angle) / min_theta) as usize;
                (0..=divs).map(move |i| {
                    let t = i as f64 / divs as f64;
                    let theta = (1.0 - t) * prev_angle + t * angle;
                    Point3::new(r * f64::cos(theta), r * f64::sin(theta), 0.0)
                })
            })
            .chain([Point3::origin(), Point3::new(10.0, 0.0, 0.0)])
            .collect()
    }

    fn dist_square(p: Point3, a: f64, b: f64) -> f64 {
        let absp = p.map(f64::abs);
        f64::min(a - absp.x, b - absp.y)
    }

    fn multiple_boundary(points: [Point3; 4], radius_ratios: [f64; 4]) -> Vec<Vec<Point3>> {
        let mut res = vec![vec![
            Point3::new(10.0, 10.0, 0.0),
            Point3::new(-10.0, 10.0, 0.0),
            Point3::new(-10.0, -10.0, 0.0),
            Point3::new(10.0, -10.0, 0.0),
            Point3::new(10.0, 10.0, 0.0),
        ]];
        let mut radii = Vec::<f64>::new();
        res.extend((0..4).map(|i| {
            let mut dist = dist_square(points[i], 10.0, 10.0);
            (0..i).for_each(|j| {
                dist = f64::min(dist, points[i].distance(points[j]) - radii[j]);
            });
            (i + 1..4).for_each(|j| {
                dist = f64::min(dist, points[i].distance(points[j]));
            });
            radii.push(radius_ratios[i] * dist);
            (0..=10)
                .map(|j| {
                    let theta = j as f64 / 10.0 * 2.0 * PI;
                    Point3::new(f64::sin(theta), f64::cos(theta), 0.0)
                })
                .collect()
        }));
        res
    }

    proptest! {
        #[test]
        fn test_circum_center(
            p0 in array::uniform3(-10.0f64..10.0),
            p1 in array::uniform3(-10.0f64..10.0),
            p2 in array::uniform3(-10.0f64..10.0),
        ) {
            let p0 = Point3::from(p0);
            let p1 = Point3::from(p1);
            let p2 = Point3::from(p2);
            let c = circum_center(p0, p1, p2);

            // The point `c` exists at the same distance from the three points.
            let d0 = c.distance2(p0);
            let d1 = c.distance2(p1);
            let d2 = c.distance2(p2);
            assert!(d0.near(&d1) && d1.near(&d2) && d2.near(&d0));
        }

        #[test]
        fn test_circle_arc_three_point(
            p0 in array::uniform3(-10.0f64..10.0),
            p1 in array::uniform3(-10.0f64..10.0),
            p2 in array::uniform3(-10.0f64..10.0),
            t in TOLERANCE..(1.0 - TOLERANCE),
        ) {
            let p0 = Point3::from(p0);
            let p1 = Point3::from(p1);
            let p2 = Point3::from(p2);
            let curve = circle_arc_by_three_points(p0, p1, p2);

            // The curve `curve` is from `p0` to `p1`.
            assert_near!(curve.front(), p0);
            assert_near!(curve.back(), p1);

            // Any point on the curve is on the same side as point `p2`.
            // Check by the circular angle theorem.
            let (t0, t1) = curve.range_tuple();
            let p3 = curve.subs((1.0 - t) * t0 + t * t1);
            let angle2 = (p2 - p1).angle(p2 - p0);
            let angle3 = (p3 - p1).angle(p3 - p0);
            assert_near!(angle2, angle3);
        }

        #[test]
        fn test_circle_arc(
            origin in array::uniform3(-10.0f64..10.0),
            axis_pole in array::uniform2(-1.0f64..1.0),
            angle in TOLERANCE..(1.5 * PI),
            pt0 in array::uniform3(-10.0f64..10.0),
            t in TOLERANCE..(1.0 - TOLERANCE),
        ) {
            let origin = Point3::from(origin);
            let axis = pole_to_normal(axis_pole);
            let angle = Rad(angle);
            let pt0 = Point3::from(pt0);
            let curve = circle_arc(pt0, origin, axis, angle);

            // front point and back point
            let trans = Matrix4::from_translation(origin.to_vec())
                * Matrix4::from_axis_angle(axis, angle)
                * Matrix4::from_translation(-origin.to_vec());
            let pt1 = trans.transform_point(pt0);
            assert_near!(curve.front(), pt0);
            assert_near!(curve.back(), pt1);

            // Any point on the curve lies in the same plane perpendicular to the axis.
            let (t0, t1) = curve.range_tuple();
            let pt2 = curve.subs((1.0 - t) * t0 + t * t1);
            let vec0 = pt0 - origin;
            let vec2 = pt2 - origin;
            assert_near!(vec0.dot(axis), vec2.dot(axis));

            // Any point on the curve lies in the circle arc from `p0` to `p1`.
            // Check by the circular angle theorem.
            let angle0 = (pt2 - pt1).angle(pt2 - pt0);
            assert_near!(angle0 * 2.0, Rad(2.0 * PI) - angle);
        }

        #[test]
        fn test_take_one_axis_by_normal(normal in array::uniform3(-100.0f64..100.0)) {
            let normal = Vector3::from(normal);
            let axis = take_one_axis_by_normal(normal);
            assert!(normal.so_small() || (!axis.so_small() && axis.dot(normal).so_small()));
        }

        #[test]
        fn test_attach_plane_with_single_boundary(
            axis_pole in array::uniform2(-1.0f64..1.0),
            origin in array::uniform3(-10.0f64..10.0),
            angles in array::uniform10(0.01f64..(2.0 * PI - 0.01)),
        ) {
            let axis = pole_to_normal(axis_pole);
            let origin = Point3::from(origin);
            let diag = take_one_axis_by_normal(axis);
            let trsf = Matrix4::from_cols(
                diag.extend(0.0),
                axis.cross(diag).extend(0.0),
                axis.extend(0.0),
                origin.to_homogeneous(),
            );
            let boundary: Vec<_> = complex_boundary(angles)
                .into_iter()
                .map(|p| trsf.transform_point(p))
                .collect();
            let plane = attach_plane(vec![boundary]).unwrap();
            assert_near!(plane.normal(), axis);
        }

        #[test]
        fn test_attach_plane_with_multiple_boundary(
            axis_pole in array::uniform2(-1.0f64..1.0),
            origin in array::uniform3(-10.0f64..10.0),
            points in array::uniform8(1.0f64..9.0),
            radius_ratios in array::uniform4(0.1f64..0.9),
        ) {
            let axis = pole_to_normal(axis_pole);
            let origin = Point3::from(origin);
            let diag = take_one_axis_by_normal(axis);
            let trsf = Matrix4::from_cols(
                diag.extend(0.0),
                axis.cross(diag).extend(0.0),
                axis.extend(0.0),
                origin.to_homogeneous(),
            );
            let points = [
                Point3::new(points[0], points[1], 0.0),
                Point3::new(points[2] - 10.0, points[3], 0.0),
                Point3::new(points[4] - 10.0, points[5] - 10.0, 0.0),
                Point3::new(points[6], points[7] - 10.0, 0.0),
            ];
            let mut multiple_boundary = multiple_boundary(points, radius_ratios);
            multiple_boundary
                .iter_mut()
                .flatten()
                .for_each(|p| *p = trsf.transform_point(*p));
            let plane = attach_plane(multiple_boundary).unwrap();
            assert_near!(plane.normal(), axis);
        }
    }
}

impl<T: Clone> GeometricMapping<T> for () {
    #[inline]
    fn mapping(self) -> impl Fn(&T) -> T { Clone::clone }
}
impl<T: Transformed<Matrix4>> GeometricMapping<T> for Matrix4 {
    #[inline]
    fn mapping(self) -> impl Fn(&T) -> T { move |t| t.transformed(self) }
}
impl<T> GeometricMapping<T> for fn(&T) -> T {
    #[inline]
    fn mapping(self) -> impl Fn(&T) -> T { self }
}

impl<T, H> Connector<T, H> for fn(&T, &T) -> H {
    #[inline]
    fn connector(self) -> impl Fn(&T, &T) -> H { self }
}

#[derive(Debug, Clone, Copy)]
pub struct LineConnector;

impl<C> Connector<Point3, C> for LineConnector
where Line<Point3>: ToSameGeometry<C>
{
    fn connector(self) -> impl Fn(&Point3, &Point3) -> C { |p, q| Line(*p, *q).to_same_geometry() }
}

#[derive(Debug, Clone, Copy)]
pub struct ExtrudeConnector {
    pub vector: Vector3,
}

impl<C, S> Connector<C, S> for ExtrudeConnector
where
    C: Clone,
    ExtrudedCurve<C, Vector3>: ToSameGeometry<S>,
{
    fn connector(self) -> impl Fn(&C, &C) -> S {
        move |curve0, _| ExtrudedCurve::by_extrusion(curve0.clone(), self.vector).to_same_geometry()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ArcConnector {
    pub origin: Point3,
    pub axis: Vector3,
    pub angle: Rad<f64>,
}

impl<C> Connector<Point3, C> for ArcConnector
where Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>: ToSameGeometry<C>
{
    fn connector(self) -> impl Fn(&Point3, &Point3) -> C {
        let Self {
            origin,
            axis,
            angle,
        } = self;
        move |p, _| circle_arc(*p, origin, axis, angle).to_same_geometry()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RevoluteConnector {
    pub origin: Point3,
    pub axis: Vector3,
}

impl<C, S> Connector<C, S> for RevoluteConnector
where
    C: Clone,
    RevolutedCurve<C>: ToSameGeometry<S>,
{
    fn connector(self) -> impl Fn(&C, &C) -> S {
        let Self { origin, axis } = self;
        move |curve, _| {
            RevolutedCurve::by_revolution(curve.clone(), origin, axis).to_same_geometry()
        }
    }
}
