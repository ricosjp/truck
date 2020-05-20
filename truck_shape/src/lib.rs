use truck_geometry::*;
use truck_topology::*;
use std::collections::HashMap;

pub struct Geometry {
    surfaces: HashMap<usize, BSplineSurface>,
    curves: HashMap<usize, BSplineCurve>,
    points: HashMap<usize, Vector>,
}

pub struct Volume {
    topology: Solid,
    geometry: Geometry,
}

pub struct Surface {
    topology: Shell,
    geometry: Geometry,
}

pub type Result<T> = std::result::Result<T, crate::errors::Error>;

mod geometry;
mod errors;
