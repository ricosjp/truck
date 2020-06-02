#[derive(Debug, PartialEq)]
pub enum Error {
    NoNormal,
    DifferentLengthArrays,
    IrregularArray,
    UnsortedDivision,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::NoNormal => f.pad("This mesh has no normal vectors."),
            Error::DifferentLengthArrays => f.pad("The length of point vector and the one of normal vector are different."),
            Error::IrregularArray => f.pad("This 2-dim array is irregular."),
            Error::UnsortedDivision => f.pad("This division vector is unsorted."),
        }
    }
}