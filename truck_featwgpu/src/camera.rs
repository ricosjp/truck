use crate::*;

impl Camera {
    #[inline(always)]
    pub fn matrix(&self) -> &Matrix4 { &self.matrix }

    #[inline(always)]
    pub fn matrix_mut(&mut self) -> &mut Matrix4 { &mut self.matrix }

    #[inline(always)]
    pub fn position(&self) -> Vector3 {
        vector!(self.matrix[3][0], self.matrix[3][1], self.matrix[3][2])
    }

    #[inline(always)]
    pub fn eye_direction(&self) -> Vector3 {
        vector!(-self.matrix[2][0], -self.matrix[2][1], -self.matrix[2][2])
    }

    #[inline(always)]
    pub fn projection_type(&self) -> ProjectionType { self.projection_type }

    #[inline(always)]
    pub fn head_direction(&self) -> Vector3 {
        vector!(self.matrix[1][0], self.matrix[1][1], self.matrix[1][2])
    }

    #[inline(always)]
    pub fn perspective_camera(
        matrix: Matrix4,
        field_of_view: f64,
        near_clip: f64,
        far_clip: f64,
    ) -> Camera
    {
        Camera {
            matrix,
            screen_size: (field_of_view / 4.0).tan() * 2.0,
            near_clip,
            far_clip,
            projection_type: ProjectionType::Perspective,
        }
    }

    #[inline(always)]
    pub fn parallel_camera(
        matrix: Matrix4,
        screen_size: f64,
        near_clip: f64,
        far_clip: f64,
    ) -> Camera
    {
        Camera {
            matrix,
            screen_size,
            near_clip,
            far_clip,
            projection_type: ProjectionType::Parallel,
        }
    }

    fn perspective_projection(&self, as_rat: f64) -> Matrix4 {
        let matrix = &self.matrix;
        let a = self.screen_size / 2.0;
        let z_min = self.near_clip;
        let z_max = self.far_clip;
        let d = z_min / z_max;

        matrix.inverse()
            * matrix!(
                [1.0 / (a * z_max) * as_rat, 0.0, 0.0, 0.0],
                [0.0, 1.0 / (a * z_max), 0.0, 0.0],
                [0.0, 0.0, -1.0 / (z_max * (1.0 - d)), -1.0 / z_max],
                [0.0, 0.0, -d / (1.0 - d), 0.0],
            )
    }

    fn parallel_projection(&self, as_rat: f64) -> Matrix4 {
        let matrix = &self.matrix;
        let a = self.screen_size;
        let z_min = self.near_clip;
        let z_max = self.far_clip;

        matrix.inverse()
            * matrix!(
                [2.0 / a * as_rat, 0.0, 0.0, 0.0],
                [0.0, 2.0 / a, 0.0, 0.0],
                [0.0, 0.0, -1.0 / (z_max - z_min), 0.0],
                [0.0, 0.0, -z_min / (z_max - z_min), 1.0],
            )
    }

    pub fn projection(&self, as_rat: f64) -> Matrix4 {
        match self.projection_type {
            ProjectionType::Perspective => self.perspective_projection(as_rat),
            ProjectionType::Parallel => self.parallel_projection(as_rat),
        }
    }

    pub fn buffer(&self, as_rat: f64, device: &Device) -> BufferHandler {
        let camera_info = CameraInfo {
            camera_matrix: (&self.matrix).into(),
            camera_projection: self.projection(as_rat).into(),
        };
        let buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[camera_info]), BufferUsage::UNIFORM,
        );
        BufferHandler::new(buffer, std::mem::size_of::<CameraInfo>() as u64)
    }
}

impl std::ops::MulAssign<&Matrix4> for Camera {
    #[inline(always)]
    fn mul_assign(&mut self, mat: &Matrix4) { self.matrix *= mat; }
}

impl std::ops::MulAssign<Matrix4> for Camera {
    #[inline(always)]
    fn mul_assign(&mut self, mat: Matrix4) { self.matrix *= mat; }
}

impl Default for Camera {
    #[inline(always)]
    fn default() -> Camera {
        Camera {
            matrix: Matrix4::identity(),
            screen_size: 1.0,
            near_clip: 0.1,
            far_clip: 1000.0,
            projection_type: ProjectionType::Perspective,
        }
    }
}
