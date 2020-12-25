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

    /// Creates a perspective camera.
    /// # Arguments
    /// * `matrix`:  camera matrix
    /// * `field_of_view`: FOV, based on the vertical direction of the screen.
    /// * `near_clip`: distance to the nearest face of the view volume
    /// * `far_clip`: distance to the farthest face of the view volume
    /// # Examples
    /// ```
    /// use std::f64::consts::PI;
    /// use truck_base::{cgmath64::*, tolerance::Tolerance};
    /// use truck_platform::*;
    /// let matrix = Matrix4::look_at(
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::origin(),
    ///     Vector3::new(0.0, 1.0, 0.0),
    /// );
    /// let camera = Camera::perspective_camera(
    ///     // depends on the difference of the style with cgmath,
    ///     // the matrix must be inverted
    ///     matrix.invert().unwrap(),
    ///     Rad(PI / 4.0),
    ///     0.1,
    ///     1.0,
    /// );
    /// assert!(camera.eye_direction().near(&-Vector3::new(1.0, 1.0, 1.0).normalize()));
    /// assert_eq!(camera.projection_type(), ProjectionType::Perspective);
    /// ```
    #[inline(always)]
    pub fn perspective_camera<R: Into<Rad<f64>>>(
        matrix: Matrix4,
        field_of_view: R,
        near_clip: f64,
        far_clip: f64,
    ) -> Camera {
        let projection = perspective(field_of_view.into(), 1.0, near_clip, far_clip);
        Camera {
            matrix,
            projection,
            projection_type: ProjectionType::Perspective,
        }
    }

    /// Creates a parallel camera.
    /// # Arguments
    /// * `matrix`:  camera matrix
    /// * `screen_size`: screen size, based on the vertical direction of the screen.`
    /// * `near_clip`: distance to the nearest face of the view volume
    /// * `far_clip`: distance to the farthest face of the view volume
    /// # Examples
    /// ```
    /// use truck_base::{cgmath64::*, tolerance::Tolerance};
    /// use truck_platform::*;
    /// let matrix = Matrix4::look_at(
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::origin(),
    ///     Vector3::new(0.0, 1.0, 0.0),
    /// );
    /// let camera = Camera::parallel_camera(
    ///     // depends on the difference of the style with cgmath,
    ///     // the matrix must be inverted
    ///     matrix.invert().unwrap(),
    ///     1.0,
    ///     0.1,
    ///     1.0,
    /// );
    /// assert!(camera.head_direction().near(&Vector3::new(-0.5, 1.0, -0.5).normalize()));
    /// assert_eq!(camera.projection_type(), ProjectionType::Parallel);
    /// ```
    #[inline(always)]
    pub fn parallel_camera(
        matrix: Matrix4,
        screen_size: f64,
        near_clip: f64,
        far_clip: f64,
    ) -> Camera {
        let a = screen_size / 2.0;
        let projection = Matrix4::new(
            1.0 / a, 0.0, 0.0, 0.0,
            0.0, 1.0 / a, 0.0, 0.0,
            0.0, 0.0, -1.0 / (far_clip - near_clip), 0.0,
            0.0, 0.0, -near_clip / (far_clip - near_clip), 1.0,
        );
        Camera {
            matrix,
            projection,
            projection_type: ProjectionType::Parallel,
        }
    }

    /// Returns the projection matrix into the normalized view volume.
    /// # Arguments
    /// `as_rat`: the aspect ratio, x-resolution / y-resulution.
    /// # Examples
    /// ```
    /// // perspective camera
    /// use std::f64::consts::PI;
    /// use truck_base::{cgmath64::*, tolerance::*};
    /// use truck_platform::*;
    /// 
    /// let fov = PI / 4.0;
    /// let as_rat = 1.2;
    /// let matrix = Matrix4::look_at(
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::origin(),
    ///     Vector3::new(0.0, 1.0, 0.0),
    /// );
    /// let camera = Camera::perspective_camera(
    ///     matrix.invert().unwrap(),
    ///     Rad(fov),
    ///     0.1,
    ///     10.0,
    /// );
    /// 
    /// // calculation by the ray-tracing
    /// let pt = Point3::new(-1.5, -1.4, -2.5);
    /// let vec = pt - camera.position();
    /// let far = 1.0 / (fov / 2.0).tan();
    /// let dir = camera.eye_direction();
    /// let y_axis = camera.head_direction();
    /// let x_axis = dir.cross(y_axis);
    /// let proj_length = dir.dot(vec);
    /// let h = (vec - proj_length * dir) * far / proj_length;
    /// let u = h.dot(x_axis) / as_rat;
    /// let v = h.dot(y_axis);
    /// 
    /// // check the answer
    /// let uv = camera.projection(as_rat).transform_point(pt);
    /// assert!(f64::near(&u, &uv[0]), "{} {}", u, uv[0]);
    /// assert!(f64::near(&v, &uv[1]));
    /// ```
    /// ```
    /// // parallel camera
    /// use truck_base::{cgmath64::*, tolerance::*};
    /// use truck_platform::*;
    /// 
    /// let size = 3.0;
    /// let as_rat = 1.2;
    /// let matrix = Matrix4::look_at(
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::origin(),
    ///     Vector3::new(0.0, 1.0, 0.0),
    /// );
    /// let camera = Camera::parallel_camera(
    ///     matrix.invert().unwrap(),
    ///     size,
    ///     0.1,
    ///     10.0,
    /// );
    /// 
    /// // calculation by the ray-tracing
    /// let pt = Point3::new(-1.5, -1.4, -2.5);
    /// let vec = pt - camera.position();
    /// let dir = camera.eye_direction();
    /// let y_axis = camera.head_direction();
    /// let x_axis = dir.cross(y_axis);
    /// let h = vec - vec.dot(dir) * dir;
    /// let u = h.dot(x_axis) / (size / 2.0) / as_rat;
    /// let v = h.dot(y_axis) / (size / 2.0);
    /// 
    /// // check the answer
    /// let uv = camera.projection(as_rat).transform_point(pt);
    /// assert!(f64::near(&u, &uv[0]), "{} {}", u, uv[0]);
    /// assert!(f64::near(&v, &uv[1]));
    /// ```
    #[inline(always)]
    pub fn projection(&self, as_rat: f64) -> Matrix4 {
        Matrix4::from_nonuniform_scale(1.0 / as_rat, 1.0, 1.0)
            * self.projection
            * self.matrix.invert().unwrap()
    }

    fn camera_info(&self, as_rat: f64) -> CameraInfo {
        CameraInfo {
            camera_matrix: (&self.matrix).cast().unwrap().into(),
            camera_projection: self.projection(as_rat).cast().unwrap().into(),
        }
    }

    /// Creates a `UNIFORM` buffer of camera.
    /// 
    /// The bind group provides [`Scene`] holds this uniform buffer.
    /// 
    /// # Shader Example
    /// ```glsl
    /// layout(set = 0, binding = 0) uniform Camera {
    ///     mat4 camera_matrix;     // the camera matrix
    ///     mat4 camera_projection; // the projection into the normalized view volume
    /// };
    /// ```
    pub fn buffer(&self, as_rat: f64, device: &Device) -> BufferHandler {
        BufferHandler::from_slice(&[self.camera_info(as_rat)], device, BufferUsage::UNIFORM)
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
