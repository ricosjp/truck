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
    pub fn projection_type(&self) -> Projection { self.projection }

    #[inline(always)]
    pub fn head_direction(&self) -> Vector3 {
        Vector3::new(self.matrix[1][0], self.matrix[1][1], self.matrix[1][2])
    }

    #[inline(always)]
    pub fn perspective_camera(
        eye: Point3,
        center: Point3,
        up: Vector3,
        field_of_view: f64,
        near_clip: f64,
        far_clip: f64,
    ) -> Camera
    {
        let per_fov = cgmath::PerspectiveFov {
            fovy: cgmath::Rad(field_of_view),
            aspect: 1.0,
            near: near_clip,
            far: far_clip,
        };
        Camera {
            matrix: Matrix4::look_at(eye, center, up).invert().unwrap(),
            projection: Projection::Perspective(per_fov),
        }
    }

    #[inline(always)]
    pub fn parallel_camera(
        eye: Point3,
        center: Point3,
        up: Vector3,
        screen_size: f64,
        near_clip: f64,
        far_clip: f64,
    ) -> Camera
    {
        let ortho = cgmath::Ortho {
            left: -screen_size / 2.0,
            right: screen_size / 2.0,
            bottom: -screen_size / 2.0,
            top: screen_size / 2.0,
            near: near_clip,
            far: far_clip,
        };
        Camera {
            matrix: Matrix4::look_at(eye, center, up).invert().unwrap(),
            projection: Projection::Parallel(ortho),
        }
    }

    pub fn projection(&self, as_rat: f64) -> Matrix4 {
        match self.projection {
            Projection::Perspective(mut per_fov) => {
                per_fov.aspect = 1.0 / as_rat;
                Matrix4::from(per_fov) * self.matrix.invert().unwrap()
            },
            Projection::Parallel(mut ortho) => {
                ortho.top /= as_rat;
                ortho.bottom /= as_rat;
                Matrix4::from(ortho) * self.matrix.invert().unwrap()
            },
        }
    }

    pub fn buffer(&self, as_rat: f64, device: &Device) -> BufferHandler {
        let camera_info = CameraInfo {
            camera_matrix: (&self.matrix).cast().unwrap().into(),
            camera_projection: self.projection(as_rat).cast().unwrap().into(),
        };
        let buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[camera_info]), BufferUsage::UNIFORM,
        );
        BufferHandler::new(buffer, std::mem::size_of::<CameraInfo>() as u64)
    }
}

impl Default for Camera {
    #[inline(always)]
    fn default() -> Camera {
        Camera::perspective_camera(
            Point3::new(0.0, 0.0, 1.0),
            Point3::origin(),
            Vector3::unit_y(),
            std::f64::consts::PI / 4.0,
            0.1,
            10.0,
        )
    }
}
