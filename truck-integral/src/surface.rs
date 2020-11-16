use crate::*;

impl Surface for NURBSSurface<Vector4> {
    type Curve = NURBSCurve<Vector4>;
    fn inverse(&self) -> Self {
        let mut surface = self.clone();
        surface.swap_axes();
        surface
    }
    fn include(&self, curve: &Self::Curve) -> bool { self.include(curve) }
}
