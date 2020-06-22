extern crate truck_geometry as geometry;
extern crate truck_polymesh as polymesh;
extern crate truck_topology as topology;
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
    director: &'a mut Director,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Transform(Matrix);

pub type Result<T> = std::result::Result<T, crate::errors::Error>;

fn get_typename<T>(_: T) -> &'static str { std::any::type_name::<T>() }

pub mod builder;
pub mod topological_curve;
pub mod director;
pub mod elements;
pub mod errors;
pub mod geom_impls;
pub mod mesher;
pub mod topo_impls;
pub mod rsweep;
pub mod transform;
pub mod transformed;
pub mod tsweep;
