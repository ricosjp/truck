use super::*;
use std::ops::Mul;

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
        BSplineSurface::debug_new(
            (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
            vec![vec![o, q], vec![p, p + q - o]],
        )
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
        NURBSSurface::new(BSplineSurface::debug_new(
            (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
            vec![vec![o, q], vec![p, p + q - o]],
        ))
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
            .control_points()
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
        curve.non_rationalized().control_points().iter().all(|pt| {
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