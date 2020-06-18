#[derive(Debug, PartialEq)]
pub enum Error {
    SameVertex,
    EmptyWire,
    NotClosedWire,
    NotSimpleWire,
    NotRegularShell,
    EmptyShell,
    NotConnected,
    NotClosedShell,
    NotManifold,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::SameVertex => f.pad("Two same vertices cannot construct the edge."),
            Error::EmptyWire => f.pad("This wire is empty."),
            Error::NotClosedWire => f.pad("The boundary of face must be closed."),
            Error::NotSimpleWire => f.pad("The boundary of face must be simple."),
            Error::EmptyShell => f.pad("This shell is empty."),
            Error::NotConnected => f.pad("This shell is not connected."),
            Error::NotRegularShell => f.pad("This shell is not regular."),
            Error::NotClosedShell => f.pad("This shell is not oriented and closed."),
            Error::NotManifold => f.pad("This shell is not a manifold."),
        }
    }
}

impl std::error::Error for Error {}

#[test]
fn print_messages() {
    use std::io::Write;
    writeln!(&mut std::io::stderr(), "****** test of the expressions of error messages ******\n").unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::SameVertex).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::EmptyWire).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotClosedWire).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotSimpleWire).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::EmptyShell).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotConnected).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotClosedShell).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotManifold).unwrap();
    writeln!(&mut std::io::stderr(), "*******************************************************").unwrap();
}


