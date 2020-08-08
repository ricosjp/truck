use crate::*;

impl Camera {
    #[inline(always)]
    pub fn matrix(&self) -> &Matrix4 { &self.matrix }

    #[inline(always)]
    pub fn matrix_mut(&mut self) -> &mut Matrix4 { &mut self.matrix }

    #[inline(always)]
    pub fn position(&self) -> Vector3 {
        Vector3::new(self.matrix[3][0], self.matrix[3][1], self.matrix[3][2])
    }

    #[inline(always)]
    pub fn eye_direction(&self) -> Vector3 {
        Vector3::new(-self.matrix[2][0], -self.matrix[2][1], -self.matrix[2][2])
    }

    #[inline(always)]
    pub fn projection_type(&self) -> ProjectionType { self.projection_type }

    #[inline(always)]
    pub fn head_direction(&self) -> Vector3 {
        Vector3::new(self.matrix[1][0], self.matrix[1][1], self.matrix[1][2])
    }

    #[inline(always)]
    pub fn perspective_camera(
        matrix: Matrix4,
        field_of_view: f64,
        front_clipping_plane: f64,
        back_clipping_plane: f64,
    ) -> Camera
    {
        Camera {
            matrix,
            screen_size: (field_of_view / 4.0).tan() * 2.0,
            front_clipping_plane,
            back_clipping_plane,
            projection_type: ProjectionType::Perspective,
        }
    }

    #[inline(always)]
    pub fn parallel_camera(
        matrix: Matrix4,
        screen_size: f64,
        front_clipping_plane: f64,
        back_clipping_plane: f64,
    ) -> Camera
    {
        Camera {
            matrix,
            screen_size,
            front_clipping_plane,
            back_clipping_plane,
            projection_type: ProjectionType::Parallel,
        }
    }

    fn perspective_projection(&self) -> Matrix4 {
        let matrix = &self.matrix;
        let a = self.screen_size / 2.0;
        let z_min = self.front_clipping_plane;
        let z_max = self.back_clipping_plane;
        let d = z_min / z_max;

        Matrix4::from_cols(
            Vector4::new(1.0 / (a * z_max), 0.0, 0.0, 0.0),
            Vector4::new(0.0, 1.0 / (a * z_max), 0.0, 0.0),
            Vector4::new(0.0, 0.0, -1.0 / (z_max * (1.0 - d)), -1.0 / z_max),
            Vector4::new(0.0, 0.0, -d / (1.0 - d), 0.0),
        ) * matrix.invert().unwrap()
    }

    fn parallel_projection(&self) -> Matrix4 {
        let matrix = &self.matrix;
        let a = self.screen_size;
        let z_min = self.front_clipping_plane;
        let z_max = self.back_clipping_plane;

        Matrix4::from_cols(
                Vector4::new(2.0 / a, 0.0, 0.0, 0.0),
                Vector4::new(0.0, 2.0 / a, 0.0, 0.0),
                Vector4::new(0.0, 0.0, -1.0 / (z_max - z_min), 0.0),
                Vector4::new(0.0, 0.0, -z_min / (z_max - z_min), 1.0),
            )
        * matrix.invert().unwrap()
    }

    pub fn projection(&self) -> Matrix4 {
        match self.projection_type {
            ProjectionType::Perspective => self.perspective_projection(),
            ProjectionType::Parallel => self.parallel_projection(),
        }
    }
}

impl std::ops::MulAssign<&Matrix4> for Camera {
    #[inline(always)]
    fn mul_assign(&mut self, mat: &Matrix4) { self.matrix = mat * self.matrix; }
}

impl std::ops::MulAssign<Matrix4> for Camera {
    #[inline(always)]
    fn mul_assign(&mut self, mat: Matrix4) { self.matrix = mat * self.matrix; }
}

impl Default for Camera {
    #[inline(always)]
    fn default() -> Camera {
        Camera {
            matrix: Matrix4::identity(),
            screen_size: 1.0,
            front_clipping_plane: 0.1,
            back_clipping_plane: 1000.0,
            projection_type: ProjectionType::Perspective,
        }
    }
}
