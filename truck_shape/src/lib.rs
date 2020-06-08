extern crate truck_geometry as geometry;
extern crate truck_topology as topology;
extern crate truck_polymesh as polymesh;
use geometry::{BSplineCurve, BSplineSurface, Matrix, Vector};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Director {
    surfaces: HashMap<usize, BSplineSurface>,
    curves: HashMap<usize, BSplineCurve>,
    points: HashMap<usize, Vector>,
}

pub struct Builder<'a> {
    director: &'a mut Director,
}

pub struct Mesher<'a> {
    director: &'a mut Director
}

#[derive(Clone, PartialEq, Debug)]
pub struct Transform(Matrix);

pub type Result<T> = std::result::Result<T, crate::errors::Error>;

pub mod curve_element;
pub mod builder;
pub mod director;
pub mod elements;
pub mod errors;
pub mod mesher;
pub mod tsweep;
pub mod transform;
pub mod transformed;
