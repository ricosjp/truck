use crate::{Director, Vector4, BSplineCurve, BSplineSurface};
use crate::errors::Error;
use std::iter::FromIterator;
use geometry::*;
use std::collections::HashMap;
use topology::*;

pub trait TopologicalElement {
    type Geometry;
    fn id(&self) -> usize;
    fn geom_container(director: &Director) -> &HashMap<usize, Self::Geometry>;
    fn geom_mut_container(director: &mut Director) -> &mut HashMap<usize, Self::Geometry>;
    fn no_geometry(&self) -> Error {
        Error::NoGeometry(std::any::type_name::<Self>(), self.id())
    }
}

impl TopologicalElement for Vertex {
    type Geometry = Vector4;
    fn id(&self) -> usize { self.id() }
    fn geom_container(director: &Director) -> &HashMap<usize, Self::Geometry> { &director.points }
    fn geom_mut_container(director: &mut Director) -> &mut HashMap<usize, Self::Geometry> {
        &mut director.points
    }
}

impl TopologicalElement for Edge {
    type Geometry = BSplineCurve;
    fn id(&self) -> usize { self.id() }
    fn geom_container(director: &Director) -> &HashMap<usize, Self::Geometry> { &director.curves }
    fn geom_mut_container(director: &mut Director) -> &mut HashMap<usize, Self::Geometry> {
        &mut director.curves
    }
}

impl TopologicalElement for Face {
    type Geometry = BSplineSurface;
    fn id(&self) -> usize { self.id() }
    fn geom_container(director: &Director) -> &HashMap<usize, Self::Geometry> { &director.surfaces }
    fn geom_mut_container(director: &mut Director) -> &mut HashMap<usize, Self::Geometry> {
        &mut director.surfaces
    }
}

pub trait GeometricalElement: Sized {
    type Topology: TopologicalElement<Geometry = Self>;
    fn create_topology(self, director: &mut Director) -> Self::Topology;
}

impl GeometricalElement for Vector4 {
    type Topology = Vertex;
    fn create_topology(self, director: &mut Director) -> Vertex {
        let vert = Vertex::new();
        director.attach(&vert, self);
        vert
    }
}

impl GeometricalElement for BSplineCurve {
    type Topology = Edge;
    fn create_topology(self, director: &mut Director) -> Edge {
        let (pt0, pt1) = self.end_points();
        let edge = Edge::new_unchecked(
            pt0.create_topology(director),
            pt1.create_topology(director),
        );
        director.attach(&edge, self);
        edge
    }
}

impl GeometricalElement for BSplineSurface {
    type Topology = Face;
    fn create_topology(self, director: &mut Director) -> Face {
        let [curve0, curve1, curve2, curve3] = self.splitted_boundary();
        let edge0 = curve0.create_topology(director);
        let edge2 = curve2.create_topology(director);
        let edge1 = Edge::new_unchecked(edge0.back(), edge2.front());
        director.attach(&edge1, curve1);
        let edge3 = Edge::new_unchecked(edge2.back(), edge0.front());
        director.attach(&edge3, curve3);
        let wire = Wire::from_iter(&[edge0, edge1, edge2, edge3]);
        let face = Face::new_unchecked(wire);
        director.attach(&face, self);
        face
    }
}

/// integrity of geometric information and shell
#[derive(PartialEq, Debug)]
pub enum TopoGeomIntegrity {
    /// Every face, edge, and vertice correspond to a surface, a curve, and a point, respectively.
    /// Moreover, the geometric information is compatible with the topological information.
    Integrate,
    /// The face with id = `face_id` does not correspond to a surface.
    NoGeometryElement { typename: &'static str, id: usize },
    /// The 4th component of Vector4 is not positive.
    NonPositiveWeightedPoint,
    /// The curve which corresponds to the edge with id = `edge_id` is not in the boundary of
    /// the surface which corresponds to the face with id = `face_id`.
    NotBoundary { face_id: usize, edge_id: usize },
    /// The point which corresponds to the vertex with id = `vertex_id` is not the end point of
    /// the curve which corresponds to the edge with id = `edge_id`.
    NotEndPoint { edge_id: usize, vertex_id: usize },
}

pub trait Integrity {
    fn check_integrity(&self, director: &Director) -> TopoGeomIntegrity;
}

macro_rules! return_not_integrate {
    ($integrity: expr) => {
        let integrity = $integrity;
        if integrity != TopoGeomIntegrity::Integrate {
            return integrity;
        }
    };
}

macro_rules! got_or_return_integrity {
    ($director: expr, $elem: expr) => {
        match $director.get_geometry($elem) {
            Some(got) => got,
            None => {
                return TopoGeomIntegrity::NoGeometryElement {
                    typename: crate::get_typename($elem),
                    id: $elem.id(),
                }
            }
        }
    };
}

impl Integrity for Vertex {
    fn check_integrity(&self, director: &Director) -> TopoGeomIntegrity {
        let pt = got_or_return_integrity!(director, self);
        match pt[3] > geometry::TOLERANCE {
            true => TopoGeomIntegrity::Integrate,
            false => TopoGeomIntegrity::NonPositiveWeightedPoint,
        }
    }
}

impl Integrity for Edge {
    fn check_integrity(&self, director: &Director) -> TopoGeomIntegrity {
        let curve = got_or_return_integrity!(director, self);
        let (p0, p1) = curve.end_points();
        let (front, back) = (self.absolute_front(), self.absolute_back());
        return_not_integrate!(front.check_integrity(director));
        return_not_integrate!(back.check_integrity(director));
        let q0 = got_or_return_integrity!(director, &front);
        let q1 = got_or_return_integrity!(director, &back);
        if !p0.rational_projection().near(&q0.rational_projection()) {
            TopoGeomIntegrity::NotEndPoint {
                edge_id: self.id(),
                vertex_id: front.id(),
            }
        } else if !p1.rational_projection().near(&q1.rational_projection()) {
            TopoGeomIntegrity::NotEndPoint {
                edge_id: self.id(),
                vertex_id: back.id(),
            }
        } else {
            TopoGeomIntegrity::Integrate
        }
    }
}

impl Integrity for Wire {
    fn check_integrity(&self, director: &Director) -> TopoGeomIntegrity {
        for elem in self.edge_iter() {
            return_not_integrate!(elem.check_integrity(director));
        }
        TopoGeomIntegrity::Integrate
    }
}

impl Integrity for Face {
    fn check_integrity(&self, director: &Director) -> TopoGeomIntegrity {
        let surface = got_or_return_integrity!(director, self);
        let mut boundary = surface.boundary();
        boundary
            .make_rational_locally_injective()
            .optimize()
            .knot_normalize();
        let mut boundary0 = boundary.clone();
        let mut boundary1 = boundary.clone();
        boundary
            .knot_translate(-1.0)
            .concat(&mut boundary0)
            .concat(boundary1.knot_translate(1.0));
        let mut hint = 0.0;
        for edge in self.boundary_iter() {
            return_not_integrate!(edge.check_integrity(director));
            let mut curve = got_or_return_integrity!(director, &edge).clone();
            if edge.absolute_front() != edge.front() {
                curve.invert();
            }
            match curve.is_rational_arc_of(&mut boundary, hint) {
                Some(res) => hint = res,
                None => {
                    return TopoGeomIntegrity::NotBoundary {
                        face_id: self.id(),
                        edge_id: edge.id(),
                    }
                }
            }
        }
        TopoGeomIntegrity::Integrate
    }
}

impl Integrity for Shell {
    fn check_integrity(&self, director: &Director) -> TopoGeomIntegrity {
        for elem in self.face_iter() {
            return_not_integrate!(elem.check_integrity(director));
        }
        TopoGeomIntegrity::Integrate
    }
}

impl Integrity for Solid {
    fn check_integrity(&self, director: &Director) -> TopoGeomIntegrity {
        for elem in self.boundaries() {
            return_not_integrate!(elem.check_integrity(director));
        }
        TopoGeomIntegrity::Integrate
    }
}

