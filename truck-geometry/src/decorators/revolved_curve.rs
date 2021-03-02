use super::*;
use std::f64::consts::PI;

impl<C> RevolutedCurve<C> {
    /// Creates a surface by revoluting a curve.
    #[inline(always)]
    pub fn by_revolution(curve: C, origin: Point3, axis: Vector3) -> Self {
        RevolutedCurve {
            curve,
            origin,
            axis: axis.normalize(),
        }
    }
    #[inline(always)]
    fn point_rotation_matrix(&self, v: f64) -> Matrix4 {
        Matrix4::from_translation(self.origin.to_vec())
            * Matrix4::from_axis_angle(self.axis, Rad(v))
            * Matrix4::from_translation(-self.origin.to_vec())
    }
    #[inline(always)]
    fn vector_rotation_matrix(&self, v: f64) -> Matrix4 {
        Matrix4::from_axis_angle(self.axis, Rad(v))
    }
}

impl<C: Curve<Point = Point3, Vector = Vector3>> ParametricSurface for RevolutedCurve<C> {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 {
        self.point_rotation_matrix(v)
            .transform_point(self.curve.subs(u))
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        self.vector_rotation_matrix(v)
            .transform_vector(self.curve.der(u))
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        let pt = self.curve.subs(u);
        let radius = self.axis.cross(pt - self.origin);
        self.vector_rotation_matrix(v).transform_vector(radius)
    }
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        let uder = self.uder(u, v);
        let vder = self.axis.cross(uder);
        uder.cross(vder).normalize()
    }
}

impl<C: Curve<Point = Point3, Vector = Vector3>> BoundedSurface for RevolutedCurve<C> {
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) {
        (self.curve.parameter_range(), (0.0, 2.0 * PI))
    }
}

impl<C: Clone> Invertible for RevolutedCurve<C> {
    #[inline(always)]
    fn invert(&mut self) { self.axis = -self.axis; }
    #[inline(always)]
    fn inverse(&self) -> Self {
        RevolutedCurve {
            curve: self.curve.clone(),
            origin: self.origin,
            axis: -self.axis,
        }
    }
}