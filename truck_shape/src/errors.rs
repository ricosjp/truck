use crate::director::TopoGeomIntegrity;

#[derive(PartialEq, Debug)]
pub enum Error {
    None,
    NoSurfaceFace(usize),
    NoCurveEdge(usize),
    NoPointVertex(usize),
    NotBoundary(usize, usize),
    NotEndPoint(usize, usize),
    DifferentNumOfFacesAndSurfaces,
    DifferentNumOfEdgesAndCurves,
    DifferentNumOfVertexAndPoints,
    WireIsNotSimple,
    EmptyPointIter,
    FromGeometry(geometry::errors::Error),
    FromTopology(topology::errors::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::None => f.pad("No Error"),
            Error::NoSurfaceFace(face_id) => f.write_fmt(format_args!("The face with id = {} does not correspond to a surface.", face_id)),
            Error::NoCurveEdge(edge_id) => f.write_fmt(format_args!("The edge with id = {} does not correspond to a surface.", edge_id)),
            Error::NoPointVertex(vertex_id) => f.write_fmt(format_args!("The vertex with id = {} does not correspond to a surface.", vertex_id)),
            Error::NotBoundary(face_id, edge_id) => f.write_fmt(format_args!("The curve which corresponds to the edge with id = {} is not in the boundary of the surface which corresponds to the face with id = {}.", edge_id, face_id)),
            Error::NotEndPoint(edge_id, vertex_id) => f.write_fmt(format_args!("The point which corresponds to the vertex with id = {} is not the end point of the curve which corresponds to the edge with id = {}.", vertex_id, edge_id)),
            Error::DifferentNumOfFacesAndSurfaces => f.pad("The number of faces in topology is not equal to the one of surfaces in geometry."),
            Error::DifferentNumOfEdgesAndCurves => f.pad("The number of edges in topology is not equal to the one of curves in geometry."),
            Error::DifferentNumOfVertexAndPoints => f.pad("The number of edges in topology is not equal to the one of points in geometry."),
            Error::WireIsNotSimple => f.pad("This wire is not simple."),
            Error::EmptyPointIter => f.pad("This iterator has no points."),
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
            TopoGeomIntegrity::NoSurfaceFace { face_id } => Error::NoSurfaceFace(face_id),
            TopoGeomIntegrity::NoCurveEdge { edge_id } => Error::NoCurveEdge(edge_id),
            TopoGeomIntegrity::NoPointVertex { vertex_id } => Error::NoPointVertex(vertex_id),
            TopoGeomIntegrity::NotBoundary { face_id, edge_id } => Error::NotBoundary(face_id, edge_id),
            TopoGeomIntegrity::NotEndPoint { edge_id, vertex_id } => Error::NotEndPoint(edge_id, vertex_id),
        }
    }
}
