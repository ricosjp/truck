use crate::elements::{Integrity, TopoGeomIntegrity, TopologicalElement};
use crate::errors::Error;
use crate::Result;
use crate::*;

impl Director {
    pub fn new() -> Director { Director::default() }

    #[inline(always)]
    pub fn attach<T>(&mut self, topo: &T, geom: T::Geometry) -> Option<T::Geometry>
    where T: TopologicalElement {
        T::geom_mut_container(self).insert(topo.id(), geom)
    }

    #[inline(always)]
    pub fn get_geometry<T>(&self, topo: &T) -> Option<&T::Geometry>
    where T: TopologicalElement {
        T::geom_container(self).get(&topo.id())
    }
    #[inline(always)]
    pub fn try_get_geometry<T>(&self, topo: &T) -> Result<&T::Geometry>
    where T: TopologicalElement {
        self.get_geometry(topo).ok_or(topo.no_geom_error())
    }

    #[inline(always)]
    pub fn get_mut_geometry<T>(&mut self, topo: &T) -> Option<&mut T::Geometry>
    where T: TopologicalElement {
        T::geom_mut_container(self).get_mut(&topo.id())
    }
    #[inline(always)]
    pub fn try_get_mut_geometry<T>(&mut self, topo: &T) -> Result<&mut T::Geometry>
    where T: TopologicalElement {
        match self.get_mut_geometry(topo) {
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
    pub fn get_builder(&mut self) -> Builder { Builder { director: self } }

    #[inline(always)]
    pub fn building<T, F: FnOnce(&mut Builder) -> T>(&mut self, closure: F) -> T {
        closure(&mut self.get_builder())
    }

    #[inline(always)]
    pub fn get_mesher(&mut self) -> Mesher { Mesher { director: self } }
}
