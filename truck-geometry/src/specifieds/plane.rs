use super::*;

impl Plane {
    /// Creates a new plane from three points.
    #[inline(always)]
    pub const fn new(origin: Point3, one: Point3, another: Point3) -> Plane {
        Plane {
            o: origin,
            p: one,
            q: another,
        }
    }
    /// Returns the origin
    #[inline(always)]
    pub const fn origin(&self) -> Point3 { self.o }
    /// Returns the u-axis
    #[inline(always)]
    pub fn u_axis(&self) -> Vector3 { self.p - self.o }
    /// Returns the v-axis
    #[inline(always)]
    pub fn v_axis(&self) -> Vector3 { self.q - self.o }
    /// Returns the normal
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let plane = Plane::new(
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    /// );
    /// assert_near!(plane.normal(), Vector3::unit_z());
    /// ```
    #[inline(always)]
    pub fn normal(&self) -> Vector3 { self.u_axis().cross(self.v_axis()).normalize() }
    /// Gets the parameter of `pt` in plane's matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let plane = Plane::new(
    ///     Point3::new(1.0, 2.0, 3.0),
    ///     Point3::new(2.0, 1.0, 3.0),
    ///     Point3::new(3.0, 4.0, -1.0),
    /// );
    ///
    /// let pt = Point3::new(2.1, -6.5, 4.7);
    /// let prm = plane.get_parameter(pt);
    /// let rev = plane.origin()
    ///     + prm[0] * plane.u_axis()
    ///     + prm[1] * plane.v_axis()
    ///     + prm[2] * plane.normal();
    /// assert_near!(pt, rev);
    /// ```
    #[inline(always)]
    pub fn get_parameter(&self, pt: Point3) -> Vector3 {
        let a = self.u_axis();
        let b = self.v_axis();
        let c = self.normal();
        let mat = Matrix3::from_cols(a, b, c).invert().unwrap();
        mat * (pt - self.o)
    }
    /// into B-spline surface
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let pt0 = Point3::new(0.0, 1.0, 2.0);
    /// let pt1 = Point3::new(1.0, 1.0, 3.0);
    /// let pt2 = Point3::new(0.0, 2.0, 3.0);
    /// let plane: Plane = Plane::new(pt0, pt1, pt2);
    /// let surface: BSplineSurface<Point3> = plane.into_bspline();
    /// assert_eq!(surface.parameter_range(), ((0.0, 1.0), (0.0, 1.0)));
    ///
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     for j in 0..=N {
    ///         let u = i as f64 / N as f64;
    ///         let v = j as f64 / N as f64;
    ///         let res = surface.subs(u, v);
    ///         let ans = plane.subs(u, v);
    ///         assert_near!(ans, res);
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn into_bspline(&self) -> BSplineSurface<Point3> {
        let o = self.o;
        let p = self.p;
        let q = self.q;
        BSplineSurface::debug_new(
            (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
            vec![vec![o, q], vec![p, p + (q - o)]],
        )
    }
    /// into NURBS surface
    /// # Examples
    /// ```
    /// use truck_geometry::prelude::*;
    /// let pt0 = Point3::new(0.0, 1.0, 2.0);
    /// let pt1 = Point3::new(1.0, 1.0, 3.0);
    /// let pt2 = Point3::new(0.0, 2.0, 3.0);
    /// let plane: Plane = Plane::new(pt0, pt1, pt2);
    /// let surface: NurbsSurface<Vector4> = plane.into_nurbs();
    /// assert_eq!(surface.parameter_range(), ((0.0, 1.0), (0.0, 1.0)));
    ///
    /// const N: usize = 100;
    /// for i in 0..=N {
    ///     for j in 0..=N {
    ///         let u = i as f64 / N as f64;
    ///         let v = j as f64 / N as f64;
    ///         let res = surface.subs(u, v);
    ///         let ans = plane.subs(u, v);
    ///         assert_near!(ans, res);
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn into_nurbs(&self) -> NurbsSurface<Vector4> {
        let o = self.o.to_homogeneous();
        let p = self.p.to_homogeneous();
        let q = self.q.to_homogeneous();
        NurbsSurface::new(BSplineSurface::debug_new(
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
    fn uuder(&self, _: f64, _: f64) -> Vector3 { Vector3::zero() }
    #[inline(always)]
    fn uvder(&self, _: f64, _: f64) -> Vector3 { Vector3::zero() }
    #[inline(always)]
    fn vvder(&self, _: f64, _: f64) -> Vector3 { Vector3::zero() }
}

impl ParametricSurface3D for Plane {
    #[inline(always)]
    fn normal(&self, _: f64, _: f64) -> Vector3 { self.normal() }
}

impl BoundedSurface for Plane {
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { ((0.0, 1.0), (0.0, 1.0)) }
}

impl Invertible for Plane {
    #[inline(always)]
    fn inverse(&self) -> Self {
        Plane {
            o: self.o,
            p: self.q,
            q: self.p,
        }
    }
    #[inline(always)]
    fn invert(&mut self) { *self = self.inverse(); }
}

impl IncludeCurve<BSplineCurve<Point3>> for Plane {
    #[inline(always)]
    fn include(&self, curve: &BSplineCurve<Point3>) -> bool {
        let origin = self.origin();
        let normal = self.normal();
        curve
            .control_points()
            .iter()
            .all(|pt| (pt - origin).dot(normal).so_small())
    }
}

impl IncludeCurve<NurbsCurve<Vector4>> for Plane {
    fn include(&self, curve: &NurbsCurve<Vector4>) -> bool {
        let origin = self.origin();
        let normal = self.normal();
        let (s, e) = (curve.front(), curve.back());
        if !(s - origin).dot(normal).so_small() || !(e - origin).dot(normal).so_small() {
            return false;
        }
        curve.non_rationalized().control_points().iter().all(|pt| {
            if pt[3].so_small() {
                true
            } else {
                let pt = Point3::from_homogeneous(*pt);
                (pt - origin).dot(normal).so_small()
            }
        })
    }
}

impl ParameterDivision2D for Plane {
    #[inline(always)]
    fn parameter_division(&self, range: ((f64, f64), (f64, f64)), _: f64) -> (Vec<f64>, Vec<f64>) {
        (vec![range.0 .0, range.0 .1], vec![range.1 .0, range.1 .1])
    }
}

impl<T: Transform3<Scalar = f64>> Transformed<T> for Plane {
    #[inline(always)]
    fn transform_by(&mut self, trans: T) {
        self.o = trans.transform_point(self.o);
        self.p = trans.transform_point(self.p);
        self.q = trans.transform_point(self.q);
    }
    #[inline(always)]
    fn transformed(&self, trans: T) -> Self {
        Plane {
            o: trans.transform_point(self.o),
            p: trans.transform_point(self.p),
            q: trans.transform_point(self.q),
        }
    }
}

impl SearchParameter<D2> for Plane {
    type Point = Point3;
    #[inline(always)]
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        _: H,
        _: usize,
    ) -> Option<(f64, f64)> {
        let v = self.get_parameter(point);
        match v[2].so_small() {
            true => Some((v[0], v[1])),
            false => None,
        }
    }
}

impl SearchNearestParameter<D2> for Plane {
    type Point = Point3;
    #[inline(always)]
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        _: H,
        _: usize,
    ) -> Option<(f64, f64)> {
        let v = self.get_parameter(point);
        Some((v[0], v[1]))
    }
}
