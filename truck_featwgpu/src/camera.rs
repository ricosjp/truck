use crate::*;

impl Camera {
    #[inline(always)]
    pub fn position(&self) -> Point3 {
        Point3::new(self.matrix[3][0], self.matrix[3][1], self.matrix[3][2])
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
        near_clip: f64,
        far_clip: f64,
    ) -> Camera
    {
        let projection = cgmath::perspective(cgmath::Rad(field_of_view), 1.0, near_clip, far_clip);
        Camera {
            matrix,
            projection,
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
        let a = screen_size / 2.0;
        let ortho = cgmath::ortho(-a, a, -a, a, near_clip, far_clip);
        Camera {
            matrix: matrix,
            projection: ortho,
            projection_type: ProjectionType::Parallel,
        }
    }

    pub fn projection(&self, as_rat: f64) -> Matrix4 {
        Matrix4::from_nonuniform_scale(1.0 / as_rat, 1.0, 1.0)
            * self.projection
            * self.matrix.invert().unwrap()
    }

    pub fn buffer(&self, as_rat: f64, device: &Device) -> BufferHandler {
        let camera_info = CameraInfo {
            camera_matrix: (&self.matrix).cast().unwrap().into(),
            camera_projection: self.projection(as_rat).cast().unwrap().into(),
        };
        let buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(&[camera_info]), BufferUsage::UNIFORM);
        BufferHandler::new(buffer, std::mem::size_of::<CameraInfo>() as u64)
    }
}

impl Default for Camera {
    #[inline(always)]
    fn default() -> Camera {
        Camera::perspective_camera(
            Matrix4::identity(),
            std::f64::consts::PI / 4.0,
            0.1,
            10.0,
        )
    }
}
