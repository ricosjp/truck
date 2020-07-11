use crate::*;

impl Default for Camera {
    fn default() -> Camera {
        Camera {
            matrix: Matrix4::identity(),
            larger_screen_size: 2.0,
            front_clipping_plane: 0.1,
            back_clipping_plane: 1.0,
            projection_type: ProjectionType::Perspective,
        }
    }
}

impl std::ops::MulAssign<&Matrix4> for Camera {
    #[inline(always)]
    fn mul_assign(&mut self, mat: &Matrix4) {
        self.matrix *= mat;
    }
}

impl Camera {
    fn perspective_projection(&self) -> Matrix4 {
        let matrix = &self.matrix;
        let a = self.larger_screen_size;
        let z_min = self.front_clipping_plane;
        let z_max = self.back_clipping_plane;

        matrix.inverse().unwrap() *  Matrix4::new(
            [1.0 / a, 0.0, 0.0, 0.0],
            [0.0, 1.0 / a, 0.0, 0.0],
            [0.0, 0.0, -1.0 / (z_max * (1.0 - z_min)), 0.0],
            [0.0, 0.0, -z_min / (1.0 - z_min), 1.0],
        )
    }

    fn parallel_projection(&self) -> Matrix4 {
        let matrix = &self.matrix;
        let a = self.larger_screen_size;
        let z_min = self.front_clipping_plane;
        let z_max = self.back_clipping_plane;

        matrix.inverse().unwrap() * Matrix4::new(
            [2.0 / a, 0.0, 0.0, 0.0],
            [0.0, 2.0 / a, 0.0, 0.0],
            [0.0, 0.0, 1.0 / (z_max - z_min), 0.0],
            [0.0, 0.0, -z_min / (z_max - z_min), 1.0],
        )
    }

    pub fn projection(&self) -> Matrix4 {
        match self.projection_type {
            ProjectionType::Perspective => self.perspective_projection(),
            ProjectionType::Parallel => self.parallel_projection(),
        }
    }
}
