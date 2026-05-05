use super::*;
use truck_base::cgmath_extend_traits::control_point::ControlPoint;

impl<T, N> Offset<T, N> {
    /// constructor
    #[inline(always)]
    pub const fn new(entity: T, offset: N) -> Self { Self { entity, offset } }
    /// Returns entity geometry
    #[inline(always)]
    pub const fn entity(&self) -> &T { &self.entity }
    /// Returns offset
    #[inline(always)]
    pub const fn offset(&self) -> &N { &self.offset }
}

impl<T, F> NormalField<T, F> {
    /// constructor
    #[inline(always)]
    pub fn new(entity: T, scalar: F) -> Self { Self { entity, scalar } }
}

mod curve;
mod surface;
