use thiserror::Error;

/// Modeling errors.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    /// Wrapper of topological error.
    #[error(transparent)]
    FromTopology(#[from] monstertruck_topology::errors::Error),
    /// Tried to attach a plane to a wire that was not on one plane.
    /// cf. [`builder::try_attach_plane`](../builder/fn.try_attach_plane.html)
    #[error("cannot attach a plane to a wire that is not on one plane.")]
    WireNotInOnePlane,
    /// Tried to create homotopy for two wires with different numbers of edges.
    /// cf. [`builder::try_wire_homotopy`](../builder/fn.try_wire_homotopy.html)
    #[error("The wires must contain the same number of edges to create a homotopy.")]
    NotSameNumberOfEdges,
    /// One or more wires are not closed.
    #[error("all wires must be closed for profile construction.")]
    OpenWire,
    /// Ambiguous nesting: a loop is not clearly inside or outside another.
    #[error("ambiguous nesting between loops; loops may overlap or touch.")]
    AmbiguousNesting,
    /// No outer loop found among the provided wires.
    #[error("no outer loop found; at least one wire must have positive signed area.")]
    NoOuterLoop,
}

#[test]
fn print_messages() {
    use std::io::Write;
    writeln!(
        &mut std::io::stderr(),
        "****** test of the expressions of error messages ******\n"
    )
    .unwrap();
    writeln!(
        &mut std::io::stderr(),
        "{}\n",
        Error::FromTopology(monstertruck_topology::errors::Error::SameVertex)
    )
    .unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::WireNotInOnePlane).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::OpenWire).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::AmbiguousNesting).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NoOuterLoop).unwrap();
    writeln!(
        &mut std::io::stderr(),
        "*******************************************************"
    )
    .unwrap();
}
