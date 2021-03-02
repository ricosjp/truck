use crate::*;

/// surface constructed by revoluting a curve
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RevolutedCurve<C> {
    curve: C,
    origin: Point3,
    axis: Vector3,
}

/// transformed geometric element
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Transformed<E, T> {
    entity: E,
    transform: T,
}

/// Invertible entity
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Invertible<E>(E);

mod revolved_curve;
mod transformed;
mod invertible;
