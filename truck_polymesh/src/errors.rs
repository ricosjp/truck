#[derive(Debug, PartialEq)]
pub enum Error {
    NoNormal,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NoNormal => f.pad("This mesh has no normal vectors."),
        }
    }
}