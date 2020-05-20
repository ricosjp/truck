use truck_geometry::*;

#[derive(PartialEq, Debug)]
pub enum Error {
    ConflictPoints(usize, Vector, Vector),
    ConflictCurves(BSplineCurve, BSplineCurve),
    CannotDetermineGeometry,
    FromGeometry(truck_geometry::errors::Error),
    FromTopology(truck_topology::errors::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ConflictPoints(vertex, vector0, vector1) => f.pad(&format!("This point conflict the point which is already attached to the vertex.\nVertexID: {}\nExisting Point: {:?}\nNew Point{:?}", vertex, vector0, vector1)),
            Error::ConflictCurves(curve0, curve1) => f.pad(&format!("This new surface conflict the curve which is already attached to the edge.\nEdgeID: {:?}\nFaceID: {:?}\n", curve0, curve1)),
            Error::CannotDetermineGeometry => f.pad("cannot determine geometry."),
            Error::FromGeometry(err) => err.fmt(f),
            Error::FromTopology(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl std::convert::From<truck_geometry::errors::Error> for Error {
    fn from(err: truck_geometry::errors::Error) -> Error {
        Error::FromGeometry(err)
    }
}

impl std::convert::From<truck_topology::errors::Error> for Error {
    fn from(err: truck_topology::errors::Error) -> Error {
        Error::FromTopology(err)
    }
}
