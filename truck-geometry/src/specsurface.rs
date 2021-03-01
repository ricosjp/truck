use crate::*;
use std::f64::consts::PI;
use std::ops::Mul;

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
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    origin: Point3,
    normal: Vector3,
}

impl Plane {
    /// Creates a new plane from origin and normal.
    #[inline(always)]
    pub fn new(origin: Point3, normal: Vector3) -> Plane {
        Plane {
            origin,
            normal: normal.normalize(),
        }
    }
    /// Creates a new plane from three points.
    #[inline(always)]
    pub fn from_triangle(origin: Point3, one: Point3, another: Point3) -> Plane {
        let normal = (one - origin).cross(another - origin).normalize();
        Plane { origin, normal }
    }
    /// Returns the origin
    #[inline(always)]
    pub fn origin(&self) -> Point3 { self.origin }
    /// Returns the normal
    #[inline(always)]
    pub fn normal(&self) -> Vector3 { self.normal }
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
    pub fn into_bspline(
        &self,
        u_axis: Vector3,
        parameter_range: ((f64, f64), (f64, f64)),
    ) -> BSplineSurface<Vector3> {
        let ((u0, u1), (v0, v1)) = parameter_range;
        let uknot_vec = KnotVec(vec![u0, u0, u1, u1]);
        let vknot_vec = KnotVec(vec![v0, v0, v1, v1]);
        let u_axis = (u_axis - self.normal.dot(u_axis) * self.normal).normalize();
        let v_axis = self.normal.cross(u_axis);
        let origin = self.origin.to_vec();
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
    pub fn into_nurbs(
        &self,
        u_axis: Vector3,
        parameter_range: ((f64, f64), (f64, f64)),
    ) -> NURBSSurface<Vector4> {
        let ((u0, u1), (v0, v1)) = parameter_range;
        let uknot_vec = KnotVec(vec![u0, u0, u1, u1]);
        let vknot_vec = KnotVec(vec![v0, v0, v1, v1]);
        let origin = self.origin.to_homogeneous();
        let u_axis = (u_axis - self.normal.dot(u_axis) * self.normal).normalize();
        let v_axis = self.normal.cross(u_axis);
        let u_axis = u_axis.extend(1.0);
        let v_axis = v_axis.extend(1.0);
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

impl Invertible for Plane {
    fn inverse(&self) -> Self {
        Plane {
            origin: self.origin,
            normal: -self.normal,
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

impl Mul<Plane> for Matrix4 {
    type Output = Plane;
    fn mul(self, plane: Plane) -> Plane {
        Plane {
            origin: self.transform_point(plane.origin),
            normal: self.transform_vector(plane.normal).normalize(),
        }
    }
}

/// sphere
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    ori: f64,
}

impl Sphere {
    /// Creates a sphere
    #[inline(always)]
    pub fn new(center: Point3, radius: f64) -> Sphere {
        Sphere {
            center,
            radius,
            ori: 1.0,
        }
    }
    /// Returns the center
    #[inline(always)]
    pub fn center(&self) -> Point3 { self.center }
    /// Returns the radius
    #[inline(always)]
    pub fn radius(&self) -> f64 { self.radius }
    /// Inverts the sphere
    #[inline(always)]
    pub fn invert(&mut self) { self.ori = -self.ori }
    /// Returns orientation
    #[inline(always)]
    pub fn orientation(&self) -> bool { self.ori > 0.0 }
    /// Returns whether the point `pt` is on sphere
    #[inline(always)]
    pub fn include(&self, pt: Point3) -> bool { self.center.distance(pt).near(&self.radius) }
}

impl ParametricSurface for Sphere {
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
        self.radius * f64::sin(u) * Vector3::new(f64::sin(v), f64::cos(v), 0.0)
    }
}

impl BoundedSurface for Sphere {
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { ((0.0, PI), (0.0, 2.0 * PI)) }
}

impl Invertible for Sphere {
    #[inline(always)]
    fn inverse(&self) -> Self {
        Sphere {
            center: self.center,
            radius: self.radius,
            ori: -self.ori,
        }
    }
}

impl IncludeCurve<BSplineCurve<Vector3>> for Sphere {
    #[inline(always)]
    fn include(&self, curve: &BSplineCurve<Vector3>) -> bool {
        curve.is_const() && self.include(curve.front())
    }
}

impl IncludeCurve<NURBSCurve<Vector4>> for Sphere {
    fn include(&self, curve: &NURBSCurve<Vector4>) -> bool {
        let (knots, _) = curve.knot_vec().to_single_multi();
        let degree = curve.degree() * 2;
        knots
            .windows(2)
            .flat_map(move |window| (1..degree).map(move |i| (window, i)))
            .all(move |(window, i)| {
                let t = i as f64 / degree as f64;
                let t = window[0] * (1.0 - t) + window[1] * t;
                self.include(curve.subs(t))
            })
    }
}
