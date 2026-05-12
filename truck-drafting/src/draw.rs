use crate::{errors::Error, *};
use itertools::Itertools;

type Vertex = truck_topology::Vertex<Point2>;
type Edge<C> = truck_topology::Edge<Point2, C>;
type Wire<C> = truck_topology::Wire<Point2, C>;

type CircleArc = Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3>;

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

/// Tries to returns a line from `vertex0` to `vertex1`.
/// # Examples
/// ```
/// use truck_drafting::*;
///
/// let vertex0 = draw::vertex((0.0, 0.0));
/// let vertex1 = draw::vertex((2.0, 1.0));
/// let line: Edge = draw::try_line(&vertex0, &vertex1).unwrap();
/// # let curve = line.oriented_curve();
/// # assert_near!(curve.subs(0.0), Point2::new(0.0, 0.0));
/// # assert_near!(curve.subs(0.5), Point2::new(1.0, 0.5));
/// # assert_near!(curve.subs(1.0), Point2::new(2.0, 1.0));
/// ```
pub fn try_line<C>(vertex0: &Vertex, vertex1: &Vertex) -> Result<Edge<C>, Error>
where Line<Point2>: ToSameGeometry<C> {
    let point0 = vertex0.point();
    let point1 = vertex1.point();
    Ok(Edge::try_new(
        vertex0,
        vertex1,
        Line(point0, point1).to_same_geometry(),
    )?)
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
    try_line(vertex0, vertex1).unwrap()
}

/// Tries to returns a polyline through the given points.
/// # Examples
/// ```
/// use truck_drafting::*;
///
/// let vertices = draw::vertices([(0.0, 0.0), (1.0, 0.0), (1.0, 2.0)]);
/// let polyline: Wire = draw::try_polyline(&vertices).unwrap();
/// # assert_eq!(polyline.len(), 2);
/// # assert!(polyline.is_continuous());
/// # let vertices: Vec<_> = polyline.vertex_iter().map(|vertex| vertex.point()).collect();
/// # assert_eq!(
/// #     vertices,
/// #     vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0), Point2::new(1.0, 2.0)],
/// # );
/// ```
pub fn try_polyline<'a, C>(
    vertices: impl IntoIterator<Item = &'a Vertex>,
) -> Result<Wire<C>, Error>
where Line<Point2>: ToSameGeometry<C> {
    vertices
        .into_iter()
        .tuple_windows()
        .map(|(vertex0, vertex1)| try_line(vertex0, vertex1))
        .collect()
}

/// Returns a polyline through the given points.
/// # Examples
/// ```
/// use truck_drafting::*;
///
/// let vertices = draw::vertices([(0.0, 0.0), (1.0, 0.0), (1.0, 2.0)]);
/// let polyline: Wire = draw::polyline(&vertices);
/// # assert_eq!(polyline.len(), 2);
/// # assert!(polyline.is_continuous());
/// # let vertices: Vec<_> = polyline.vertex_iter().map(|vertex| vertex.point()).collect();
/// # assert_eq!(
/// #     vertices,
/// #     vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0), Point2::new(1.0, 2.0)],
/// # );
/// ```
pub fn polyline<'a, C>(vertices: impl IntoIterator<Item = &'a Vertex>) -> Wire<C>
where Line<Point2>: ToSameGeometry<C> {
    try_polyline(vertices).unwrap()
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
) -> Result<Edge<C>, Error>
where
    CircleArc: ToSameGeometry<C>,
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
    Ok(Edge::try_new(vertex0, vertex1, curve.to_same_geometry())?)
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
    CircleArc: ToSameGeometry<C>,
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

/// connect two vertices by two lines
pub fn try_line_line<C>(
    vertex0: &Vertex,
    vertex1: &Vertex,
    direction0: Vector2,
    direction1: Vector2,
) -> Result<Wire<C>, Error>
where
    Line<Point2>: ToSameGeometry<C>,
{
    let (point0, point1) = (vertex0.point(), vertex1.point());
    let transit = &vertex(geom_impls::lines_crossing_point(
        point0, point1, direction0, direction1,
    )?);
    
    Ok(wire![
        try_line(vertex0, transit)?,
        try_line(transit, vertex1)?
    ])
}

/// connect two vertices by two lines.
pub fn line_line<C>(
    vertex0: &Vertex,
    vertex1: &Vertex,
    direction0: Vector2,
    direction1: Vector2,
) -> Wire<C>
where
    Line<Point2>: ToSameGeometry<C>,
{
    try_line_line(vertex0, vertex1, direction0, direction1).unwrap()
}

/// Tries to connect vertices by two circle arcs.
pub fn try_arc_arc<C>(
    vertex0: &Vertex,
    vertex1: &Vertex,
    tangent0: Vector2,
    radius0: f64,
    tangent1: Vector2,
) -> Result<Wire<C>, Error>
where
    CircleArc: ToSameGeometry<C>,
{
    let (point0, point1) = (vertex0.point(), vertex1.point());
    let transit = &vertex(geom_impls::arc_arc_transit(
        point0, point1, tangent0, radius0, tangent1,
    )?);
    let edge0 = circle_arc(vertex0, transit, tangent0);
    let mut edge1 = circle_arc(vertex1, transit, -tangent1);
    edge1.invert();
    Ok(wire![edge0, edge1])
}

/// connect vertices by two circle arcs.
pub fn arc_arc<C>(
    vertex0: &Vertex,
    vertex1: &Vertex,
    tangent0: Vector2,
    radius0: f64,
    tangent1: Vector2,
) -> Wire<C>
where
    CircleArc: ToSameGeometry<C>,
{
    try_arc_arc(vertex0, vertex1, tangent0, radius0, tangent1).unwrap()
}

/// 2つの頂点を2直線と間の円弧で滑らかに接続する
pub fn try_line_arc_line<C>(
    vertex0: &Vertex,
    vertex1: &Vertex,
    tangent0: Vector2,
    radius: f64,
    tangent1: Vector2,
) -> Result<Wire<C>, Error>
where
    Line<Point2>: ToSameGeometry<C>,
    CircleArc: ToSameGeometry<C>,
{
    let (point0, point1) = (vertex0.point(), vertex1.point());
    let (transit_point0, transit_point1) =
        geom_impls::line_arc_line_transit(point0, point1, tangent0, radius, tangent1)?;
    let (transit0, transit1) = (&vertex(transit_point0), &vertex(transit_point1));
    Ok(wire![
        try_line(vertex0, transit0)?,
        try_circle_arc(transit0, transit1, transit_point0 - point0)?,
        try_line(transit1, vertex1)?,
    ])
}

/// 2つの頂点を2直線と間の円弧で滑らかに接続する
pub fn line_arc_line<C>(
    vertex0: &Vertex,
    vertex1: &Vertex,
    tangent0: Vector2,
    radius: f64,
    tangent1: Vector2,
) -> Wire<C>
where
    Line<Point2>: ToSameGeometry<C>,
    CircleArc: ToSameGeometry<C>,
{
    try_line_arc_line(vertex0, vertex1, tangent0, radius, tangent1).unwrap()
}

/// 2つの頂点を2円と間の接線で滑らかに接続する
pub fn try_arc_line_arc<C>(
    vertex0: &Vertex,
    vertex1: &Vertex,
    tangent0: Vector2,
    tangent1: Vector2,
    radius0: f64,
    radius1: f64,
) -> Result<Wire<C>, Error>
where
    Line<Point2>: ToSameGeometry<C>,
    CircleArc: ToSameGeometry<C>,
{
    let (point0, point1) = (vertex0.point(), vertex1.point());
    let (transit_point0, transit_point1) =
        geom_impls::arc_line_arc_transit(point0, point1, tangent0, tangent1, radius0, radius1)?;
    let (transit0, transit1) = (&vertex(transit_point0), &vertex(transit_point1));
    Ok(wire![
        try_circle_arc(vertex0, transit0, tangent0)?,
        try_line(transit0, transit1)?,
        try_circle_arc(transit1, vertex1, tangent1)?,
    ])
}

/// 2つの頂点を2円と間の接線で滑らかに接続する
pub fn arc_line_arc<C>(
    vertex0: &Vertex,
    vertex1: &Vertex,
    tangent0: Vector2,
    tangent1: Vector2,
    radius0: f64,
    radius1: f64,
) -> Wire<C>
where
    Line<Point2>: ToSameGeometry<C>,
    CircleArc: ToSameGeometry<C>,
{
    try_arc_line_arc(vertex0, vertex1, tangent0, tangent1, radius0, radius1).unwrap()
}
