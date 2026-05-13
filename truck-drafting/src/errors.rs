use thiserror::Error;

/// Drafting errors
#[derive(Debug, PartialEq, Error)]
pub enum Error {
    /// cannot construct a circle arc from collinear points.
    #[error("cannot construct a circle arc from collinear points.")]
    CollinearArcPoints,
    /// cannot construct a circle arc when the tangent is parallel to the chord.
    #[error("cannot construct a circle arc when the tangent is parallel to the chord.")]
    ParallelArcTangent,
    /// the radius of circle must be positive.
    #[error("the radius of circle must be positive.")]
    NonPositiveRadius,
    /// the tangent vector at the specified parameter vanished.
    #[error("the tangent vector vanished near the specified corner.")]
    DegenerateTangent,
    /// two line directions are parallel and cannot determine a crossing point.
    #[error("two line directions are parallel and cannot determine a crossing point.")]
    ParallelLineDirections,
    /// the connection corner is too close to one of the vertices.
    #[error("the connection corner is too close to one of the vertices.")]
    DegenerateConnectionCorner,
    /// the requested vertices and tangents cannot be connected by the selected primitive sequence.
    #[error(
        "the requested vertices and tangents cannot be connected by the selected primitive sequence."
    )]
    NoConnection,
    /// the specified corner is degenerate and cannot define a fillet direction.
    #[error("the specified corner is degenerate and cannot define a fillet direction.")]
    DegenerateCorner,
    /// the Jacobian of fillet equations became degenerate.
    #[error("failed to solve fillet candidate because the Jacobian became degenerate. {0}")]
    DegenerateFilletJacobian(String),
    /// Newton method did not converge while solving the fillet equations.
    #[error("failed to solve fillet candidate because Newton method did not converge. {0}")]
    FilletNewtonNotConverged(String),
    /// the chamfer distance must be positive.
    #[error("the chamfer distance must be positive.")]
    NonPositiveChamferDistance,
    /// the requested curve length goes outside the curve parameter range.
    #[error("the requested curve length goes outside the curve parameter range.")]
    CurveLengthOutOfRange,
    /// corner operations require a continuous wire.
    #[error("corner operations require a continuous wire.")]
    NonContinuousWire,
    /// error from `truck_geometry::errors::Error`.
    #[error("{0}")]
    GeometricError(truck_geometry::errors::Error),
    /// error from `truck_topology::errors::Error`.
    #[error("{0}")]
    TopologicalError(truck_topology::errors::Error),
}

impl From<truck_geometry::errors::Error> for Error {
    fn from(value: truck_geometry::errors::Error) -> Self { Self::GeometricError(value) }
}

impl From<truck_topology::errors::Error> for Error {
    fn from(value: truck_topology::errors::Error) -> Self { Self::TopologicalError(value) }
}
