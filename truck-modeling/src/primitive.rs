use crate::builder;
use std::f64::consts::PI;
use truck_geometry::prelude::*;
use truck_topology::*;

/// rectangle
/// # Example
/// ```
/// use truck_modeling::*;
///
/// let r#box = BoundingBox::from_iter([
///     Point2::new(-1.0, -2.0),
///     Point2::new(2.0, 1.0),
/// ]);
/// let plane = Plane::zx();
/// let rect: Wire = primitive::rect(r#box, plane);
///
/// assert_eq!(rect[0].front().point(), Point3::new(-2.0, 0.0, -1.0));
/// assert_eq!(rect[1].front().point(), Point3::new(-2.0, 0.0, 2.0));
/// assert_eq!(rect[2].front().point(), Point3::new(1.0, 0.0, 2.0));
/// assert_eq!(rect[3].front().point(), Point3::new(1.0, 0.0, -1.0));
/// ```
/// # Remarks
/// Since it is a rectangle in a coordinate system on plane,
/// if the coordinate system is tilted, a parallelogram is drawn.
/// ```
/// use truck_modeling::*;
///
/// let r#box = BoundingBox::from_iter([
///     Point2::new(-1.0, -2.0),
///     Point2::new(2.0, 1.0),
/// ]);
/// let plane = Plane::new(
///     Point3::origin(),
///     Point3::new(1.0, 0.0, 0.0),
///     Point3::new(1.0, 1.0, 0.0),
/// );
/// let rect: Wire = primitive::rect(r#box, plane);
///
/// assert_eq!(rect[0].front().point(), Point3::new(-3.0, -2.0, 0.0));
/// assert_eq!(rect[1].front().point(), Point3::new(0.0, -2.0, 0.0));
/// assert_eq!(rect[2].front().point(), Point3::new(3.0, 1.0, 0.0));
/// assert_eq!(rect[3].front().point(), Point3::new(0.0, 1.0, 0.0));
/// ```
pub fn rect<C>(r#box: BoundingBox<Point2>, plane: Plane) -> Wire<Point3, C>
where Line<Point3>: ToSameGeometry<C> {
    let (min, max) = (r#box.min(), r#box.max());
    let v = builder::vertices([
        plane.subs(min.x, min.y),
        plane.subs(max.x, min.y),
        plane.subs(max.x, max.y),
        plane.subs(min.x, max.y),
    ]);
    wire![
        builder::line(&v[0], &v[1]),
        builder::line(&v[1], &v[2]),
        builder::line(&v[2], &v[3]),
        builder::line(&v[3], &v[0]),
    ]
}

/// circle, specified by the start point and the rotation axis.
/// # Example
/// ```
/// use truck_modeling::*;
///
/// let origin = Point3::new(1.0, -2.0, 3.0);
/// let axis = Vector3::new(0.0, 1.0, 0.0);
/// let start = origin + Vector3::new(3.0, 0.0, 4.0);
///
/// let wire: Wire = primitive::circle(start, origin, axis, 2);
///
/// assert_eq!(wire.len(), 2);
/// for edge in wire {
///     let arc = edge.oriented_curve();
///     let (t0, t1) = arc.range_tuple();
///     for i in 0..=10 {
///         let u = i as f64 / 10.0;
///         let t = (1.0 - u) * t0 + u * t1;
///         let p = arc.subs(t);
///         let der = arc.der(t);
///         assert_near!(p.distance(origin), 5.0);
///         assert!(der.dot(axis).so_small());
///         assert!((p - origin).cross(der).dot(axis) > 0.0);
///     }
/// }
/// ```
pub fn circle<C>(start: Point3, origin: Point3, axis: Vector3, division: usize) -> Wire<Point3, C>
where Processor<TrimmedCurve<UnitCircle<Point3>>, Matrix4>: ToSameGeometry<C> {
    let origin = origin + (start - origin).dot(axis) * axis;
    let radius = start - origin;
    let y = axis.cross(radius);
    let mat = Matrix4::from_cols(
        radius.extend(0.0),
        y.extend(0.0),
        axis.extend(0.0),
        origin.to_homogeneous(),
    );

    let make_vertices = move |i: usize| {
        let t = 2.0 * PI * i as f64 / division as f64;
        let p = Point3::new(f64::cos(t), f64::sin(t), 0.0);
        Vertex::new(mat.transform_point(p))
    };
    let v = (0..division).map(make_vertices).collect::<Vec<_>>();

    let make_edges = move |i: usize| {
        let t0 = 2.0 * PI * i as f64 / division as f64;
        let t1 = 2.0 * PI * (i + 1) as f64 / division as f64;
        let unit_circle = UnitCircle::new();
        let trimmed = TrimmedCurve::new(unit_circle, (t0, t1));
        let mut arc = Processor::new(trimmed);
        arc.transform_by(mat);
        Edge::new(&v[i], &v[(i + 1) % division], arc.to_same_geometry())
    };
    (0..division).map(make_edges).collect()
}

/// cuboid, defined by bounding box
/// # Example
/// ```
/// use truck_modeling::*;
/// let p = Point3::new(-1.0, 2.0, -3.0);
/// let q = Point3::new(10.0, -5.0, 4.0);
///
/// let bbd = BoundingBox::from_iter([p, q]);
/// let solid: Solid = primitive::cuboid(bbd);
///
/// for v in solid.vertex_iter() {
///     let x = v.point();
///     assert!(x.x.near(&p.x) || x.x.near(&q.x));
///     assert!(x.y.near(&p.y) || x.y.near(&q.y));
///     assert!(x.z.near(&p.z) || x.z.near(&q.z));
/// }
/// ```
pub fn cuboid<C, S>(r#box: BoundingBox<Point3>) -> Solid<Point3, C, S>
where
    Line<Point3>: ToSameGeometry<C>,
    Plane: ToSameGeometry<S>, {
    let (p, q) = (r#box.min(), r#box.max());
    let v = builder::vertices([
        (p.x, p.y, p.z),
        (q.x, p.y, p.z),
        (q.x, q.y, p.z),
        (p.x, q.y, p.z),
        (p.x, p.y, q.z),
        (q.x, p.y, q.z),
        (q.x, q.y, q.z),
        (p.x, q.y, q.z),
    ]);
    let e = [
        builder::line(&v[0], &v[1]),
        builder::line(&v[1], &v[2]),
        builder::line(&v[2], &v[3]),
        builder::line(&v[3], &v[0]),
        builder::line(&v[0], &v[4]),
        builder::line(&v[1], &v[5]),
        builder::line(&v[2], &v[6]),
        builder::line(&v[3], &v[7]),
        builder::line(&v[4], &v[5]),
        builder::line(&v[5], &v[6]),
        builder::line(&v[6], &v[7]),
        builder::line(&v[7], &v[4]),
    ];

    let wire0 = wire![
        e[3].inverse(),
        e[2].inverse(),
        e[1].inverse(),
        e[0].inverse(),
    ];
    let plane0 = Plane::new(v[0].point(), v[3].point(), v[1].point());
    let mut shell = shell![Face::new(vec![wire0], plane0.to_same_geometry())];

    (0..4).for_each(|i| {
        let wirei = wire![
            e[i].clone(),
            e[(i + 1) % 4 + 4].clone(),
            e[i + 8].inverse(),
            e[i + 4].inverse(),
        ];
        let planei = Plane::new(v[i].point(), v[i + 1].point(), v[i + 4].point());
        shell.push(Face::new(vec![wirei], planei.to_same_geometry()));
    });

    let wire5 = wire![e[8].clone(), e[9].clone(), e[10].clone(), e[11].clone(),];
    let plane5 = Plane::new(v[4].point(), v[5].point(), v[7].point());
    shell.push(Face::new(vec![wire5], plane5.to_same_geometry()));

    Solid::new(vec![shell])
}
