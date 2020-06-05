use crate::errors::Error;
use crate::{Director, Result};
use geometry::*;
use std::collections::HashMap;
use topology::*;

/// integrity of geometric information and shell
#[derive(PartialEq, Debug)]
pub enum TopoGeomIntegrity {
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

/// basic methods
impl Director {
    pub fn new() -> Director { Director::default() }

    #[inline(always)]
    pub fn insert_point(&mut self, vertex: &Vertex, point: Vector) -> Option<Vector> {
        self.points.insert(vertex.id(), point)
    }

    #[inline(always)]
    pub fn insert_curve(&mut self, edge: &Edge, curve: BSplineCurve) -> Option<BSplineCurve> {
        self.curves.insert(edge.id(), curve)
    }

    #[inline(always)]
    pub fn insert_surface(
        &mut self,
        face: &Face,
        surface: BSplineSurface,
    ) -> Option<BSplineSurface>
    {
        self.surfaces.insert(face.id(), surface)
    }

    #[inline(always)]
    pub fn get_point(&self, vertex: &Vertex) -> Result<&Vector> {
        match self.points.get(&vertex.id()) {
            Some(got) => Ok(got),
            None => Err(Error::NoPointVertex(vertex.id())),
        }
    }

    #[inline(always)]
    pub fn get_curve(&self, edge: &Edge) -> Result<&BSplineCurve> {
        match self.curves.get(&edge.id()) {
            Some(got) => Ok(got),
            None => Err(Error::NoCurveEdge(edge.id())),
        }
    }

    #[inline(always)]
    pub fn get_surface(&self, face: &Face) -> Result<&BSplineSurface> {
        match self.surfaces.get(&face.id()) {
            Some(got) => Ok(got),
            None => Err(Error::NoSurfaceFace(face.id())),
        }
    }

    #[inline(always)]
    pub fn create_vertex(&mut self, point: Vector) -> Vertex {
        let vert = Vertex::new();
        self.insert_point(&vert, point);
        vert
    }

    #[inline(always)]
    pub fn create_edge(
        &mut self,
        vertex0: Vertex,
        vertex1: Vertex,
        curve: BSplineCurve,
    ) -> Result<Edge>
    {
        let edge = Edge::try_new(vertex0, vertex1)?;
        self.insert_curve(&edge, curve);
        Ok(edge)
    }

    #[inline(always)]
    pub fn create_face(&mut self, wire: Wire, surface: BSplineSurface) -> Result<Face> {
        let face = Face::try_new(wire)?;
        self.insert_surface(&face, surface);
        Ok(face)
    }

    #[inline(always)]
    pub fn remove_point(&mut self, vertex: Vertex) -> Option<Vector> {
        self.points.remove(&vertex.id())
    }

    #[inline(always)]
    pub fn remove_curve(&mut self, edge: Edge) -> Option<BSplineCurve> {
        self.curves.remove(&edge.id())
    }

    #[inline(always)]
    pub fn remove_surface(&mut self, face: Face) -> Option<BSplineSurface> {
        self.surfaces.remove(&face.id())
    }

    #[inline(always)]
    pub fn get_oriented_curve(&self, edge: &Edge) -> Result<BSplineCurve> {
        let mut curve = self.get_curve(edge)?.clone();
        if edge.front() != edge.absolute_front() {
            curve.inverse();
        }
        Ok(curve)
    }

    pub fn bspline_by_wire(&self, wire: &Wire) -> Result<BSplineCurve> {
        let mut iter = wire.edge_iter();
        let mut curve = self.get_oriented_curve(iter.next().unwrap())?;
        curve.knot_normalize();
        for (i, edge) in iter.enumerate() {
            let mut tmp_curve = self.get_oriented_curve(edge)?;
            let pt0 = curve.control_points().last().unwrap();
            let pt1 = tmp_curve.control_point(0);
            if !pt0[3].near(&pt1[3]) {
                let scalar = pt0[3] / pt1[3];
                tmp_curve *= scalar;
            }
            tmp_curve.knot_normalize().knot_translate((i + 1) as f64);
            curve.concat(&mut tmp_curve)?;
        }
        Ok(curve)
    }

    pub fn check_solid_integrity(&mut self, solid: &Solid) -> TopoGeomIntegrity {
        for shell in solid.boundaries() {
            let integrity = self.check_shell_integrity(shell);
            if integrity != TopoGeomIntegrity::Integrate {
                return integrity;
            }
        }
        TopoGeomIntegrity::Integrate
    }

    pub fn check_shell_integrity(&mut self, shell: &Shell) -> TopoGeomIntegrity {
        for face in shell.face_iter() {
            let surface = match self.surfaces.get(&face.id()) {
                Some(got) => got,
                None => return TopoGeomIntegrity::NoSurfaceFace { face_id: face.id() },
            };
            let integrity = check_integrate_one_face(&self.points, &self.curves, face, surface);
            if integrity != TopoGeomIntegrity::Integrate {
                return integrity;
            }
        }
        TopoGeomIntegrity::Integrate
    }

    pub fn check_wire_integrity(&mut self, wire: &Wire) -> TopoGeomIntegrity {
        for edge in wire.edge_iter() {
            let curve = match self.curves.get(&edge.id()) {
                Some(got) => got,
                None => return TopoGeomIntegrity::NoCurveEdge { edge_id: edge.id() },
            };
            let integrity = check_integrate_one_edge(&self.points, edge, curve);
            if integrity != TopoGeomIntegrity::Integrate {
                return integrity;
            }
        }
        TopoGeomIntegrity::Integrate
    }
}

fn check_integrate_one_face(
    points: &HashMap<usize, Vector>,
    curves: &HashMap<usize, BSplineCurve>,
    face: &Face,
    surface: &BSplineSurface,
) -> TopoGeomIntegrity
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
            None => return TopoGeomIntegrity::NoCurveEdge { edge_id: edge.id() },
        };
        let integrity = check_integrate_one_edge(points, edge, curve);
        if integrity != TopoGeomIntegrity::Integrate {
            return integrity;
        }
        let mut curve = curve.clone();
        if edge.absolute_front() != edge.front() {
            curve.inverse();
        }
        match curve.is_projected_arc_of(&mut boundary, hint) {
            Some(res) => hint = res,
            None => {
                return TopoGeomIntegrity::NotBoundary {
                    face_id: face.id(),
                    edge_id: edge.id(),
                }
            }
        }
    }
    TopoGeomIntegrity::Integrate
}

fn check_integrate_one_edge(
    points: &HashMap<usize, Vector>,
    edge: &Edge,
    curve: &BSplineCurve,
) -> TopoGeomIntegrity
{
    let front_id = edge.absolute_front().id();
    let topo_pt0 = match points.get(&front_id) {
        Some(pt) => pt.projection(),
        None => {
            return TopoGeomIntegrity::NoPointVertex {
                vertex_id: front_id,
            }
        }
    };
    let back_id = edge.absolute_back().id();
    let topo_pt1 = match points.get(&back_id) {
        Some(pt) => pt.projection(),
        None => return TopoGeomIntegrity::NoPointVertex { vertex_id: back_id },
    };

    let knot_vec = curve.knot_vec();
    let geom_pt0 = curve.subs(knot_vec[0]).projection();
    let geom_pt1 = curve.subs(knot_vec[knot_vec.len() - 1]).projection();

    if !topo_pt0.near(&geom_pt0) {
        TopoGeomIntegrity::NotEndPoint {
            edge_id: edge.id(),
            vertex_id: front_id,
        }
    } else if !topo_pt1.near(&geom_pt1) {
        TopoGeomIntegrity::NotEndPoint {
            edge_id: edge.id(),
            vertex_id: back_id,
        }
    } else {
        TopoGeomIntegrity::Integrate
    }
}
