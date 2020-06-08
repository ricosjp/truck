#[derive(Debug, PartialEq)]
pub enum Error {
    SameVertex,
    CannotAddEdge,
    NotClosedWire,
    NotSimpleWire,
    EmptyShell,
    NotConnectedManifold,
    NotClosedShell,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::SameVertex => f.pad("Two same vertices cannot construct the edge."),
            Error::CannotAddEdge => f.pad("This edge cannot add in this wire."),
            Error::NotClosedWire => f.pad("The boundary of face must be closed."),
            Error::NotSimpleWire => f.pad("The boundary of face must be simple."),
            Error::EmptyShell => f.pad("This shell is empty."),
            Error::NotConnectedManifold => f.pad("This shell is not a connected manifold."),
            Error::NotClosedShell => f.pad("This shell is not oriented and closed."),
        }
    }
}

impl std::error::Error for Error {}

#[test]
fn print_messages() {
    use std::io::Write;
    writeln!(&mut std::io::stderr(), "****** test of the expressions of error messages ******\n").unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::SameVertex).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::CannotAddEdge).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotClosedWire).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotSimpleWire).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::EmptyShell).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotConnectedManifold).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotClosedShell).unwrap();
    writeln!(&mut std::io::stderr(), "*******************************************************").unwrap();
}


