/// Modeling errors
#[derive(Debug, PartialEq)]
pub enum Error {
    /// wrapper of topological error
    FromTopology(truck_topology::errors::Error),
    /// tried to attach a plane to a wire that was not on one plane.
    /// cf. [`builder::try_attach_plane`](../builder/fn.try_attach_plane.html)
    WireNotInOnePlane,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::FromTopology(error) => error.fmt(f),
            Error::WireNotInOnePlane => f.pad("cannot attach a plane to a wire that is not on one plane."),
        }
    }
}

impl std::error::Error for Error {}

impl From<truck_topology::errors::Error> for Error {
    #[inline(always)]
    fn from(error: truck_topology::errors::Error) -> Error { Error::FromTopology(error) }
}

#[test]
fn print_messages() {
    use std::io::Write;
    writeln!(&mut std::io::stderr(), "****** test of the expressions of error messages ******\n").unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::FromTopology(truck_topology::errors::Error::SameVertex)).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::WireNotInOnePlane).unwrap();
    writeln!(&mut std::io::stderr(), "*******************************************************").unwrap();
}
