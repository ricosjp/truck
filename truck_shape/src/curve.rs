use crate::errors::Error;
use crate::shape_geometry::GeometryShellIntegrity;
use crate::{Curve, Geometry, Result};
use geometry::*;
use topology::*;

impl Curve {
    pub fn new(wire: Wire, mut geom: Geometry) -> Curve {
        let integrity = geom.check_wire_integrity(&wire);
        if integrity != GeometryShellIntegrity::Integrate {
            panic!("{}", Error::from(integrity));
        }
        if wire.len() != geom.curves.len() {
            panic!("{}", Error::DifferentNumOfEdgesAndCurves);
        }
        if !wire.is_simple() {
            panic!("{}", Error::WireIsNotSimple);
        }
        if wire.is_closed() && wire.len() + 1 != geom.points.len() {
            panic!("{}", Error::DifferentNumOfVertexAndPoints);
        } else if wire.len() != geom.points.len() {
            panic!("{}", Error::DifferentNumOfVertexAndPoints);
        }
        Curve { wire, geom }
    }
    pub fn try_new(wire: Wire, mut geom: Geometry) -> Result<Curve> {
        let integrity = geom.check_wire_integrity(&wire);
        if integrity != GeometryShellIntegrity::Integrate {
            return Err(integrity.into());
        }
        if wire.len() != geom.curves.len() {
            return Err(Error::DifferentNumOfEdgesAndCurves);
        }
        if !wire.is_simple() {
            return Err(Error::WireIsNotSimple);
        }
        if wire.is_closed() && wire.len() + 1 != geom.points.len() {
            return Err(Error::DifferentNumOfVertexAndPoints);
        } else if wire.len() != geom.points.len() {
            return Err(Error::DifferentNumOfVertexAndPoints);
        }
        Ok(Curve { wire, geom })
    }

    pub fn new_unchecked(wire: Wire, geom: Geometry) -> Curve { Curve { wire, geom } }

    pub fn topology(&self) -> &Wire { &self.wire }
    pub fn geometry(&self) -> &Geometry { &self.geom }

    pub fn concat(&mut self, other: &mut Curve) -> &mut Curve {
        if self.wire.is_empty() {
            self.wire.append(&mut other.wire);
            self.geom = other.geom.clone();
        } else {
            self.wire.append(&mut other.wire);
            if !self.wire.is_simple() {
                panic!("{}", Error::WireIsNotSimple);
            }
            for (id, point) in other.geom.points.iter() {
                self.geom.points.insert(*id, point.clone());
            }
            for (id, curve) in other.geom.curves.iter() {
                self.geom.curves.insert(*id, curve.clone());
            }
        }
        other.geom = Geometry::new();
        self
    }
    pub fn try_concat(&mut self, other: &mut Curve) -> Result<&mut Curve> {
        if self.wire.is_empty() {
            self.wire.try_append(&mut other.wire)?;
            self.geom = other.geom.clone();
        } else {
            self.wire.append(&mut other.wire);
            if !self.wire.is_simple() {
                return Err(Error::WireIsNotSimple);
            }
            for (id, point) in other.geom.points.iter() {
                self.geom.points.insert(*id, point.clone());
            }
            for (id, curve) in other.geom.curves.iter() {
                self.geom.curves.insert(*id, curve.clone());
            }
        }
        other.geom = Geometry::new();
        Ok(self)
    }

    pub fn to_bspcurve(&self) -> BSplineCurve {
        let mut iter = self.wire.edge_iter();
        let edge = iter.next().unwrap();
        let mut curve = self.geom.curves.get(&edge.id()).unwrap().clone();
        curve.knot_normalize();
        for (i, edge) in iter.enumerate() {
            let mut curve0 = self.geom.curves.get(&edge.id()).unwrap().clone();
            curve0.knot_normalize().knot_translate((i + 1) as f64);
            curve.concat(&mut curve0).unwrap();
        }
        curve.knot_normalize();
        curve
    }

    pub fn poly_line<'a, I: Iterator<Item = &'a Vector>>(mut pts: I) -> Curve {
        let mut wire = Wire::new();
        let mut geom = Geometry::new();

        let mut prev_vertex = match pts.next() {
            Some(pt) => {
                let first_vertex = Vertex::new();
                geom.attach_point(&first_vertex, pt.clone());
                first_vertex
            },
            None => panic!("{}", Error::EmptyPointIter),
        };
        for pt in pts {
            let new_vertex = Vertex::new();
            geom.attach_point(&new_vertex, pt.clone());
            let edge = Edge::new(prev_vertex, new_vertex);
            let line = BSplineCurve::new(
                KnotVec::try_from(vec![0.0, 0.0, 1.0, 1.0]).unwrap(),
                vec![
                    geom.get_point(&prev_vertex).unwrap().clone(),
                    geom.get_point(&new_vertex).unwrap().clone(),
                ],
            );
            geom.attach_curve(&edge, line);
            wire.push_back(edge);
            prev_vertex = new_vertex;
        }
        Curve { wire, geom }
    }
}
