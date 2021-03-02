use crate::*;
use std::ops::{Deref, DerefMut, Mul};

/// surface constructed by revoluting a curve
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RevolutedCurve<C> {
    curve: C,
    origin: Point3,
    axis: Vector3,
}

/// invertible and transformable geometric element
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Processor<E, T> {
    entity: E,
    transform: T,
    orientation: bool,
}

mod revolved_curve;
mod processor;
