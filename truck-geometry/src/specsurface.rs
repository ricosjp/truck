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
/// // The normal is (A - O).cross(B - O)
/// Vector3::assert_near(&plane.normal(), &Vector3::new(-1.0, -1.0, 1.0).normalize());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    o: Point3,
    p: Point3,
    q: Point3,
}

impl Plane {
    /// Creates a new plane from three points.
    #[inline(always)]
    pub fn new(origin: Point3, one: Point3, another: Point3) -> Plane {
        Plane {
            o: origin,
            p: one,
            q: another,
        }
    }
    /// Returns the origin
    #[inline(always)]
    pub fn origin(&self) -> Point3 { self.o }
    /// Returns the normal
    #[inline(always)]
    pub fn normal(&self) -> Vector3 { (self.p - self.o).cross(self.q - self.o).normalize() }
    /// into B-spline surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let pt0 = Point3::new(0.0, 1.0, 2.0);
    /// let pt1 = Point3::new(1.0, 1.0, 3.0);
    /// let pt2 = Point3::new(0.0, 2.0, 3.0);
    /// let plane: Plane = Plane::new(pt0, pt1, pt2);
    /// let surface: BSplineSurface<Vector3> = plane.into_bspline();
    /// assert_eq!(surface.parameter_range(), ((0.0, 1.0), (0.0, 1.0)));
    ///
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     for j in 0..=N {
    ///         let u = i as f64 / N as f64;
    ///         let v = j as f64 / N as f64;
    ///         let res = ParametricSurface::subs(&surface, u, v);
    ///         let ans = plane.subs(u, v);
    ///         Point3::assert_near(&ans, &res);
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn into_bspline(&self) -> BSplineSurface<Vector3> {
        let o = self.o.to_vec();
        let p = self.p.to_vec();
        let q = self.q.to_vec();
        BSplineSurface {
            knot_vecs: (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
            control_points: vec![vec![o, q], vec![p, p + q - o]],
        }
    }
    /// into NURBS surface
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let pt0 = Point3::new(0.0, 1.0, 2.0);
    /// let pt1 = Point3::new(1.0, 1.0, 3.0);
    /// let pt2 = Point3::new(0.0, 2.0, 3.0);
    /// let plane: Plane = Plane::new(pt0, pt1, pt2);
    /// let surface: NURBSSurface<Vector4> = plane.into_nurbs();
    /// assert_eq!(surface.parameter_range(), ((0.0, 1.0), (0.0, 1.0)));
    ///
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     for j in 0..=N {
    ///         let u = i as f64 / N as f64;
    ///         let v = j as f64 / N as f64;
    ///         let res = surface.subs(u, v);
    ///         let ans = plane.subs(u, v);
    ///         Point3::assert_near(&ans, &res);
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn into_nurbs(&self) -> NURBSSurface<Vector4> {
        let o = self.o.to_homogeneous();
        let p = self.p.to_homogeneous();
        let q = self.q.to_homogeneous();
        NURBSSurface(BSplineSurface {
            knot_vecs: (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
            control_points: vec![vec![o, q], vec![p, p + q - o]],
        })
    }
}

impl ParametricSurface for Plane {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 {
        self.o + u * (self.p - self.o) + v * (self.q - self.o)
    }
    #[inline(always)]
    fn uder(&self, _: f64, _: f64) -> Vector3 { self.p - self.o }
    #[inline(always)]
    fn vder(&self, _: f64, _: f64) -> Vector3 { self.q - self.o }
    #[inline(always)]
    fn normal(&self, _: f64, _: f64) -> Vector3 { self.normal() }
}

impl BoundedSurface for Plane {
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { ((0.0, 1.0), (0.0, 1.0)) }
}

impl Invertible for Plane {
    fn inverse(&self) -> Self {
        Plane {
            o: self.o,
            p: self.q,
            q: self.p,
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
            o: self.transform_point(plane.o),
            p: self.transform_point(plane.p),
            q: self.transform_point(plane.q),
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
