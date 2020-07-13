use crate::*;

impl PolygonMesh {
    pub fn bounding_box(&self) -> [(f64, f64); 3] {
        let mut x = (std::f64::MAX, std::f64::MIN);
        let mut y = (std::f64::MAX, std::f64::MIN);
        let mut z = (std::f64::MAX, std::f64::MIN);
        for pos in &self.positions {
            if pos[0] < x.0 {
                x.0 = pos[0];
            }
            if pos[0] > x.1 {
                x.1 = pos[0];
            }
            if pos[1] < y.0 {
                y.0 = pos[1];
            }
            if pos[1] > y.1 {
                y.1 = pos[1];
            }
            if pos[2] < z.0 {
                z.0 = pos[2];
            }
            if pos[2] > z.1 {
                z.1 = pos[2];
            }
        }
        [x, y, z]
    }
}
