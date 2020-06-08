use crate::elements::{TopologicalElement, Integrity};
use crate::errors::Error;
use crate::*;
use crate::Result;
use geometry::*;
use topology::*;

/// integrity of geometric information and shell
#[derive(PartialEq, Debug)]
pub enum TopoGeomIntegrity {
    /// Every face, edge, and vertice correspond to a surface, a curve, and a point, respectively.
    /// Moreover, the geometric information is compatible with the topological information.
    Integrate,
    /// The face with id = `face_id` does not correspond to a surface.
    NoGeometryElement { typename: &'static str, id: usize },
    /// The 4th component of Vector is not positive.
    NonPositiveWeightedPoint,
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
    pub fn insert<T>(&mut self, topo: &T, geom: T::Geometry) -> Option<T::Geometry>
    where T: TopologicalElement {
        T::geom_mut_container(self).insert(topo.id(), geom)
    }

    #[inline(always)]
    pub fn get_geometry<T>(&self, topo: &T) -> Result<&T::Geometry>
    where T: TopologicalElement {
        match T::geom_container(self).get(&topo.id()) {
            Some(got) => Ok(got),
            None => Err(Error::NoGeometry(std::any::type_name::<T>(), topo.id())),
        }
    }

    #[inline(always)]
    pub fn get_mut_geometry<T>(&mut self, topo: &T) -> Result<&T::Geometry>
    where T: TopologicalElement {
        match T::geom_mut_container(self).get(&topo.id()) {
            Some(got) => Ok(got),
            None => Err(Error::NoGeometry(std::any::type_name::<T>(), topo.id())),
        }
    }

    #[inline(always)]
    pub fn remove<T>(&mut self, topo: &T) -> Option<T::Geometry>
    where T: TopologicalElement {
        T::geom_mut_container(self).remove(&topo.id())
    }

    #[inline(always)]
    pub fn check_integrity<T: Integrity>(&self, topo: &T) -> TopoGeomIntegrity {
        topo.check_integrity(self)
    }

    #[inline(always)]
    pub fn get_oriented_curve(&self, edge: &Edge) -> Result<BSplineCurve> {
        let mut curve = self.get_geometry(edge)?.clone();
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

    pub fn get_builder(&mut self) -> Builder { Builder { director: self } }

    pub fn building<T, F: FnOnce(&mut Builder) -> T>(&mut self, closure: F) -> T {
        closure(&mut self.get_builder())
    }

    pub fn get_mesher(&mut self) -> Mesher { Mesher { director: self } }
}