use crate::*;

impl Ray {
    /// Returns the origin of the ray
    #[inline(always)]
    pub const fn origin(&self) -> Point3 { self.origin }
    /// Returns the (normalized) direction of the ray
    #[inline(always)]
    pub const fn direction(&self) -> Vector3 { self.direction }
}

const fn parallel(screen_size: f64, near_clip: f64, far_clip: f64) -> Matrix4 {
    let a = 2.0 / screen_size;
    let b = -1.0 / (far_clip - near_clip);
    let c = -near_clip / (far_clip - near_clip);
    Matrix4::from_cols(
        Vector4::new(a, 0.0, 0.0, 0.0),
        Vector4::new(0.0, a, 0.0, 0.0),
        Vector4::new(0.0, 0.0, b, 0.0),
        Vector4::new(0.0, 0.0, c, 1.0),
    )
}

impl ProjectionMethod {
    /// Returns `ProjectionMethod::Perspective { fov }`.
    #[inline(always)]
    pub const fn perspective(fov: Rad<f64>) -> Self { Self::Perspective { fov } }
    /// Returns `ProjectionMethod::Parallel { screen_size }`.
    #[inline(always)]
    pub const fn parallel(screen_size: f64) -> Self { Self::Parallel { screen_size } }
}

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

    /// Returns the projection matrix into the normalized view volume.
    /// # Arguments
    /// `as_rat`: the aspect ratio, x-resolution / y-resolution.
    /// # Examples
    /// ```
    /// // perspective camera
    /// use std::f64::consts::PI;
    /// use truck_base::{assert_near, cgmath64::*, tolerance::*};
    /// use truck_platform::*;
    ///
    /// let fov = PI / 4.0;
    /// let as_rat = 1.2;
    /// let matrix = Matrix4::look_at_rh(
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::origin(),
    ///     Vector3::new(0.0, 1.0, 0.0),
    /// );
    /// let camera = Camera {
    ///     matrix: matrix.invert().unwrap(),
    ///     method: ProjectionMethod::perspective(Rad(fov)),
    ///     near_clip: 0.1,
    ///     far_clip: 10.0,
    /// };
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
    /// assert_near!(u, uv[0]);
    /// assert_near!(v, uv[1]);
    /// ```
    /// ```
    /// // parallel camera
    /// use truck_base::{assert_near, cgmath64::*, tolerance::*};
    /// use truck_platform::*;
    ///
    /// let size = 3.0;
    /// let as_rat = 1.2;
    /// let matrix = Matrix4::look_at_rh(
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::origin(),
    ///     Vector3::new(0.0, 1.0, 0.0),
    /// );
    /// let camera = Camera {
    ///     matrix: matrix.invert().unwrap(),
    ///     method: ProjectionMethod::parallel(size),
    ///     near_clip: 0.1,
    ///     far_clip: 10.0,
    /// };
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
    /// assert_near!(u, uv[0]);
    /// assert_near!(v, uv[1]);
    /// ```
    #[inline(always)]
    pub fn projection(&self, as_rat: f64) -> Matrix4 {
        let (near, far) = (self.near_clip, self.far_clip);
        let normal_projection = match self.method {
            ProjectionMethod::Perspective { fov } => perspective(fov, 1.0, near, far),
            ProjectionMethod::Parallel { screen_size } => parallel(screen_size, near, far),
        };
        Matrix4::from_nonuniform_scale(1.0 / as_rat, 1.0, 1.0)
            * normal_projection
            * self.matrix.invert().unwrap()
    }

    fn camera_info(&self, as_rat: f64) -> CameraInfo {
        CameraInfo {
            camera_matrix: self.matrix.cast().unwrap().into(),
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
        BufferHandler::from_slice(&[self.camera_info(as_rat)], device, BufferUsages::UNIFORM)
    }

    /// Returns the ray from camera with aspect-ratio = 1.0.
    ///
    /// # Examples
    /// ```
    /// // Perspective case
    /// use std::f64::consts::PI;
    /// use truck_base::{assert_near, cgmath64::*, tolerance::Tolerance};
    /// use truck_platform::*;
    ///
    ///
    /// let matrix = Matrix4::look_at_rh(
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::origin(),
    ///     Vector3::new(0.0, 1.0, 0.0),
    /// );
    /// let camera = Camera {
    ///     // depends on the difference of the style with cgmath,
    ///     // the matrix must be inverted
    ///     matrix: matrix.invert().unwrap(),
    ///     method: ProjectionMethod::perspective(Rad(PI / 4.0)),
    ///     near_clip: 0.1,
    ///     far_clip: 1.0,
    /// };
    ///
    /// // take a point in the 3D space
    /// let point = Point3::new(0.1, 0.15, 0.0);
    /// // project to the normalized view volume
    /// let uvz = camera.projection(1.0).transform_point(point);
    /// // coordinate on the screen
    /// let uv = Point2::new(uvz.x, uvz.y);
    ///
    /// let ray = camera.ray(uv);
    /// // the origin of the ray is camera position
    /// assert_near!(ray.origin(), camera.position());
    /// // the direction of the ray is the normalized vector of point - camera.position().
    /// assert_near!(ray.direction(), (point - camera.position()).normalize());
    /// ```
    /// ```
    /// // Parallel case
    /// use truck_base::{assert_near, cgmath64::*, tolerance::*};
    /// use truck_platform::*;
    ///
    /// let matrix = Matrix4::look_at_rh(
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::origin(),
    ///     Vector3::new(0.0, 1.0, 0.0),
    /// );
    /// let camera = Camera {
    ///     matrix: matrix.invert().unwrap(),
    ///     method: ProjectionMethod::parallel(3.0),
    ///     near_clip: 0.1,
    ///     far_clip: 10.0,
    /// };
    ///
    /// // take a point in the 3D space
    /// let point = Point3::new(0.1, 0.15, 0.0);
    /// // the projection of the point to the screen
    /// let projed = point
    ///     - camera.eye_direction() * camera.eye_direction().dot(point - camera.position());
    /// // project to the normalized view volume
    /// let uvz = camera.projection(1.0).transform_point(point);
    /// // coordinate on the screen
    /// let uv = Point2::new(uvz.x, uvz.y);
    ///
    /// let ray = camera.ray(uv);
    /// // the origin of the ray is the projection of the point.
    /// assert_near!(ray.origin(), projed);
    /// // the direction of the ray is eye direction.
    /// assert_near!(ray.direction(), camera.eye_direction());
    /// ```
    pub fn ray(&self, coord: Point2) -> Ray {
        match self.method {
            ProjectionMethod::Perspective { .. } => {
                let mat = self
                    .projection(1.0)
                    .invert()
                    .expect("non-invertible projection");
                let x = mat.transform_point(Point3::new(coord.x, coord.y, 0.5));
                let y = mat.transform_point(Point3::new(coord.x, coord.y, 1.0));
                Ray {
                    origin: self.position(),
                    direction: (y - x).normalize(),
                }
            }
            ProjectionMethod::Parallel { screen_size } => {
                let a = screen_size / 2.0;
                let axis_x = self.matrix[0].truncate() * a;
                let axis_y = self.matrix[1].truncate() * a;
                Ray {
                    origin: self.position() + coord.x * axis_x + coord.y * axis_y,
                    direction: self.eye_direction(),
                }
            }
        }
    }
}

impl Default for Camera {
    #[inline(always)]
    fn default() -> Self {
        Self {
            matrix: Matrix4::identity(),
            method: ProjectionMethod::perspective(Rad(std::f64::consts::PI / 4.0)),
            near_clip: 0.1,
            far_clip: 10.0,
        }
    }
}
