use crate::*;
use errors::Error;
use std::f64::consts::PI;

/// bounded plane
/// # Example
/// ```
/// use truck_geometry::*;
/// let plane: Plane = Plane::new(
///     Point3::new(0.0, 1.0, 2.0), // O
///     Point3::new(1.0, 1.0, 3.0), // A
///     Point3::new(0.0, 2.0, 3.0), // B
/// );
/// // The origin of the plane is O.
/// Point3::assert_near(&plane.origin(), &Point3::new(0.0, 1.0, 2.0));
/// // A is on the u-axis.
/// Vector3::assert_near(&plane.u_axis(), &Vector3::new(1.0, 0.0, 1.0).normalize());
/// // The normal is (A - O).cross(B - O)
/// Vector3::assert_near(&plane.normal(), &Vector3::new(-1.0, -1.0, 1.0).normalize());
/// // The v-axis is calculated by the u-axis and the normal.
/// Vector3::assert_near(&plane.v_axis(), &Vector3::new(-1.0, 2.0, 1.0).normalize());
///
/// // the parameter range of the bounded plane
/// let (urange, vrange) = plane.parameter_range();
/// // The minimum of the range is 0.0.
/// f64::assert_near(&urange.0, &0.0); f64::assert_near(&vrange.0, &0.0);
/// // The maximum of the u-range is |OA|
/// f64::assert_near(&urange.1, &f64::sqrt(2.0));
/// // The minimum of the v-range is OB.dot(v-axis).
/// f64::assert_near(&urange.1, &f64::sqrt(2.0));
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    matrix: Matrix4,
    parameter_range: ((f64, f64), (f64, f64)),
}

impl Plane {
    /// Creates a new plane whose origin is `origin` and `one_point` is on the u-axis.
    #[inline(always)]
    pub fn new(origin: Point3, one_point: Point3, another_point: Point3) -> Plane {
        let v0 = (one_point - origin).normalize();
        let v1 = another_point - origin;
        let v2 = v0.cross(v1).normalize();
        let v1 = v2.cross(v0).normalize();
        Plane {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            matrix: Matrix4::new(
                v0[0], v0[1], v0[2], 0.0,
                v1[0], v1[1], v1[2], 0.0,
                v2[0], v2[1], v2[2], 0.0,
                origin[0], origin[1], origin[2], 1.0,
            ),
            parameter_range: (
                (0.0, v0.dot(one_point - origin)),
                (0.0, v1.dot(another_point - origin)),
            ),
        }
    }
    /// Creates a new plane whose origin is `origin` and `one_point` is on the u-axis.
    /// # Panics
    /// Panic occurs if `urange.0 >= urange.1` or `vrange.0 >= vrange.1`.
    #[inline(always)]
    pub fn with_parameter_range(
        origin: Point3,
        one_point: Point3,
        another_point: Point3,
        urange: (f64, f64),
        vrange: (f64, f64),
    ) -> Plane {
        if urange.0 >= urange.1 {
            panic!("{}", Error::IncorrectRange(urange.0, urange.1));
        } else if vrange.0 >= vrange.1 {
            panic!("{}", Error::IncorrectRange(vrange.0, vrange.1));
        }
        let v0 = (one_point - origin).normalize();
        let v1 = another_point - origin;
        let v2 = v0.cross(v1).normalize();
        let v1 = v2.cross(v0).normalize();
        Plane {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            matrix: Matrix4::new(
                v0[0], v0[1], v0[2], 0.0,
                v1[0], v1[1], v1[2], 0.0,
                v2[0], v2[1], v2[2], 0.0,
                origin[0], origin[1], origin[2], 1.0,
            ),
            parameter_range: (urange, vrange),
        }
    }

    /// Returns the u-axis
    #[inline(always)]
    pub fn u_axis(&self) -> Vector3 { self.matrix[0].truncate() }
    /// Returns the v-axis
    #[inline(always)]
    pub fn v_axis(&self) -> Vector3 { self.matrix[1].truncate() }
    /// Returns the normal
    #[inline(always)]
    pub fn normal(&self) -> Vector3 { self.matrix[2].truncate() }
    /// Returns the origin
    #[inline(always)]
    pub fn origin(&self) -> Point3 { Point3::from_vec(self.matrix[3].truncate()) }
    /// Returns the matrix of the plane
    #[inline(always)]
    pub fn matrix(&self) -> Matrix4 { self.matrix }
    /// Set the paraameter range
    #[inline(always)]
    pub fn set_parameter_range(&mut self, urange: (f64, f64), vrange: (f64, f64)) {
        if urange.0 > urange.1 {
            panic!("urange is incorrect.")
        } else if vrange.0 > vrange.1 {
            panic!("vrange is incorrect.")
        } else {
            self.parameter_range = (urange, vrange);
        }
    }
    /// into B-spline surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let plane: Plane = Plane::new(
    ///     Point3::new(0.0, 1.0, 2.0),
    ///     Point3::new(1.0, 1.0, 3.0),
    ///     Point3::new(0.0, 2.0, 3.0),
    /// );
    /// let surface: BSplineSurface<Vector3> = plane.into_bspline();
    /// assert_eq!(plane.parameter_range(), surface.parameter_range());
    ///
    /// let ((u0, u1), (v0, v1)) = plane.parameter_range();
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     for j in 0..=N {
    ///         let mut u = i as f64 / N as f64;
    ///         u = u0 * (1.0 - u) + u1 * u;
    ///         let mut v = j as f64 / N as f64;
    ///         v = v0 * (1.0 - v) + v1 * v;
    ///         Point3::assert_near(&plane.subs(u, v), &Surface::subs(&surface, u, v));
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn into_bspline(&self) -> BSplineSurface<Vector3> {
        let ((u0, u1), (v0, v1)) = self.parameter_range;
        let uknot_vec = KnotVec(vec![u0, u0, u1, u1]);
        let vknot_vec = KnotVec(vec![v0, v0, v1, v1]);
        let origin = self.matrix[3].truncate();
        let u_axis = self.matrix[0].truncate();
        let v_axis = self.matrix[1].truncate();
        let control_points = vec![
            vec![
                origin + u0 * u_axis + v0 * v_axis,
                origin + u0 * u_axis + v1 * v_axis,
            ],
            vec![
                origin + u1 * u_axis + v0 * v_axis,
                origin + u1 * u_axis + v1 * v_axis,
            ],
        ];
        BSplineSurface {
            knot_vecs: (uknot_vec, vknot_vec),
            control_points,
        }
    }
    /// into NURBS surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let plane: Plane = Plane::new(
    ///     Point3::new(0.0, 1.0, 2.0),
    ///     Point3::new(1.0, 1.0, 3.0),
    ///     Point3::new(0.0, 2.0, 3.0),
    /// );
    /// let surface: NURBSSurface<Vector4> = plane.into_nurbs();
    /// assert_eq!(plane.parameter_range(), surface.parameter_range());
    ///
    /// let ((u0, u1), (v0, v1)) = plane.parameter_range();
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     for j in 0..=N {
    ///         let mut u = i as f64 / N as f64;
    ///         u = u0 * (1.0 - u) + u1 * u;
    ///         let mut v = j as f64 / N as f64;
    ///         v = v0 * (1.0 - v) + v1 * v;
    ///         Point3::assert_near(&plane.subs(u, v), &surface.subs(u, v));
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn into_nurbs(&self) -> NURBSSurface<Vector4> {
        let ((u0, u1), (v0, v1)) = self.parameter_range;
        let uknot_vec = KnotVec(vec![u0, u0, u1, u1]);
        let vknot_vec = KnotVec(vec![v0, v0, v1, v1]);
        let origin = self.matrix[3];
        let u_axis = self.matrix[0];
        let v_axis = self.matrix[1];
        let control_points = vec![
            vec![
                origin + u0 * u_axis + v0 * v_axis,
                origin + u0 * u_axis + v1 * v_axis,
            ],
            vec![
                origin + u1 * u_axis + v0 * v_axis,
                origin + u1 * u_axis + v1 * v_axis,
            ],
        ];
        NURBSSurface(BSplineSurface {
            knot_vecs: (uknot_vec, vknot_vec),
            control_points,
        })
    }
}

impl Surface for Plane {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 {
        self.origin() + u * self.u_axis() + v * self.v_axis()
    }
    #[inline(always)]
    fn uder(&self, _: f64, _: f64) -> Vector3 { self.u_axis() }
    #[inline(always)]
    fn vder(&self, _: f64, _: f64) -> Vector3 { self.v_axis() }
    #[inline(always)]
    fn normal(&self, _: f64, _: f64) -> Vector3 { self.normal() }
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { self.parameter_range }
    #[inline(always)]
    fn inverse(&self) -> Plane {
        let (urange, vrange) = self.parameter_range;
        Plane {
            matrix: Matrix4::from_cols(
                self.matrix[1],
                self.matrix[0],
                -self.matrix[2],
                self.matrix[3],
            ),
            parameter_range: (vrange, urange),
        }
    }
}

impl IncludeCurve<BSplineCurve<Vector3>> for Plane {
    #[inline(always)]
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        let origin = self.origin().to_vec();
        let normal = self.normal();
        curve
            .control_points
            .iter()
            .all(|pt| (pt - origin).dot(normal).so_small())
    }
}

impl IncludeCurve<NURBSCurve<Vector4>> for Plane {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let origin = self.origin();
        let normal = self.normal();
        let (s, e) = (curve.front(), curve.back());
        if !(s - origin).dot(normal).so_small() || !(e - origin).dot(normal).so_small() {
            return false;
        }
        curve.0.control_points.iter().all(|pt| {
            if pt[4].so_small() {
                true
            } else {
                let pt = Point3::from_homogeneous(*pt);
                (pt - origin).dot(normal).so_small()
            }
        })
    }
}

/// the sphere
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere {
    matrix: Matrix4,
    radius: f64,
    ori: f64,
}

impl Sphere {
    /// Creates a sphere
    #[inline(always)]
    pub fn new(center: Point3, radius: f64) -> Sphere {
        Sphere {
            matrix: Matrix4::from_translation(center.to_vec()),
            radius,
            ori: 1.0,
        }
    }
    /// Returns the center
    #[inline(always)]
    pub fn center(&self) -> Point3 { Point3::from_vec(self.matrix[3].truncate()) }
    /// Returns the radius
    #[inline(always)]
    pub fn radius(&self) -> f64 { self.radius }
}

impl Surface for Sphere {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        Vector3::new(
            f64::sin(u) * f64::cos(v),
            self.ori * f64::sin(u) * f64::sin(v),
            f64::cos(u),
        )
    }
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 { self.center() + self.radius * self.normal(u, v) }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        self.radius
            * Vector3::new(
                f64::cos(u) * f64::cos(v),
                self.ori * f64::cos(u) * f64::sin(v),
                -f64::sin(u),
            )
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        self.radius * f64::sin(u) * Vector3::new(-self.ori * f64::sin(v), f64::cos(v), 0.0)
    }
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { ((0.0, PI), (0.0, 2.0 * PI)) }
    #[inline(always)]
    fn inverse(&self) -> Self {
        Sphere {
            matrix: self.matrix,
            radius: self.radius,
            ori: -self.ori,
        }
    }
}
