use thiserror::Error;

/// Error handler for [`Error`](./errors/enum.Error.html)
pub type Result<T> = std::result::Result<T, Error>;

/// Geometrical Errors
#[derive(Debug, PartialEq, Error)]
pub enum Error {
    /// The following operations are failed if the knot vector has zero range.
    /// * Creating `BSplineCurve` or `BSplineSurface`,
    /// * Calculating bspline basis functions, or
    /// * Normalizing the knot vector.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    /// let mut knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0]);
    /// assert_eq!(knot_vec.try_normalize(), Err(Error::ZeroRange));
    /// assert_eq!(knot_vec.try_bspline_basis_functions(1, 0, 0.0), Err(Error::ZeroRange));
    ///
    /// let ctrl_pts = vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)];
    /// assert!(matches!(BSplineCurve::try_new(knot_vec, ctrl_pts), Err(Error::ZeroRange)));
    /// ```
    #[error("This knot vector consists single value.")]
    ZeroRange,
    /// Fails concatting two knot vectors if there is a difference between the back knot of
    /// the former knot vector and the front knot of the latter knot vector.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    /// let mut knot_vec0 = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let knot_vec1 = KnotVec::from(vec![2.0, 2.0, 3.0, 3.0]);
    /// assert!(matches!(knot_vec0.try_concat(&knot_vec1, 1), Err(Error::DifferentBackFront(1.0, 2.0))));
    /// ```
    #[error("Cannot concat two knot vectors whose the back of the first and the front of the second are different.
the back of the first knot vector: {0}
the front of the second knot vector: {1}")]
    DifferentBackFront(f64, f64),
    /// If the knot vector is not clamped, then one cannot concat the vector with another knot vector.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    /// let mut knot_vec0 = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let knot_vec1 = KnotVec::from(vec![2.0, 2.0, 3.0, 3.0]);
    /// assert!(matches!(knot_vec0.try_concat(&knot_vec1, 2), Err(Error::NotClampedKnotVector)));
    /// ```
    #[error("This knot vector is not clamped.")]
    NotClampedKnotVector,
    /// Creating a knot vector by `KnotVec::try_from()` is failed if the given vector is not sorted.
    /// `<KnotVec as From<Vec<f64>>>::from()` does not panic by this error because sorts the given
    /// vector before creating the knot vector. So, `KnotVec::try_from()` is more efficient than
    /// `<KnotVec as From<Vec<f64>>>::from()`.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    /// use std::convert::*;
    ///
    /// assert!(matches!(KnotVec::try_from(vec![1.0, 3.0, 0.0, 2.0]), Err(Error::NotSortedVector)));
    /// assert_eq!(
    ///     <KnotVec as From<Vec<f64>>>::from(vec![1.0, 3.0, 0.0, 2.0]),
    ///     KnotVec::try_from(vec![0.0, 1.0, 2.0, 3.0]).unwrap(),
    /// );
    /// ```
    #[error("This knot vector is not sorted.")]
    NotSortedVector,
    /// The given degree is too large to calculate bspline basis functions.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    ///
    /// // a knot vector with length = 4.
    /// let knot_vec = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// assert!(matches!(
    ///     knot_vec.try_bspline_basis_functions(5, 0, 0.5),
    ///     Err(Error::TooLargeDegree(4, 5)),
    /// ));
    /// ```
    #[error(
        "This knot vector is too short compared to the degree.
the length of knot_vec: {0}
the degree: {1}"
    )]
    TooLargeDegree(usize, usize),
    /// The specified knot cannot be removed.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts = vec![Vector2::new(-1.0, 1.0), Vector2::new(0.0, -1.0), Vector2::new(1.0, 1.0)];
    /// let mut bspcurve = BSplineCurve::new(knot_vec, ctrl_pts);
    /// let org_curve = bspcurve.clone();
    /// bspcurve.add_knot(0.5).add_knot(0.5).add_knot(0.25).add_knot(0.75);
    /// assert!(bspcurve.try_remove_knot(3).is_ok());
    /// assert!(matches!(bspcurve.try_remove_knot(2), Err(Error::CannotRemoveKnot(2))));
    /// ```
    #[error("The {0}th knot in this knot vector cannot be removed.")]
    CannotRemoveKnot(usize),
    /// Empty vector of points cannot construct B-spline.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    ///
    /// let knot_vec = KnotVec::bezier_knot(2);
    /// let ctrl_pts: Vec<Vector4> = Vec::new();
    /// assert!(matches!(
    ///     BSplineCurve::try_new(knot_vec, ctrl_pts),
    ///     Err(Error::EmptyControlPoints),
    /// ));
    /// ```
    #[error("The control point must not be empty.")]
    EmptyControlPoints,
    /// The knot vector of B-spline curves or B-spline surfaces must be longer than the corresponded
    /// array of control points.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    /// let knot_vec = KnotVec::from(vec![0.0, 1.0, 2.0]);
    /// let ctrl_pts = vec![Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0), Vector2::new(0.0, 0.0)];
    /// assert!(matches!(
    ///     BSplineCurve::try_new(knot_vec, ctrl_pts),
    ///     Err(Error::TooShortKnotVector(3, 4)),
    /// ));
    /// ```
    #[error(
        "The knot vector must be more than the control points.
the length of knot_vec: {0}
the number of control points: {1}"
    )]
    TooShortKnotVector(usize, usize),
    /// The length of the given arrays of control points to create a B-spline surface is irregular.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    /// let knot_vecs = (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2));
    /// let ctrl_pts = vec![
    ///     vec![Vector2::new(1.0, 2.0), Vector2::new(1.0, 2.0)], // length = 2
    ///     vec![Vector2::new(1.0, 2.0)] // length = 1
    /// ];
    /// assert!(matches!(
    ///     BSplineSurface::try_new(knot_vecs, ctrl_pts),
    ///     Err(Error::IrregularControlPoints),
    /// ));
    /// ```
    #[error("The number of control points is irregular")]
    IrregularControlPoints,
    /// The vector of control points and the one of weights have different length.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    /// let bspcurve = BSplineCurve::new(
    ///     KnotVec::bezier_knot(2),
    ///     vec![Point2::new(0.0, 1.0), Point2::new(2.0, 3.0), Point2::new(4.0, 5.0)],
    /// );
    /// let weights = vec![1.0, 2.0]; // less than control points
    ///
    /// assert!(matches!(
    ///     NurbsCurve::<Vector3>::try_from_bspline_and_weights(bspcurve, weights),
    ///     Err(Error::DifferentLength),
    /// ));
    /// ```
    #[error("The vector of control points and the one of weights have different length.")]
    DifferentLength,
    /// Gaussian eliminaition is failed. Typically, this is because one has included multiple
    /// parameters for which the B-spline basis functions take the same value.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// use truck_geometry::errors::Error;
    ///
    /// let knot_vec = KnotVec::uniform_knot(2, 2);
    /// let parameter_points = [
    ///     (0.1, Point3::new(1.0, 2.0, 3.0)),
    ///     (0.8, Point3::new(4.0, -1.0, 10.0)),
    ///     (0.8, Point3::new(-3.0, 5.0, 6.0)),
    ///     (1.0, Point3::new(6.0, 2.0, 12.0)),
    /// ];
    ///
    /// assert!(matches!(
    ///     BSplineCurve::try_interpole(knot_vec, parameter_points),
    ///     Err(Error::GaussianEliminationFailure),
    /// ));
    /// ```
    #[error("Gaussian elimination is failed.")]
    GaussianEliminationFailure,
    /// Cannot connect T-mesh control point to an existing connection.
    #[error("Cannot connect T-mesh control point to an existing connection.")]
    TmeshExistingConnection,
    /// Cannot locate the desired control point in the T-mesh.
    #[error("Cannot locate the desired control point in the T-mesh.")]
    TmeshControlPointNotFound,
    /// T-mesh connection does not exist.
    #[error("T-mesh connection does not exist.")]
    TmeshConnectionNotFound,
    /// T-mesh knot ratios for new connections must be between 0.0 and 1.0.
    #[error("T-mesh knot ratios for new connections must be between 0.0 and 1.0.")]
    TmeshInvalidKnotRatio,
    /// T-mesh knot intervals on a connection do not match in both directions.
    #[error("T-mesh knot intervals on a connection do not match in both directions.")]
    TmeshConnectionInvalidKnotInterval,
    /// An unknown T-mesh error occurred.
    #[error("An unknown T-mesh error occurred.")]
    TmeshUnkownError,
    /// The T-mesh is malformed and should no longer be used.
    #[error("The T-mesh is malformed and should no longer be used.")]
    TmeshMalformedMesh,
    /// A knot could not be inserted without violating Rule 3.
    #[error("A knot could not be inserted into the mesh at the desired location without violating Rule 3.")]
    TmeshKnotVectorsNotEqual,
    /// A control point with the same knot coordinates already exists.
    #[error("A control point with the same knot coordinates already exists.")]
    TmeshExistingControlPoint,
    /// The control point passed does not belong to the current `Tmesh`.
    #[error("The control point passed does not belong to the current Tmesh.")]
    TmeshForeignControlPoint,
    /// The control point being inserted is out of the bounds of the `Tmesh`.
    #[error("The control point being inserted is out of the bounds of the Tmesh.")]
    TmeshOutOfBoundsInsertion,
    /// The two T-NURCC points are already connected.
    #[error("The two points are already connected.")]
    TnurccExistingConnection,
    /// A T-NURCC face has opposing edges with differing knot intervals.
    #[error("A T-NURCC face has opposing edges with differing knot intervals.")]
    TnurccNonRectangularFace,
    /// A T-NURCC face edge must contain at least 2 control points.
    #[error("A T-NURCC face edge must contain at least 2 control points.")]
    TnurccIncompleteFaceEdge,
    /// A T-NURCC face must have at least two points and one edge defining it.
    #[error("A T-NURCC face must have at least two points and one edge defining it.")]
    TnurccMalformedFace,
    /// Automatically connecting two T-NURCC edges would result in a bad topological state.
    #[error("Automatically connecting the two edges {0} and {1} would result in a bad topological state.")]
    TnurccBadConnectionConditions(usize, usize),
    /// Missing face in T-NURCC constructor.
    #[error("Missing face in T-NURCC constructor.")]
    TnurccMissingFace,
    /// An edge in T-NURCC constructor has more than 2 faces.
    #[error("An edge in T-NURCC constructor has been defined to have more than 2 faces.")]
    TnurccEdgeTrippleFace,
}

#[test]
#[rustfmt::skip]
fn print_messages() {
    use std::io::Write;
    let stderr = &mut std::io::stderr();
    writeln!(stderr, "****** test of the expressions of error messages ******\n").unwrap();
    writeln!(stderr, "{}\n", Error::ZeroRange).unwrap();
    writeln!(stderr, "{}\n", Error::DifferentBackFront(0.0, 1.0)).unwrap();
    writeln!(stderr, "{}\n", Error::NotClampedKnotVector).unwrap();
    writeln!(stderr, "{}\n", Error::NotSortedVector).unwrap();
    writeln!(stderr, "{}\n", Error::TooLargeDegree(1, 2)).unwrap();
    writeln!(stderr, "{}\n", Error::CannotRemoveKnot(7)).unwrap();
    writeln!(stderr, "{}\n", Error::EmptyControlPoints).unwrap();
    writeln!(stderr, "{}\n", Error::TooShortKnotVector(1, 2)).unwrap();
    writeln!(stderr, "{}\n", Error::IrregularControlPoints).unwrap();
    writeln!(stderr, "*******************************************************").unwrap();
}
