#[derive(Debug)]
pub enum Error {
    NoNormal,
    DifferentLengthArrays,
    IrregularArray,
    UnsortedDivision,
    FromIO(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NoNormal => f.pad("This mesh has no normal vectors."),
            Error::DifferentLengthArrays => {
                f.pad("The length of point vector and the one of normal vector are different.")
            }
            Error::IrregularArray => f.pad("This 2-dim array is irregular."),
            Error::UnsortedDivision => f.pad("This division vector is unsorted."),
            Error::FromIO(error) => f.write_fmt(format_args!("{:?}", error)),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error { Error::FromIO(error) }
}

impl std::convert::From<std::num::ParseFloatError> for Error {
    fn from(error: std::num::ParseFloatError) -> Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, error).into()
    }
}

impl std::convert::From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, error).into()
    }
}

impl std::error::Error for Error {}
