use truck_geometry::*;
use truck_topology::*;
use crate::{Geometry, Result};
use crate::errors::Error;
use std::collections::HashMap;

pub type PointIter<'a> = std::collections::hash_map::Values<'a, usize, Vector>;
pub type CurveIter<'a> = std::collections::hash_map::Values<'a, usize, BSplineCurve>;
pub type SurfaceIter<'a> = std::collections::hash_map::Values<'a, usize, BSplineSurface>;
pub type PointIterMut<'a> = std::collections::hash_map::ValuesMut<'a, usize, Vector>;
pub type CurveIterMut<'a> = std::collections::hash_map::ValuesMut<'a, usize, BSplineCurve>;
pub type SurfaceIterMut<'a> = std::collections::hash_map::ValuesMut<'a, usize, BSplineSurface>;

impl Geometry {
    pub fn new() -> Geometry {
        Geometry {
            surfaces: HashMap::new(),
            curves: HashMap::new(),
            points: HashMap::new(),
        }
    }

    pub fn points(&self) -> PointIter { self.points.values() }
    pub fn curves(&self) -> CurveIter { self.curves.values() }
    pub fn surfaces(&self) -> SurfaceIter { self.surfaces.values() }
    pub fn points_mut(&mut self) -> PointIterMut { self.points.values_mut() }
    pub fn curves_mut(&mut self) -> CurveIterMut { self.curves.values_mut() }
    pub fn surfaces_mut(&mut self) -> SurfaceIterMut { self.surfaces.values_mut() }

    pub fn attach_point(&mut self, vertex: &Vertex, point: Vector) -> Result<&mut Self> {
        if let Some(old) = self.points.get(&vertex.id()) {
            let x = point.projection();
            let y = old.projection();
            if f64::near(&x[0], &y[0]) && f64::near(&x[1], &y[1]) && f64::near(&x[2], &y[2]) {
                Ok(self)
            } else {
                Err(Error::ConflictPoints(vertex.id(), old.clone(), point))
            }
        } else {
            self.points.insert(vertex.id(), point);
            Ok(self)
        }
    }

    pub fn attach_curve(&mut self, edge: &Edge, mut curve: BSplineCurve) -> Result<&mut Self> {
        curve.clamp().optimize();
        self.attach_point(&edge.absolute_front(), curve.control_points().first().unwrap().clone())?;
        self.attach_point(&edge.absolute_back(), curve.control_points().last().unwrap().clone())?;
        self.curves.insert(edge.id(), curve);
        Ok(self)
    }

    pub fn attach_surface(&mut self, face: &Face, surface: BSplineSurface) -> Result<&mut Self> {
        let mut boundary = surface.boundary();
        boundary.make_locally_projected_injective().optimize().knot_normalize();
        boundary.concat(boundary.clone().knot_translate(1.0)).unwrap();
        let mut hint = 0.0;
        for edge in face.boundary().edge_iter() {
            if let Some(curve) = self.curves.get(&edge.id()) {
                let mut curve = curve.clone();
                if edge.absolute_front() != edge.front() {
                    curve.inverse();
                }
                match curve.is_projected_arc_of(&mut boundary, hint) {
                    Some(res) => hint = res,
                    None => return Err(Error::ConflictCurves(boundary.clone(), curve.clone()))
                }
            } else {
                match self.points.get(&edge.back().id()) {
                    Some(point) => {
                        let t = boundary.search_projected_nearest_parameter(&point, hint)?;
                        let mut new_curve = boundary.clone();
                        new_curve = new_curve.cut(t).cut(hint);
                        self.attach_curve(&edge, new_curve)?;
                        hint = t;
                    },
                    None => return Err(Error::CannotDetermineGeometry),
                }
            }
        }
        self.surfaces.insert(face.id(), surface);
        Ok(self)
    }
}
