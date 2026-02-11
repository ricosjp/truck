use thiserror::Error;

/// Errors that can occur during fillet operations.
#[derive(Debug, Error)]
pub enum FilletError {
    /// Variable radius is not supported for wire-based fillets.
    #[error("Variable radius is not supported for fillet_along_wire.")]
    VariableRadiusUnsupported,
    /// The wire is not continuous.
    #[error("Wire must be continuous.")]
    DiscontinuousWire,
    /// Could not find a shared face for the wire's front edge.
    #[error("Shared face not found.")]
    SharedFaceNotFound,
    /// Could not find adjacent faces for all edges in the wire.
    #[error("Adjacent faces not found.")]
    AdjacentFacesNotFound,
    /// Failed to compute fillet surfaces along the wire.
    #[error("Fillet surface computation failed.")]
    FilletSurfaceComputationFailed,
    /// A geometry computation failed.
    #[error("Geometry failed: {context}.")]
    GeometryFailed {
        /// Description of which step failed.
        context: &'static str,
    },
    /// The edge is shared by a non-manifold number of faces.
    #[error("Non-manifold edge: shared by {0} faces.")]
    NonManifoldEdge(usize),
    /// The requested edge was not found in the shell.
    #[error("Edge not found in shell.")]
    EdgeNotFound,
    /// The shell contains geometry that cannot be converted for filleting.
    #[error("Unsupported geometry: {context}.")]
    UnsupportedGeometry {
        /// Description of which conversion failed.
        context: &'static str,
    },
    /// The edge is too short for the requested fillet radius.
    #[error("Edge too short for fillet radius.")]
    DegenerateEdge,
    /// Per-edge radius count does not match the number of edges.
    #[error("Per-edge radius count ({given}) does not match edge count ({expected}).")]
    PerEdgeRadiusMismatch {
        /// Number of radii provided.
        given: usize,
        /// Number of edges requested.
        expected: usize,
    },
}
