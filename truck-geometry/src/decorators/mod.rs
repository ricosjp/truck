use crate::*;

/// surface constructed by revoluting a curve
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevolutedCurve<C> {
    curve: C,
    origin: Point3,
    axis: Vector3,
}

mod revolved_curve;
