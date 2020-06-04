use crate::errors::Error;
use crate::shape_geometry::GeometryShellIntegrity;
use crate::{Geometry, Volume};
use topology::Solid;

impl Volume {
    pub fn new(solid: Solid, mut geom: Geometry) -> Volume {
        let mut face_counter = 0;
        for shell in solid.boundaries() {
            let integrity = geom.check_shell_integrity(shell);
            if integrity != GeometryShellIntegrity::Integrate {
                panic!("{}", Error::from(integrity));
            }
            face_counter += shell.len();
        }
        if face_counter != geom.surfaces.len() {
            panic!("{}", Error::DifferentNumOfFacesAndSurfaces)
        }
        Volume { solid, geom }
    }

    pub fn try_new(solid: Solid, mut geom: Geometry) -> crate::Result<Volume> {
        let mut face_counter = 0;
        for shell in solid.boundaries() {
            let integrity = geom.check_shell_integrity(shell);
            if integrity != GeometryShellIntegrity::Integrate {
                return Err(integrity.into());
            }
            face_counter += shell.len();
        }
        if face_counter != geom.surfaces.len() {
            return Err(Error::DifferentNumOfFacesAndSurfaces);
        }
        Ok(Volume { solid, geom })
    }

    pub fn new_unchcked(solid: Solid, geom: Geometry) -> Volume { Volume { solid, geom } }

    pub fn topology(&self) -> &Solid { &self.solid }
    pub fn geometry(&self) -> &Geometry { &self.geom }
}

impl std::convert::From<Volume> for (Solid, Geometry) {
    fn from(volume: Volume) -> (Solid, Geometry) { (volume.solid, volume.geom) }
}
