/// Errors occured by polygon mesh handling
#[derive(Debug)]
pub enum Error {
    /// There is an index in out of range.
    OutOfRange,
    /// There are not enough attribute to convert.
    NotEnoughAttrs,
    /// There are no normal in polygon mesh.
    NoNormal,
    /// The length of arrays of `StructuredMesh` is incorrect.
    DifferentLengthArrays,
    /// The length of arrays of `StructuredMesh` is incorrect.
    IrregularArray,
    /// The division of uv coords of `StructuredMesh` is not sorted.
    UnsortedDivision,
    /// Errors caused by obj files I/O.
    FromIO(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::OutOfRange => f.pad("The index is out of range."),
            Error::NotEnoughAttrs => f.pad("The polygon mesh does not have enough attribute."),
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

impl From<std::num::ParseFloatError> for Error {
    fn from(error: std::num::ParseFloatError) -> Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, error).into()
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, error).into()
    }
}

impl std::error::Error for Error {}
