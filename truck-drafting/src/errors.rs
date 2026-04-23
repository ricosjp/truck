use thiserror::Error;

/// Drafting errors
#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    /// the fillet radius must be positive.
    #[error("the fillet radius must be positive.")]
    NonPositiveFilletRadius,
    /// the tangent vector at the specified parameter vanished.
    #[error("the tangent vector vanished near the specified corner.")]
    DegenerateTangent,
    /// the Jacobian of fillet equations became degenerate.
    #[error("failed to solve fillet candidate because the Jacobian became degenerate. {0}")]
    DegenerateFilletJacobian(String),
    /// Newton method did not converge while solving the fillet equations.
    #[error("failed to solve fillet candidate because Newton method did not converge. {0}")]
    FilletNewtonNotConverged(String),
}
