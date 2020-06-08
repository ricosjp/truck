use crate::director::TopoGeomIntegrity;
use crate::Director;
use geometry::*;
use std::collections::HashMap;
use topology::*;

pub trait TopologicalElement {
    type Geometry;
    fn id(&self) -> usize;
    fn geom_container(director: &Director) -> &HashMap<usize, Self::Geometry>;
    fn geom_mut_container(director: &mut Director) -> &mut HashMap<usize, Self::Geometry>;
    fn create_by_geometry(geom: Self::Geometry, director: &mut Director) -> Self;
}

impl TopologicalElement for Vertex {
    type Geometry = Vector;
    fn id(&self) -> usize { self.id() }
    fn geom_container(director: &Director) -> &HashMap<usize, Self::Geometry> { &director.points }
    fn geom_mut_container(director: &mut Director) -> &mut HashMap<usize, Self::Geometry> {
        &mut director.points
    }
    fn create_by_geometry(point: Vector, director: &mut Director) -> Vertex {
        let vert = Vertex::new();
        director.insert(&vert, point);
        vert
    }
}

impl TopologicalElement for Edge {
    type Geometry = BSplineCurve;
    fn id(&self) -> usize { self.id() }
    fn geom_container(director: &Director) -> &HashMap<usize, Self::Geometry> { &director.curves }
    fn geom_mut_container(director: &mut Director) -> &mut HashMap<usize, Self::Geometry> {
        &mut director.curves
    }
    fn create_by_geometry(curve: BSplineCurve, director: &mut Director) -> Edge {
        let (pt0, pt1) = curve.end_points();
        let edge = Edge::new_unchecked(
            Vertex::create_by_geometry(pt0, director),
            Vertex::create_by_geometry(pt1, director),
        );
        director.insert(&edge, curve);
        edge
    }
}

impl TopologicalElement for Face {
    type Geometry = BSplineSurface;
    fn id(&self) -> usize { self.id() }
    fn geom_container(director: &Director) -> &HashMap<usize, Self::Geometry> { &director.surfaces }
    fn geom_mut_container(director: &mut Director) -> &mut HashMap<usize, Self::Geometry> {
        &mut director.surfaces
    }
    fn create_by_geometry(surface: BSplineSurface, director: &mut Director) -> Face {
        let [curve0, curve1, curve2, curve3] = surface.splitted_boundary();
        let edge0 = Edge::create_by_geometry(curve0, director);
        let edge2 = Edge::create_by_geometry(curve2, director);
        let edge1 = Edge::new_unchecked(edge0.back(), edge2.front());
        director.insert(&edge1, curve1);
        let edge3 = Edge::new_unchecked(edge2.back(), edge0.front());
        director.insert(&edge3, curve3);
        let wire = Wire::by_slice(&[edge0, edge1, edge2, edge3]);
        let face = Face::new_unchecked(wire);
        director.insert(&face, surface);
        face
    }
}

pub trait GeometricalElement: Sized {
    type Topology: TopologicalElement<Geometry = Self>;
}

impl GeometricalElement for Vector {
    type Topology = Vertex;
}

impl GeometricalElement for BSplineCurve {
    type Topology = Edge;
}

impl GeometricalElement for BSplineSurface {
    type Topology = Face;
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
            Ok(got) => got,
            Err(_) => {
                return TopoGeomIntegrity::NoGeometryElement {
                    typename: get_typename($elem),
                    id: $elem.id(),
                }
            }
        }
    };
}

impl Integrity for Vertex {
    fn check_integrity(&self, director: &Director) -> TopoGeomIntegrity {
        let pt = got_or_return_integrity!(director, self);
        match pt[3] > f64::TOLERANCE {
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
        if !p0.projection().near(&q0.projection()) {
            TopoGeomIntegrity::NotEndPoint {
                edge_id: self.id(),
                vertex_id: front.id(),
            }
        } else if !p1.projection().near(&q1.projection()) {
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
        for edge in self.boundary().edge_iter() {
            return_not_integrate!(edge.check_integrity(director));
            let mut curve = got_or_return_integrity!(director, edge).clone();
            if edge.absolute_front() != edge.front() {
                curve.inverse();
            }
            match curve.is_projected_arc_of(&mut boundary, hint) {
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

fn get_typename<T>(_: T) -> &'static str { std::any::type_name::<T>() }

