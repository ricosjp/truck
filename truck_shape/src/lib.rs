extern crate truck_geometry as geometry;
extern crate truck_topology as topology;
use geometry::{BSplineCurve, BSplineSurface, Matrix, Vector};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Director {
    pub surfaces: HashMap<usize, BSplineSurface>,
    pub curves: HashMap<usize, BSplineCurve>,
    pub points: HashMap<usize, Vector>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Transform(Matrix);

pub type Result<T> = std::result::Result<T, crate::errors::Error>;

pub mod director;
pub mod errors;
pub mod transform;
pub mod transformed;
pub mod utility;
