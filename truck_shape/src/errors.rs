use crate::director::TopoGeomIntegrity;

#[derive(PartialEq, Debug)]
pub enum Error {
    None,
    NoGeometry(&'static str, usize),
    NotBoundary(usize, usize),
    NotEndPoint(usize, usize),
    NonPositiveWeightedPoint,
    DifferentNumOfFacesAndSurfaces,
    DifferentNumOfEdgesAndCurves,
    DifferentNumOfVertexAndPoints,
    DifferentHomotopyType,
    WireIsNotSimple,
    EmptyPointIter,
    NotStartingOrigin,
    IrregularShell,
    FromGeometry(geometry::errors::Error),
    FromTopology(topology::errors::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::None => f.pad("No Error"),
            Error::NoGeometry(typename, id) => f.write_fmt(format_args!("There are no geometry corresponding to the {} with id = {}", typename, id)),
            Error::NotBoundary(face_id, edge_id) => f.write_fmt(format_args!("The curve which corresponds to the edge with id = {} is not in the boundary of the surface which corresponds to the face with id = {}.", edge_id, face_id)),
            Error::NotEndPoint(edge_id, vertex_id) => f.write_fmt(format_args!("The point which corresponds to the vertex with id = {} is not the end point of the curve which corresponds to the edge with id = {}.", vertex_id, edge_id)),
            Error::NonPositiveWeightedPoint => f.write_fmt(format_args!("The 4th component of Vector is not positive.")),
            Error::DifferentNumOfFacesAndSurfaces => f.pad("The number of faces in topology is not equal to the one of surfaces in geometry."),
            Error::DifferentNumOfEdgesAndCurves => f.pad("The number of edges in topology is not equal to the one of curves in geometry."),
            Error::DifferentNumOfVertexAndPoints => f.pad("The number of edges in topology is not equal to the one of points in geometry."),
            Error::DifferentHomotopyType => f.pad("The one curve element is open and the other closed."),
            Error::WireIsNotSimple => f.pad("This wire is not simple."),
            Error::EmptyPointIter => f.pad("This iterator has no points."),
            Error::NotStartingOrigin => f.pad("This curve does not start from (0, 0, 0, 1)."),
            Error::IrregularShell => f.pad("This shell is irregular."),
            Error::FromGeometry(err) => err.fmt(f),
            Error::FromTopology(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl std::convert::From<truck_geometry::errors::Error> for Error {
    fn from(err: truck_geometry::errors::Error) -> Error { Error::FromGeometry(err) }
}

impl std::convert::From<truck_topology::errors::Error> for Error {
    fn from(err: truck_topology::errors::Error) -> Error { Error::FromTopology(err) }
}

impl std::convert::From<TopoGeomIntegrity> for Error {
    fn from(integrity: TopoGeomIntegrity) -> Error {
        match integrity {
            TopoGeomIntegrity::Integrate => Error::None,
            TopoGeomIntegrity::NoGeometryElement { typename, id } => {
                Error::NoGeometry(typename, id)
            },
            TopoGeomIntegrity::NonPositiveWeightedPoint => {
                Error::NonPositiveWeightedPoint
            }
            TopoGeomIntegrity::NotBoundary { face_id, edge_id } => {
                Error::NotBoundary(face_id, edge_id)
            }
            TopoGeomIntegrity::NotEndPoint { edge_id, vertex_id } => {
                Error::NotEndPoint(edge_id, vertex_id)
            }
        }
    }
}
