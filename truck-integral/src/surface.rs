use crate::*;

impl Surface for BSplineSurface<Vector4> {
    type Curve = BSplineCurve<Vector4>;
    fn inverse(&self) -> Self {
        let mut surface = self.clone();
        surface.swap_axes();
        surface
    }
    fn include(&self, curve: &Self::Curve) -> bool { self.rational_include(curve) }
}
