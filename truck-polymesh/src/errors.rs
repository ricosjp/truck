use thiserror::Error;

/// Errors occured by polygon mesh handling
#[derive(Debug, Error)]
pub enum Error<V: std::fmt::Debug = crate::StandardVertex> {
    /// There is an index in out of range.
    /// # Examples
    /// ```
    /// use truck_polymesh::*;
    /// use std::iter::FromIterator;
    /// use errors::Error;
    /// 
    /// let positions = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[0, 1, 2],
    ///     &[1, 2, 4],
    /// ]);
    /// 
    /// let res = PolygonMesh::try_new(
    ///     StandardAttributes {
    ///         positions,
    ///         ..Default::default()
    ///     },
    ///     faces,
    /// );
    /// match res {
    ///     Err(Error::OutOfRange(vertex)) => {
    ///         assert_eq!(vertex.pos, 4);
    ///     }
    ///     _ => panic!("wrong result!"),
    /// }
    /// ```
    #[error("The index {0:?} is out of range.")]
    OutOfRange(V),
    /// There are no normal in polygon mesh.
    #[error("This mesh has no normal vectors.")]
    NoNormal,
    /// The length of arrays of `StructuredMesh` is incorrect.
    #[error("The lengthes of point vector, uvdivisions, normal vector are incompatible.")]
    DifferentLengthArrays,
    /// The length of arrays of `StructuredMesh` is incorrect.
    /// # Examples
    /// ```
    /// use truck_polymesh::*;
    /// use errors::Error;
    /// 
    /// let positions = vec![
    ///     vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)],
    ///     vec![Point3::new(0.0, 1.0, 0.0)],
    /// ];
    /// 
    /// match StructuredMesh::try_from_positions(positions) {
    ///     Err(Error::IrregularArray) => {}
    ///     _ => panic!("wrong result!"),
    /// }
    /// ```
    #[error("This 2-dim array is irregular.")]
    IrregularArray,
    /// The division of uv coords of `StructuredMesh` is not sorted.
    /// # Examples
    /// ```
    /// use truck_polymesh::*;
    /// use errors::Error;
    /// 
    /// let positions = vec![
    ///     vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0)],
    ///     vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0)],
    /// ];
    /// 
    /// let udiv = vec![1.0, 0.0];
    /// let vdiv = vec![0.0, 1.0];
    /// 
    /// match StructuredMesh::try_from_positions_and_uvs(positions, (udiv, vdiv)) {
    ///     Err(Error::UnsortedDivision) => {}
    ///     _ => panic!("wrong result!"),
    /// }
    /// ``` 
    #[error("This division vector is unsorted.")]
    UnsortedDivision,
    /// Errors caused by obj files I/O.
    #[error(transparent)]
    FromIO(#[from] std::io::Error),
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
