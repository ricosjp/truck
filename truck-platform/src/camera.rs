use crate::*;

impl Camera {
    /// Returns the position of camera,
    /// the forth column of the camera matrix.
    ///
    /// # Examples
    /// ```
    /// use truck_platform::*;
    /// use truck_base::cgmath64::*;
    /// let mut camera = Camera::default();
    /// camera.matrix = Matrix4::from_translation(Vector3::new(1.0, 2.0, 3.0));
    /// assert_eq!(camera.position(), Point3::new(1.0, 2.0, 3.0));
    /// ```
    #[inline(always)]
    pub fn position(&self) -> Point3 { Point3::from_vec(self.matrix[3].truncate()) }

    /// Returns the eye direction of camera.
    /// the inverse of the z-axis of the camera matrix.
    /// 
    /// # Examples
    /// ```
    /// use std::f64::consts::PI;
    /// use truck_platform::*;
    /// use truck_base::{cgmath64::*, tolerance::Tolerance};
    /// let mut camera = Camera::default();
    /// camera.matrix = Matrix4::from_axis_angle(
    ///     Vector3::new(1.0, 1.0, 1.0).normalize(),
    ///     Rad(2.0 * PI / 3.0),
    /// );
    /// assert!(camera.eye_direction().near(&-Vector3::unit_x()));
    /// ```
    #[inline(always)]
    pub fn eye_direction(&self) -> Vector3 { -self.matrix[2].truncate() }

    /// Returns the direction of the head vector, the y-axis of the camera matrix.
    /// # Examples
    /// ```
    /// use std::f64::consts::PI;
    /// use truck_platform::*;
    /// use truck_base::{cgmath64::*, tolerance::Tolerance};
    /// let mut camera = Camera::default();
    /// camera.matrix = Matrix4::from_axis_angle(
    ///     Vector3::new(1.0, 1.0, 1.0).normalize(),
    ///     Rad(2.0 * PI / 3.0),
    /// );
    /// assert!(camera.head_direction().near(&Vector3::unit_z()));
    /// ```
    #[inline(always)]
    pub fn head_direction(&self) -> Vector3 { self.matrix[1].truncate() }

    /// Returns the projection type of the camera.
    /// # Examples
    /// ```
    /// use truck_platform::*;
    /// // the projection type of the default camera is perspective.
    /// assert_eq!(Camera::default().projection_type(), ProjectionType::Perspective);
    /// ```
    #[inline(always)]
    pub fn projection_type(&self) -> ProjectionType { self.projection_type }

    #[inline(always)]
    pub fn perspective_camera<R: Into<Rad<f64>>>(
        matrix: Matrix4,
        field_of_view: R,
        near_clip: f64,
        far_clip: f64,
    ) -> Camera {
        let projection = crate::perspective(field_of_view.into(), 1.0, near_clip, far_clip);
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
    ) -> Camera {
        let a = screen_size / 2.0;
        let projection = crate::ortho(-a, a, -a, a, near_clip, far_clip);
        Camera {
            matrix,
            projection,
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
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[camera_info]),
            usage: BufferUsage::UNIFORM,
            label: None,
        });
        BufferHandler::new(buffer, std::mem::size_of::<CameraInfo>() as u64)
    }
}

impl Default for Camera {
    #[inline(always)]
    fn default() -> Camera {
        Camera::perspective_camera(
            Matrix4::identity(),
            Rad(std::f64::consts::PI / 4.0),
            0.1,
            10.0,
        )
    }
}
