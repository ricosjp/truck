use crate::{errors::Error, geom_impl::FilletCandidate, *};

type Edge<C> = truck_topology::Edge<Point2, C>;

/// Trait alias for 2-dimensional curves that can be trimmed for corner operations.
pub trait TrimmableCurve2D: ParametricCurve2D + BoundedCurve + Cut + Invertible {}

impl<C: ParametricCurve2D + BoundedCurve + Cut + Invertible> TrimmableCurve2D for C {}

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
    } = geom_impl::fillet_candidate(&curve0, &curve1, t0, t1, radius)?;

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

    let parameter0 = geom_impl::parameter_at_curve_length(&curve0, t0, -distance0)?;
    let parameter1 = geom_impl::parameter_at_curve_length(&curve1, t1, distance1)?;
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
