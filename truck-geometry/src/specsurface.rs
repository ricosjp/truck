use crate::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    matrix: Matrix4,
    parameter_range: ((f64, f64), (f64, f64)),
}
