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
        let (u0, u1) = self.curve.parameter_range();
        let (uder, vder) = if u.near(&u0) {
            let pt = self.curve.subs(u);
            let radius = self.axis.cross(pt - self.origin);
            if radius.so_small() {
                let uder = self.curve.der(u);
                (uder, self.axis.cross(uder))
            } else {
                (self.uder(u, v), self.vder(u, v))
            }
        } else if u.near(&u1) {
            let pt = self.curve.subs(u);
            let radius = self.axis.cross(pt - self.origin);
            if radius.so_small() {
                let uder = self.curve.der(u);
                (uder, uder.cross(self.axis))
            } else {
                (self.uder(u, v), self.vder(u, v))
            }
        } else {
            (self.uder(u, v), self.vder(u, v))
        };
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

#[test]
fn revolve_test() {
    let pt0 = Vector3::new(0.0, 2.0, 1.0);
    let pt1 = Vector3::new(1.0, 0.0, 0.0);
    let vec = pt1 - pt0;
    let curve = BSplineCurve::new(KnotVec::bezier_knot(1), vec![pt0, pt1]);
    let surface = RevolutedCurve::by_revolution(curve, Point3::origin(), Vector3::unit_y());
    const N: usize = 100;
    for i in 0..=N {
        for j in 0..=N {
            let u = i as f64 / N as f64;
            let v = 2.0 * PI * j as f64 / N as f64;
            let uder = Matrix3::from_axis_angle(Vector3::unit_y(), Rad(v)) * vec;
            Vector3::assert_near(&surface.uder(u, v), &uder);
            let pt = pt0 * (1.0 - u) + pt1 * u;
            let vec = Vector3::new(pt[2], 0.0, -pt[0]);
            let vder = Matrix3::from_axis_angle(Vector3::unit_y(), Rad(v)) * vec;
            Vector3::assert_near(&surface.vder(u, v), &vder);
            let n = surface.normal(u, v);
            assert!(n.dot(uder).so_small2());
            assert!(n.dot(vder).so_small2());
        }
    }
}
