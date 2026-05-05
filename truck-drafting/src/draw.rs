use crate::*;

type Vertex = truck_topology::Vertex<Point2>;
type Edge<C> = truck_topology::Edge<Point2, C>;
type Wire<C> = truck_topology::Wire<Point2, C>;

/// Additional constraint to determine a circular arc from its start and end points.
#[derive(Clone, Copy, Debug, derive_more::From)]
pub enum ArcConstraint {
    /// A point that the arc must pass through.
    Transit(Point2),
    /// A tangent vector that the arc must have at the start point.
    Tangent(Vector2),
}

/// Creates and returns a vertex by a two dimensional point.
/// # Examples
/// ```
/// use truck_drafting::*;
///
/// let vertex = draw::vertex((1.0, 2.0));
/// # assert_eq!(vertex.point(), Point2::new(1.0, 2.0));
/// ```
#[inline(always)]
pub fn vertex<P: Into<Point2>>(point: P) -> Vertex { Vertex::new(point.into()) }

/// Creates and returns vertices by two dimensional points.
/// # Examples
/// ```
/// use truck_drafting::*;
///
/// let vertices = draw::vertices([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]);
/// # assert_eq!(vertices[0].point(), Point2::new(0.0, 0.0));
/// # assert_eq!(vertices[1].point(), Point2::new(1.0, 0.0));
/// # assert_eq!(vertices[2].point(), Point2::new(1.0, 1.0));
/// ```
#[inline(always)]
pub fn vertices<P: Into<Point2>>(points: impl IntoIterator<Item = P>) -> Vec<Vertex> {
    points
        .into_iter()
        .map(|point| Vertex::new(point.into()))
        .collect()
}

/// Returns a line from `vertex0` to `vertex1`.
/// # Examples
/// ```
/// use truck_drafting::*;
///
/// let vertex0 = draw::vertex((0.0, 0.0));
/// let vertex1 = draw::vertex((2.0, 1.0));
/// let line: Edge = draw::line(&vertex0, &vertex1);
/// # let curve = line.oriented_curve();
/// # assert_near!(curve.subs(0.0), Point2::new(0.0, 0.0));
/// # assert_near!(curve.subs(0.5), Point2::new(1.0, 0.5));
/// # assert_near!(curve.subs(1.0), Point2::new(2.0, 1.0));
/// ```
pub fn line<C>(vertex0: &Vertex, vertex1: &Vertex) -> Edge<C>
where Line<Point2>: ToSameGeometry<C> {
    let point0 = vertex0.point();
    let point1 = vertex1.point();
    Edge::new(vertex0, vertex1, Line(point0, point1).to_same_geometry())
}

/// Returns a polyline through the given points.
/// # Examples
/// ```
/// use truck_drafting::*;
///
/// let polyline: Wire = draw::polyline([(0.0, 0.0), (1.0, 0.0), (1.0, 2.0)]);
/// # assert_eq!(polyline.len(), 2);
/// # assert!(polyline.is_continuous());
/// # let vertices: Vec<_> = polyline.vertex_iter().map(|vertex| vertex.point()).collect();
/// # assert_eq!(
/// #     vertices,
/// #     vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0), Point2::new(1.0, 2.0)],
/// # );
/// ```
pub fn polyline<C, P>(points: impl IntoIterator<Item = P>) -> Wire<C>
where
    P: Into<Point2>,
    Line<Point2>: ToSameGeometry<C>, {
    let vertices = vertices(points);
    vertices
        .windows(2)
        .map(|vertices| line(&vertices[0], &vertices[1]))
        .collect()
}

/// Tries to return a circle arc from `vertex0` to `vertex1` with `constraint`.
/// # Examples
/// ```
/// use truck_drafting::*;
///
/// let vertex0 = draw::vertex((1.0, 0.0));
/// let vertex1 = draw::vertex((-1.0, 0.0));
/// let transit = Point2::new(0.0, 1.0);
/// let arc: Edge = draw::try_circle_arc(&vertex0, &vertex1, transit).unwrap();
/// # let curve = arc.oriented_curve();
/// # let (t0, t1) = curve.range_tuple();
/// # assert_near!(curve.subs(t0), Point2::new(1.0, 0.0));
/// # assert_near!(curve.subs((t0 + t1) * 0.5), Point2::new(0.0, 1.0));
/// # assert_near!(curve.subs(t1), Point2::new(-1.0, 0.0));
/// ```
/// ```
/// use truck_drafting::*;
///
/// let vertex0 = draw::vertex((1.0, 0.0));
/// let vertex1 = draw::vertex((0.0, 1.0));
/// let tangent = Vector2::new(0.0, 1.0);
/// let arc: Edge = draw::try_circle_arc(&vertex0, &vertex1, tangent).unwrap();
/// # let curve = arc.oriented_curve();
/// # let (t0, t1) = curve.range_tuple();
/// # assert_near!(curve.subs(t0), Point2::new(1.0, 0.0));
/// # assert_near!(curve.subs(t1), Point2::new(0.0, 1.0));
/// # assert_near!(curve.der(t0).normalize(), Vector2::new(0.0, 1.0));
/// ```
pub fn try_circle_arc<C>(
    vertex0: &Vertex,
    vertex1: &Vertex,
    constraint: impl Into<ArcConstraint>,
) -> Result<Edge<C>, errors::Error>
where
    Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3>: ToSameGeometry<C>,
{
    let point0 = vertex0.point();
    let point1 = vertex1.point();
    let curve = match constraint.into() {
        ArcConstraint::Transit(transit) => {
            geom_impls::circle_arc_by_three_points(point0, point1, transit)
        }
        ArcConstraint::Tangent(tangent) => {
            geom_impls::circle_arc_by_tangent0(point0, point1, tangent)
        }
    }?;
    Ok(Edge::new(vertex0, vertex1, curve.to_same_geometry()))
}

/// Returns a circle arc from `vertex0` to `vertex1` with `constraint`.
/// # Examples
/// ```
/// use truck_drafting::*;
///
/// let vertex0 = draw::vertex((1.0, 0.0));
/// let vertex1 = draw::vertex((-1.0, 0.0));
/// let transit = Point2::new(0.0, 1.0);
/// let arc: Edge = draw::circle_arc(&vertex0, &vertex1, transit);
/// # let curve = arc.oriented_curve();
/// # let (t0, t1) = curve.range_tuple();
/// # assert_near!(curve.subs(t0), Point2::new(1.0, 0.0));
/// # assert_near!(curve.subs((t0 + t1) * 0.5), Point2::new(0.0, 1.0));
/// # assert_near!(curve.subs(t1), Point2::new(-1.0, 0.0));
/// ```
/// ```
/// use truck_drafting::*;
///
/// let vertex0 = draw::vertex((1.0, 0.0));
/// let vertex1 = draw::vertex((0.0, 1.0));
/// let tangent = Vector2::new(0.0, 1.0);
/// let arc: Edge = draw::circle_arc(&vertex0, &vertex1, tangent);
/// # let curve = arc.oriented_curve();
/// # let (t0, t1) = curve.range_tuple();
/// # assert_near!(curve.subs(t0), Point2::new(1.0, 0.0));
/// # assert_near!(curve.subs(t1), Point2::new(0.0, 1.0));
/// # assert_near!(curve.der(t0).normalize(), Vector2::new(0.0, 1.0));
/// ```
pub fn circle_arc<C>(
    vertex0: &Vertex,
    vertex1: &Vertex,
    constraint: impl Into<ArcConstraint>,
) -> Edge<C>
where
    Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3>: ToSameGeometry<C>,
{
    try_circle_arc(vertex0, vertex1, constraint).unwrap()
}

/// Returns a Bezier curve from `vertex0` to `vertex1` with inter control points `inter_points`.
/// # Examples
/// ```
/// use truck_drafting::*;
///
/// let vertex0 = draw::vertex((0.0, 0.0));
/// let vertex1 = draw::vertex((3.0, 0.0));
/// let inter_points = vec![Point2::new(1.0, 1.0), Point2::new(2.0, -1.0)];
/// let bezier: Edge = draw::bezier(&vertex0, &vertex1, inter_points);
/// # let curve = bezier.oriented_curve();
/// # assert_near!(curve.subs(0.0), Point2::new(0.0, 0.0));
/// # assert_near!(curve.subs(1.0), Point2::new(3.0, 0.0));
/// ```
pub fn bezier<C>(vertex0: &Vertex, vertex1: &Vertex, mut inter_points: Vec<Point2>) -> Edge<C>
where BSplineCurve<Point2>: ToSameGeometry<C> {
    let point0 = vertex0.point();
    let point1 = vertex1.point();
    let mut control_points = vec![point0];
    control_points.append(&mut inter_points);
    control_points.push(point1);
    let knot_vec = KnotVec::bezier_knot(control_points.len() - 1);
    let curve = BSplineCurve::new(knot_vec, control_points);
    Edge::new(vertex0, vertex1, curve.to_same_geometry())
}
