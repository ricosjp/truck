use crate::errors::Error;
use crate::shape_geometry::GeometryShellIntegrity;
use crate::{Curve, Geometry, Surface};
use geometry::BSplineSurface;
use topology::*;

impl Surface {
    pub fn new(shell: Shell, mut geom: Geometry) -> Surface {
        let integrity = geom.check_shell_integrity(&shell);
        if integrity != GeometryShellIntegrity::Integrate {
            panic!("{}", Error::from(integrity));
        }
        if shell.len() != geom.surfaces.len() {
            panic!("{}", Error::DifferentNumOfFacesAndSurfaces);
        }
        Surface { shell, geom }
    }

    pub fn try_new(shell: Shell, mut geom: Geometry) -> crate::Result<Surface> {
        let integrity = geom.check_shell_integrity(&shell);
        if integrity != GeometryShellIntegrity::Integrate {
            return Err(integrity.into());
        }
        if shell.len() != geom.surfaces.len() {
            return Err(Error::DifferentNumOfFacesAndSurfaces);
        }
        Ok(Surface { shell, geom })
    }

    pub fn new_unchecked(shell: Shell, geom: Geometry) -> Surface { Surface { shell, geom } }

    pub fn topology(&self) -> &Shell { &self.shell }
    pub fn geometry(&self) -> &Geometry { &self.geom }

    pub fn homotopy(curve0: &Curve, curve1: &Curve) {
        // create surface
        let bspcurve0 = curve0.to_bspcurve();
        let bspcurve1 = curve1.to_bspcurve();
        let surface = BSplineSurface::homotopy(&bspcurve0, &bspcurve1);
        
        // create face
        let mut wire = curve0.wire.clone();
        let mut wire0 = curve1.wire.clone();
        wire0.inverse();
        let edge0 = Edge::new(
            wire.front_vertex().unwrap(),
            wire0.back_vertex().unwrap(),
        );
        let edge1 = Edge::new(
            wire.back_vertex().unwrap(),
            wire0.front_vertex().unwrap(),
        );
        wire.push_back(edge0);
        wire.append(&mut wire0);
        wire.push_back(edge1);

        // create a line corresponding to `edge0`.
        let pt0 = bspcurve0.subs(0.0);
        let pt1 = bspcurve1.subs(0.0);
    }
}

impl std::convert::From<Surface> for (Shell, Geometry) {
    fn from(surface: Surface) -> (Shell, Geometry) { (surface.shell, surface.geom) }
}
