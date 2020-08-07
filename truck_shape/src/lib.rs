#[macro_use]
extern crate truck_geometry as geometry;
extern crate truck_polymesh as polymesh;
extern crate truck_topology as topology;
use std::collections::HashMap;

type Point3 = cgmath::Point3<f64>;
use geometry::*;
type BSplineCurve = geometry::BSplineCurve<Vector4>;
type BSplineSurface = geometry::BSplineSurface<Vector4>;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Director {
    surfaces: HashMap<usize, BSplineSurface>,
    curves: HashMap<usize, BSplineCurve>,
    points: HashMap<usize, Vector4>,
}

pub struct Builder<'a> {
    director: &'a mut Director,
}

pub struct Mesher<'a> {
    director: &'a mut Director,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Transform(Matrix4);

pub type Result<T> = std::result::Result<T, crate::errors::Error>;

fn get_typename<T>(_: T) -> &'static str { std::any::type_name::<T>() }

pub mod builder;
pub mod director;
pub mod elements;
pub mod errors;
pub mod geom_impls;
pub mod mesher;
pub mod rsweep;
pub mod topo_impls;
pub mod topological_curve;
pub mod transform;
pub mod transformed;
pub mod tsweep;
