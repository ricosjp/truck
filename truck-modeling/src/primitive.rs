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
/// let rect: Face = primitive::rect(r#box, plane);
///
/// let wire = &rect.boundaries()[0];
/// assert_eq!(wire[0].front().point(), Point3::new(-2.0, 0.0, -1.0));
/// assert_eq!(wire[1].front().point(), Point3::new(-2.0, 0.0, 2.0));
/// assert_eq!(wire[2].front().point(), Point3::new(1.0, 0.0, 2.0));
/// assert_eq!(wire[3].front().point(), Point3::new(1.0, 0.0, -1.0));
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
/// let rect: Face = primitive::rect(r#box, plane);
///
/// let wire = &rect.boundaries()[0];
/// assert_eq!(wire[0].front().point(), Point3::new(-3.0, -2.0, 0.0));
/// assert_eq!(wire[1].front().point(), Point3::new(0.0, -2.0, 0.0));
/// assert_eq!(wire[2].front().point(), Point3::new(3.0, 1.0, 0.0));
/// assert_eq!(wire[3].front().point(), Point3::new(0.0, 1.0, 0.0));
/// ```
pub fn rect<C, S>(r#box: BoundingBox<Point2>, plane: Plane) -> Face<Point3, C, S>
where
    Line<Point3>: ToSameGeometry<C>,
    Plane: ToSameGeometry<S>, {
    let (min, max) = (r#box.min(), r#box.max());
    let p = [
        plane.subs(min.x, min.y),
        plane.subs(max.x, min.y),
        plane.subs(max.x, max.y),
        plane.subs(min.x, max.y),
    ];
    let v = Vertex::news(p);
    let wire = wire![
        Edge::new(&v[0], &v[1], Line(p[0], p[1]).to_same_geometry()),
        Edge::new(&v[1], &v[2], Line(p[1], p[2]).to_same_geometry()),
        Edge::new(&v[2], &v[3], Line(p[2], p[3]).to_same_geometry()),
        Edge::new(&v[3], &v[0], Line(p[3], p[0]).to_same_geometry()),
    ];
    Face::new(vec![wire], plane.to_same_geometry())
}
