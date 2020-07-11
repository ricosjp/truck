use crate::*;

/// Geometrical Errors
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The iterator is too short to construct the given dimensional vector.  
    /// This error is only for the output panic error message.
    /// # Examples
    /// ```should_panic
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let arr = vec![0.0, 1.0, 2.0];
    /// let _: Vector<[f64; 4]> = arr.into_iter().collect();
    /// ```
    TooShortIterator,
    /// The following operations are failed if the knot vector has zero range.
    /// * Creating `BSplineCurve` or `BSplineSurface`,
    /// * Calculating bspline basis functions, or
    /// * Normalizing the knot vector.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let mut knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0]);
    /// assert_eq!(knot_vec.try_normalize(), Err(Error::ZeroRange));
    /// assert_eq!(knot_vec.try_bspline_basis_functions(1, 0.0), Err(Error::ZeroRange));
    /// 
    /// let ctrl_pts = vec![vector!(0, 0), vector!(1, 1)];
    /// assert_eq!(BSplineCurve::try_new(knot_vec, ctrl_pts), Err(Error::ZeroRange));
    /// ```
    ZeroRange,
    /// Fails concatting two knot vectors if there is a difference between the back knot of
    /// the former knot vector and the front knot of the latter knot vector.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let mut knot_vec0 = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let knot_vec1 = KnotVec::from(vec![2.0, 2.0, 3.0, 3.0]);
    /// assert_eq!(knot_vec0.try_concat(&knot_vec1, 1), Err(Error::DifferentBackFront(1.0, 2.0)));
    /// ```
    DifferentBackFront(f64, f64),
    /// If the knot vector is not clamped, then one cannot concat the vector with another knot vector.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// let mut knot_vec0 = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    /// let knot_vec1 = KnotVec::from(vec![2.0, 2.0, 3.0, 3.0]);
    /// assert_eq!(knot_vec0.try_concat(&knot_vec1, 2), Err(Error::NotClampedKnotVector));
    /// ```
    NotClampedKnotVector,
    /// Creating a knot vector by `KnotVec::try_from()` is failed if the given vector is not sorted.
    /// `<KnotVec as From<Vec<f64>>>::from()` does not panic by this error because sorts the given
    /// vector before creating the knot vector. So, `KnotVec::try_from()` is more efficient than
    /// `<KnotVec as From<Vec<f64>>>::from()`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use errors::Error;
    /// use std::convert::*;
    /// 
    /// assert_eq!(KnotVec::try_from(vec![1.0, 3.0, 0.0, 2.0]), Err(Error::NotSortedVector));
    /// assert_eq!(
    ///     <KnotVec as From<Vec<f64>>>::from(vec![1.0, 3.0, 0.0, 2.0]),
    ///     KnotVec::try_from(vec![0.0, 1.0, 2.0, 3.0]).unwrap(),
    /// );
    /// ```
    NotSortedVector,
    EmptyKnotVector,
    SmallerThanSmallestKnot(f64, f64),
    TooLargeDegree(usize, usize),
    CannotRemoveKnot(usize),
    EmptyControlPoints,
    TooShortKnotVector(usize, usize),
    TooSmallNewKnot(f64, f64),
    ConstantCurve,
    IrregularControlPoints,
    NotConverge,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::TooShortIterator => f.pad("The length of the iterator is too short."),
            Error::ZeroRange => f.pad("This knot vector consists single value."),
            Error::DifferentBackFront(knot0, knot1) => f.pad(&format!("Cannot concat two knot vectors whose the back of the first and the front of the second are different.\nthe back of the first knot vector: {}\nthe front of the second knot vector: {}", knot0, knot1)),
            Error::NotClampedKnotVector => f.pad("This knot vector is not clamped."),
            Error::NotSortedVector => f.pad("This knot vector is not sorted."),
            Error::EmptyKnotVector => f.pad("This knot vector is empty."),
            Error::SmallerThanSmallestKnot(x, smallest) => f.pad(
                &format!("The input value is smaller than the smallest knot.\nthe input value: {}\nthe smaller knot: {}", x, smallest)
                ),
            Error::TooLargeDegree(knot_len, degree) => f.pad(
                &format!("This knot vector is too short compared to the degree.\nthe length of knot_vec: {}\nthe degree: {}",
                    knot_len, degree)
                ),
            Error::CannotRemoveKnot(idx) => f.pad(&format!("The {}th knot in this knot vector cannot be removed.", idx)),
            Error::EmptyControlPoints => f.pad("The control point must not be empty."),
            Error::TooShortKnotVector(knot_len, cont_len) => f.pad(
                &format!("The knot vector must be more than the control points.\nthe length of knot_vec: {}\nthe number of control points: {}",
                    knot_len, cont_len)
                ),
            Error::TooSmallNewKnot(new_knot, smallest) => f.pad(
                &format!("The new knot must be smaller than the minimum knot.\nthe new knot: {}\nthe smallest knot: {}",
                    new_knot, smallest)
                ),
            Error::ConstantCurve => f.pad("This curve is global"),
            Error::IrregularControlPoints => f.pad("The number of control points is irregular"),
            Error::NotConverge => f.pad("Newton method did not converge."),
        }
    }
}

impl std::error::Error for Error {}

#[test]
fn print_messages() {
    use std::io::Write;
    writeln!(&mut std::io::stderr(), "****** test of the expressions of error messages ******\n").unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::TooShortIterator).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::ZeroRange).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::DifferentBackFront(0.0, 1.0)).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotSortedVector).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::EmptyKnotVector).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::SmallerThanSmallestKnot(0.0, 1.0)).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::TooLargeDegree(2, 1)).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::CannotRemoveKnot(7)).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::EmptyControlPoints).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::TooShortKnotVector(1, 2)).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::TooSmallNewKnot(1.0, 2.0)).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::IrregularControlPoints).unwrap();
    writeln!(&mut std::io::stderr(), "{}\n", Error::NotConverge).unwrap();
    writeln!(&mut std::io::stderr(), "*******************************************************").unwrap();
}
