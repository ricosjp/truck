extern crate obj as extern_obj;
extern crate serde;
extern crate truck_geometry as geometry;
extern crate truck_polymesh as polymesh;
extern crate truck_topology as topology;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use truck_geometry::*;

/// geometric data
#[derive(Clone, Debug, Default)]
pub struct GeomData {
    pub curves: Vec<BSplineCurve>,
    pub surfaces: Vec<BSplineSurface>,
}

/// geometric  
#[derive(Clone, Debug, Default)]
pub struct GeomDataRef<'a> {
    pub curves: Vec<&'a BSplineCurve>,
    pub surfaces: Vec<&'a BSplineSurface>,
}

#[derive(Debug)]
pub enum Error {
    TopologyError(truck_topology::errors::Error),
    GeometryError(truck_geometry::errors::Error),
    IOError(std::io::Error),
    SerdeError(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::TopologyError(error) => error.fmt(f),
            Error::GeometryError(error) => error.fmt(f),
            Error::IOError(error) => error.fmt(f),
            Error::SerdeError(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl std::convert::From<truck_topology::errors::Error> for Error {
    fn from(error: truck_topology::errors::Error) -> Error { Error::TopologyError(error) }
}

impl std::convert::From<truck_geometry::errors::Error> for Error {
    fn from(error: truck_geometry::errors::Error) -> Error { Error::GeometryError(error) }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error { Error::IOError(error) }
}

impl std::convert::From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error { Error::SerdeError(error) }
}

impl std::convert::From<std::num::ParseFloatError> for Error {
    fn from(error: std::num::ParseFloatError) -> Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, error).into()
    }
}

impl std::convert::From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, error).into()
    }
}

pub mod obj;
pub mod tgb;
pub mod tts;
