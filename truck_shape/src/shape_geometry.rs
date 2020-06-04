use crate::Geometry;
use geometry::*;
use std::collections::HashMap;
use topology::*;

/// integrity of geometric information and shell
#[derive(PartialEq, Debug)]
pub enum GeometryShellIntegrity {
    /// Every face, edge, and vertice correspond to a surface, a curve, and a point, respectively.
    /// Moreover, the geometric information is compatible with the topological information.
    Integrate,
    /// The face with id = `face_id` does not correspond to a surface.
    NoSurfaceFace { face_id: usize },
    /// The edge with id = `edge_id` does not correspond to an edge.
    NoCurveEdge { edge_id: usize },
    /// The vertex with id=`vertex_id` does not correspond to a point.
    NoPointVertex { vertex_id: usize },
    /// The curve which corresponds to the edge with id = `edge_id` is not in the boundary of
    /// the surface which corresponds to the face with id = `face_id`.
    NotBoundary { face_id: usize, edge_id: usize },
    /// The point which corresponds to the vertex with id = `vertex_id` is not the end point of
    /// the curve which corresponds to the edge with id = `edge_id`.
    NotEndPoint { edge_id: usize, vertex_id: usize },
}

impl Geometry {
    pub fn new() -> Geometry {
        Geometry {
            surfaces: HashMap::new(),
            curves: HashMap::new(),
            points: HashMap::new(),
        }
    }

    pub fn check_shell_integrity(&mut self, shell: &Shell) -> GeometryShellIntegrity {
        for face in shell.face_iter() {
            let surface = match self.surfaces.get(&face.id()) {
                Some(got) => got,
                None => return GeometryShellIntegrity::NoSurfaceFace { face_id: face.id() },
            };
            let integrity = check_integrate_one_face(&self.points, &self.curves, face, surface);
            if integrity != GeometryShellIntegrity::Integrate {
                return integrity;
            }
        }
        GeometryShellIntegrity::Integrate
    }

    pub fn check_wire_integrity(&mut self, wire: &Wire) -> GeometryShellIntegrity {
        for edge in wire.edge_iter() {
            let curve = match self.curves.get(&edge.id()) {
                Some(got) => got,
                None => return GeometryShellIntegrity::NoCurveEdge { edge_id: edge.id() },
            };
            let integrity = check_integrate_one_edge(&self.points, edge, curve);
            if integrity != GeometryShellIntegrity::Integrate {
                return integrity;
            }
        }
        GeometryShellIntegrity::Integrate
    }

    pub fn attach_point(&mut self, vertex: &Vertex, point: Vector) -> Option<Vector> {
        self.points.insert(vertex.id(), point)
    }

    pub fn attach_curve(&mut self, edge: &Edge, curve: BSplineCurve) -> Option<BSplineCurve> {
        self.curves.insert(edge.id(), curve)
    }

    pub fn attach_surface(
        &mut self,
        face: &Face,
        surface: BSplineSurface,
    ) -> Option<BSplineSurface>
    {
        self.surfaces.insert(face.id(), surface)
    }

    pub fn get_point(&self, vertex: &Vertex) -> Option<&Vector> { self.points.get(&vertex.id()) }

    pub fn get_curve(&mut self, edge: &Edge) -> Option<&BSplineCurve> {
        self.curves.get(&edge.id())
    }

    pub fn get_surface(&mut self, face: &Face) -> Option<&BSplineSurface> {
        self.surfaces.get(&face.id())
    }
}

fn check_integrate_one_face(
    points: &HashMap<usize, Vector>,
    curves: &HashMap<usize, BSplineCurve>,
    face: &Face,
    surface: &BSplineSurface,
) -> GeometryShellIntegrity
{
    let mut boundary = surface.boundary();
    boundary
        .make_locally_projected_injective()
        .optimize()
        .knot_normalize();
    let mut boundary0 = boundary.clone();
    let mut boundary1 = boundary.clone();
    boundary
        .knot_translate(-1.0)
        .concat(&mut boundary0)
        .unwrap()
        .concat(boundary1.knot_translate(1.0))
        .unwrap();
    let mut hint = 0.0;
    for edge in face.boundary().edge_iter() {
        let curve = match curves.get(&edge.id()) {
            Some(curve) => curve,
            None => return GeometryShellIntegrity::NoCurveEdge { edge_id: edge.id() },
        };
        let integrity = check_integrate_one_edge(points, edge, curve);
        if integrity != GeometryShellIntegrity::Integrate {
            return integrity;
        }
        let mut curve = curve.clone();
        if edge.absolute_front() != edge.front() {
            curve.inverse();
        }
        match curve.is_projected_arc_of(&mut boundary, hint) {
            Some(res) => hint = res,
            None => {
                return GeometryShellIntegrity::NotBoundary {
                    face_id: face.id(),
                    edge_id: edge.id(),
                }
            }
        }
    }
    GeometryShellIntegrity::Integrate
}

fn check_integrate_one_edge(
    points: &HashMap<usize, Vector>,
    edge: &Edge,
    curve: &BSplineCurve,
) -> GeometryShellIntegrity
{
    let front_id = edge.absolute_front().id();
    let topo_pt0 = match points.get(&front_id) {
        Some(pt) => pt.projection(),
        None => {
            return GeometryShellIntegrity::NoPointVertex {
                vertex_id: front_id,
            }
        }
    };
    let back_id = edge.absolute_back().id();
    let topo_pt1 = match points.get(&back_id) {
        Some(pt) => pt.projection(),
        None => return GeometryShellIntegrity::NoPointVertex { vertex_id: back_id },
    };

    let knot_vec = curve.knot_vec();
    let geom_pt0 = curve.subs(knot_vec[0]).projection();
    let geom_pt1 = curve.subs(knot_vec[knot_vec.len() - 1]).projection();

    if !topo_pt0.near(&geom_pt0) {
        GeometryShellIntegrity::NotEndPoint {
            edge_id: edge.id(),
            vertex_id: front_id,
        }
    } else if !topo_pt1.near(&geom_pt1) {
        GeometryShellIntegrity::NotEndPoint {
            edge_id: edge.id(),
            vertex_id: back_id,
        }
    } else {
        GeometryShellIntegrity::Integrate
    }
}
