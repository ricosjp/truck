use crate::{errors::Error, geom_impls::FilletCandidate, *};

type Vertex = truck_topology::Vertex<Point2>;
type Edge<C> = truck_topology::Edge<Point2, C>;
type Wire<C> = truck_topology::Wire<Point2, C>;

/// Trait alias for 2-dimensional curves that can be trimmed for corner operations.
pub trait TrimmableCurve2D:
    ParametricCurve2D + BoundedCurve + Cut + Invertible + SearchNearestParameter<D1, Point = Point2>
{
}

impl<
    C: ParametricCurve2D
        + BoundedCurve
        + Cut
        + Invertible
        + SearchNearestParameter<D1, Point = Point2>,
> TrimmableCurve2D for C
{
}

/// Result of a corner operation represented by the trimmed incoming edge,
/// the inserted connector, and the trimmed outgoing edge.
#[derive(Clone, Debug)]
pub struct CornerResult<C> {
    /// incoming edge after trimming
    pub edge0: Edge<C>,
    /// connector edge such as a fillet arc or a chamfer segment
    pub connector: Edge<C>,
    /// outgoing edge after trimming
    pub edge1: Edge<C>,
}

/// Creates a fillet between two consecutive edges.
/// # Examples
/// ```
/// use truck_drafting::{corner::CornerResult, *};
///
/// let v0 = draw::vertex((0.0, 0.0));
/// let v1 = draw::vertex((1.0, 0.0));
/// let v2 = draw::vertex((1.0, 1.0));
/// let edge0: Edge = draw::line(&v0, &v1);
/// let edge1: Edge = draw::line(&v1, &v2);
/// let CornerResult {
///     edge0,
///     connector,
///     edge1,
/// } = corner::fillet(&edge0, &edge1, 0.2).unwrap();
///
/// assert_near!(edge0.front().point(), Point2::new(0.0, 0.0));
/// assert_near!(edge0.back().point(), Point2::new(0.8, 0.0));
/// assert_near!(edge1.front().point(), Point2::new(1.0, 0.2));
/// assert_near!(edge1.back().point(), Point2::new(1.0, 1.0));
///
/// assert_near!(connector.front().point(), Point2::new(0.8, 0.0));
/// assert_near!(connector.back().point(), Point2::new(1.0, 0.2));
///
/// let curve = connector.oriented_curve();
/// let (t0, t1) = curve.range_tuple();
/// assert_near!(curve.der(t0).normalize(), Vector2::new(1.0, 0.0));
/// assert_near!(curve.der(t1).normalize(), Vector2::new(0.0, 1.0));
/// ```
pub fn fillet<C>(edge0: &Edge<C>, edge1: &Edge<C>, radius: f64) -> Result<CornerResult<C>, Error>
where
    C: TrimmableCurve2D,
    Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3>: ToSameGeometry<C>, {
    let mut curve0 = edge0.oriented_curve();
    let (_, t0) = curve0.range_tuple();
    let mut curve1 = edge1.oriented_curve();
    let (t1, _) = curve1.range_tuple();

    let FilletCandidate {
        parameter0,
        parameter1,
        ..
    } = geom_impls::fillet_candidate(&curve0, &curve1, t0, t1, radius)?;

    let point0 = curve0.subs(parameter0);
    let point1 = curve1.subs(parameter1);
    let tangent0 = curve0.der(parameter0);

    curve0.cut(parameter0);
    curve1 = curve1.cut(parameter1);

    let v0 = edge0.front();
    let v1 = draw::vertex(point0);
    let v2 = draw::vertex(point1);
    let v3 = edge1.back();

    let connector = draw::circle_arc(&v1, &v2, tangent0);
    let edge0 = Edge::new(v0, &v1, curve0);
    let edge1 = Edge::new(&v2, v3, curve1);

    Ok(CornerResult {
        edge0,
        edge1,
        connector,
    })
}

/// Creates a chamfer between two consecutive edges.
/// # Examples
/// ```
/// use truck_drafting::{corner::CornerResult, *};
///
/// let v0 = draw::vertex((0.0, 0.0));
/// let v1 = draw::vertex((1.0, 0.0));
/// let v2 = draw::vertex((1.0, 1.0));
/// let edge0: Edge = draw::line(&v0, &v1);
/// let edge1: Edge = draw::line(&v1, &v2);
/// let CornerResult {
///     edge0,
///     connector,
///     edge1,
/// } = corner::chamfer(&edge0, &edge1, 0.2, 0.3).unwrap();
///
/// assert_near!(edge0.front().point(), Point2::new(0.0, 0.0));
/// assert_near!(edge0.back().point(), Point2::new(0.8, 0.0));
/// assert_near!(edge1.front().point(), Point2::new(1.0, 0.3));
/// assert_near!(edge1.back().point(), Point2::new(1.0, 1.0));
///
/// assert_near!(connector.front().point(), Point2::new(0.8, 0.0));
/// assert_near!(connector.back().point(), Point2::new(1.0, 0.3));
///
/// let curve = connector.oriented_curve();
/// assert_near!(curve.der(0.0).normalize(), Vector2::new(2.0, 3.0).normalize());
/// ```
pub fn chamfer<C>(
    edge0: &Edge<C>,
    edge1: &Edge<C>,
    distance0: f64,
    distance1: f64,
) -> Result<CornerResult<C>, Error>
where
    C: TrimmableCurve2D,
    Line<Point2>: ToSameGeometry<C>,
{
    if distance0 <= 0.0 || distance1 <= 0.0 {
        return Err(Error::NonPositiveChamferDistance);
    }

    let mut curve0 = edge0.oriented_curve();
    let (_, t0) = curve0.range_tuple();
    let mut curve1 = edge1.oriented_curve();
    let (t1, _) = curve1.range_tuple();

    let parameter0 = geom_impls::parameter_at_curve_length(&curve0, t0, -distance0)?;
    let parameter1 = geom_impls::parameter_at_curve_length(&curve1, t1, distance1)?;
    let point0 = curve0.subs(parameter0);
    let point1 = curve1.subs(parameter1);

    curve0.cut(parameter0);
    curve1 = curve1.cut(parameter1);

    let v0 = edge0.front();
    let v1 = draw::vertex(point0);
    let v2 = draw::vertex(point1);
    let v3 = edge1.back();

    let connector = draw::line(&v1, &v2);
    let edge0 = Edge::new(v0, &v1, curve0);
    let edge1 = Edge::new(&v2, v3, curve1);

    Ok(CornerResult {
        edge0,
        connector,
        edge1,
    })
}

fn corner_ops_all<C>(
    wire: &Wire<C>,
    mut ops: impl FnMut(&Edge<C>, &Edge<C>) -> Result<CornerResult<C>, Error>,
) -> Result<Wire<C>, Error>
where
    C: TrimmableCurve2D,
{
    if !wire.is_continuous() {
        return Err(Error::NonContinuousWire);
    }
    let len = wire.len();
    if len < 2 {
        return Ok(wire.clone());
    }

    let cyclic = wire.is_cyclic();
    let corner_count = match cyclic {
        true => len,
        false => len - 1,
    };
    let edges = wire.edge_iter().collect::<Vec<_>>();
    let corners = (0..corner_count)
        .map(|idx| ops(edges[idx], edges[(idx + 1) % len]))
        .collect::<Result<Vec<_>, _>>()?;

    let mut new_edges = Vec::with_capacity(len + corner_count);
    if cyclic {
        for idx in 0..len {
            let prev = (idx + corner_count - 1) % corner_count;
            new_edges.push(trimmed_middle_edge(
                edges[idx],
                &corners[prev],
                &corners[idx],
            )?);
            new_edges.push(corners[idx].connector.clone());
        }
    } else {
        new_edges.push(corners[0].edge0.clone());
        new_edges.push(corners[0].connector.clone());
        for idx in 1..corner_count {
            new_edges.push(trimmed_middle_edge(
                edges[idx],
                &corners[idx - 1],
                &corners[idx],
            )?);
            new_edges.push(corners[idx].connector.clone());
        }
        new_edges.push(corners[corner_count - 1].edge1.clone());
    }
    Ok(new_edges.into())
}

fn trimmed_middle_edge<C>(
    edge: &Edge<C>,
    prev: &CornerResult<C>,
    next: &CornerResult<C>,
) -> Result<Edge<C>, Error>
where
    C: TrimmableCurve2D,
{
    let (start, _) = prev.edge1.curve().range_tuple();
    let (_, end) = next.edge0.curve().range_tuple();
    if start > end + TOLERANCE {
        return Err(Error::CurveLengthOutOfRange);
    }

    let mut curve = edge.oriented_curve();
    let (front, back) = curve.range_tuple();
    if end < back - TOLERANCE {
        curve.cut(end);
    }
    if start > front + TOLERANCE {
        curve = curve.cut(start);
    }

    Ok(Edge::new(prev.edge1.front(), next.edge0.back(), curve))
}

/// Creates fillets at all corners in a continuous wire.
/// # Examples
/// open wire case
/// ```
/// use truck_drafting::*;
///
/// let wire: Wire = draw::polyline([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]);
/// let filleted = corner::fillet_all(&wire, |_| 0.2).unwrap();
/// assert_eq!(filleted.len(), 3);
/// # assert_near!(filleted[0].back().point(), Point2::new(0.8, 0.0));
/// # assert_near!(filleted[1].front().point(), Point2::new(0.8, 0.0));
/// # assert_near!(filleted[1].back().point(), Point2::new(1.0, 0.2));
/// # assert_near!(filleted[2].front().point(), Point2::new(1.0, 0.2));
/// ```
/// closed wire case
/// ```
/// use truck_drafting::*;
///
/// let v0 = draw::vertex((0.0, 0.0));
/// let v1 = draw::vertex((1.0, 0.0));
/// let v2 = draw::vertex((1.0, 1.0));
/// let wire: Wire = wire![
///     draw::line(&v0, &v1),
///     draw::line(&v1, &v2),
///     draw::line(&v2, &v0),
/// ];
/// let radius = 0.1;
/// let filleted = corner::fillet_all(&wire, |_| radius).unwrap();
/// # let tangent_length_at_v0 = radius * (f64::sqrt(2.0) + 1.0);
/// assert_eq!(filleted.len(), 6);
/// assert!(filleted.is_closed());
/// # assert_near!(filleted[0].front().point(), Point2::new(tangent_length_at_v0, 0.0));
/// # assert_near!(filleted[0].back().point(), Point2::new(0.9, 0.0));
/// # assert_near!(filleted[1].front().point(), Point2::new(0.9, 0.0));
/// # assert_near!(filleted[1].back().point(), Point2::new(1.0, 0.1));
/// # assert_near!(filleted[5].back().point(), Point2::new(tangent_length_at_v0, 0.0));
/// ```
pub fn fillet_all<C>(
    wire: &Wire<C>,
    mut radius: impl FnMut(&Vertex) -> f64,
) -> Result<Wire<C>, Error>
where
    C: TrimmableCurve2D,
    Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3>: ToSameGeometry<C>,
{
    corner_ops_all(wire, |edge0, edge1| {
        fillet(edge0, edge1, radius(edge0.back()))
    })
}

/// Creates chamfers at all corners in a continuous wire.
/// # Examples
/// open wire case
/// ```
/// use truck_drafting::*;
///
/// let wire: Wire = draw::polyline([(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]);
/// let chamfered = corner::chamfer_all(&wire, |_| (0.2, 0.3)).unwrap();
/// assert_eq!(chamfered.len(), 3);
/// # assert_near!(chamfered[0].back().point(), Point2::new(0.8, 0.0));
/// # assert_near!(chamfered[1].front().point(), Point2::new(0.8, 0.0));
/// # assert_near!(chamfered[1].back().point(), Point2::new(1.0, 0.3));
/// # assert_near!(chamfered[2].front().point(), Point2::new(1.0, 0.3));
/// ```
/// closed wire case
/// ```
/// use truck_drafting::*;
///
/// let v0 = draw::vertex((0.0, 0.0));
/// let v1 = draw::vertex((1.0, 0.0));
/// let v2 = draw::vertex((1.0, 1.0));
/// let wire: Wire = wire![
///     draw::line(&v0, &v1),
///     draw::line(&v1, &v2),
///     draw::line(&v2, &v0),
/// ];
/// let chamfered = corner::chamfer_all(&wire, |_| (0.1, 0.1)).unwrap();
/// assert_eq!(chamfered.len(), 6);
/// assert!(chamfered.is_closed());
/// # assert_near!(chamfered[0].front().point(), Point2::new(0.1, 0.0));
/// # assert_near!(chamfered[0].back().point(), Point2::new(0.9, 0.0));
/// # assert_near!(chamfered[1].front().point(), Point2::new(0.9, 0.0));
/// # assert_near!(chamfered[1].back().point(), Point2::new(1.0, 0.1));
/// # assert_near!(chamfered[5].back().point(), Point2::new(0.1, 0.0));
/// ```
pub fn chamfer_all<C>(
    wire: &Wire<C>,
    mut distance: impl FnMut(&Vertex) -> (f64, f64),
) -> Result<Wire<C>, Error>
where
    C: TrimmableCurve2D,
    Line<Point2>: ToSameGeometry<C>,
{
    corner_ops_all(wire, |edge0, edge1| {
        let (distance0, distance1) = distance(edge0.back());
        chamfer(edge0, edge1, distance0, distance1)
    })
}
