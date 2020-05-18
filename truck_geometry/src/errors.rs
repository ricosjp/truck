use crate::*;

#[derive(Debug, PartialEq)]
pub enum Error {
    IrregularMatrix(Matrix),
    ZeroRange,
    DifferentBackFront(f64, f64),
    NotClampedKnotVector(Vec<f64>, Vec<f64>),
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
            Error::IrregularMatrix(mat) => f.write_fmt(format_args!("This matrix is irregular.\n{}", mat)),
            Error::ZeroRange => f.pad("This knot vector consists single value."),
            Error::DifferentBackFront(knot0, knot1) => f.pad(&format!("Cannot concat two knot vectors whose the back of the first and the front of the second are different.\nthe back of the first knot vector: {}\nthe front of the second knot vector: {}", knot0, knot1)),
            Error::NotClampedKnotVector(_, _) => f.pad("This knot vector is not clamped."),
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
