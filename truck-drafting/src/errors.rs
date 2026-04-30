use thiserror::Error;

/// Drafting errors
#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    /// cannot construct a circle arc from collinear points.
    #[error("cannot construct a circle arc from collinear points.")]
    CollinearArcPoints,
    /// cannot construct a circle arc when the tangent is parallel to the chord.
    #[error("cannot construct a circle arc when the tangent is parallel to the chord.")]
    ParallelArcTangent,
    /// the fillet radius must be positive.
    #[error("the fillet radius must be positive.")]
    NonPositiveFilletRadius,
    /// the tangent vector at the specified parameter vanished.
    #[error("the tangent vector vanished near the specified corner.")]
    DegenerateTangent,
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
}
