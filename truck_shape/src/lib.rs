extern crate truck_geometry as geometry;
extern crate truck_topology as topology;
use topology::*;
use geometry::*;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Geometry {
    pub surfaces: HashMap<usize, BSplineSurface>,
    pub curves: HashMap<usize, BSplineCurve>,
    pub points: HashMap<usize, Vector>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Volume {
    solid: Solid,
    geom: Geometry,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Surface {
    shell: Shell,
    geom: Geometry,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Curve {
    wire: Wire,
    geom: Geometry,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Point(pub Vertex, pub Vector);

impl Point {
    pub fn by_coord(coord: Vector) -> Point { Point (Vertex::new(), coord) }
}

pub type Result<T> = std::result::Result<T, crate::errors::Error>;

pub mod curve;
pub mod shape_geometry;
pub mod errors;
pub mod surface;
pub mod volume;
