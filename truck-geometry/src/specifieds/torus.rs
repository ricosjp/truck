use super::*;
use std::f64::consts::PI;

impl Torus {
    /// constructor
    #[inline(always)]
    pub fn new(center: Point3, large_radius: f64, small_radius: f64) -> Self {
        if large_radius <= 0.0 || small_radius <= 0.0 {
            panic!("radius must be larger than 0");
        }
        Self {
            center,
            large_radius,
            small_radius,
        }
    }

    /// get center
    #[inline(always)]
    pub const fn center(&self) -> Point3 { self.center }

    /// get large radius
    #[inline(always)]
    pub const fn large_radius(&self) -> f64 { self.large_radius }

    /// get small radius
    #[inline(always)]
    pub const fn small_radius(&self) -> f64 { self.small_radius }
}

impl ParametricSurface for Torus {
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Self::Vector {
        let ((su, cu), (sv, cv)) = (u.sin_cos(), v.sin_cos());
        let center = match (m, n) {
            (0, 0) => self.center.to_vec(),
            _ => Vector3::zero(),
        };
        let u_z = if m == 0 { 1.0 } else { 0.0 };
        let u_part = match m % 4 {
            0 => Vector3::new(cu, su, u_z),
            1 => Vector3::new(-su, cu, 0.0),
            2 => Vector3::new(-cu, -su, 0.0),
            _ => Vector3::new(su, -cu, 0.0),
        };
        let r0 = if n == 0 { self.large_radius } else { 0.0 };
        let r1 = self.small_radius;
        let v_part_d2 = match n % 4 {
            0 => Vector2::new(r0 + r1 * cv, r1 * sv),
            1 => Vector2::new(-r1 * sv, r1 * cv),
            2 => Vector2::new(-r1 * cv, -r1 * sv),
            _ => Vector2::new(r1 * sv, -r1 * cv),
        };
        let v_part = Vector3::new(v_part_d2.x, v_part_d2.x, v_part_d2.y);
        center + u_part.mul_element_wise(v_part)
    }
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 {
        let sr = self.small_radius() * Vector2::new(f64::cos(v), f64::sin(v));
        let lr = (self.large_radius() + sr.x) * Vector2::new(f64::cos(u), f64::sin(u));
        self.center() + Vector3::new(lr.x, lr.y, sr.y)
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        let sr = self.small_radius() * f64::cos(v);
        let lr = (self.large_radius() + sr) * Vector2::new(f64::cos(u), f64::sin(u));
        Vector3::new(-lr.y, lr.x, 0.0)
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        let sv = self.small_radius() * Vector2::new(-f64::sin(v), f64::cos(v));
        Vector3::new(sv.x * f64::cos(u), sv.x * f64::sin(u), sv.y)
    }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Vector3 {
        let sr = self.small_radius() * f64::cos(v);
        let lr = (self.large_radius() + sr) * Vector2::new(f64::cos(u), f64::sin(u));
        Vector3::new(-lr.x, -lr.y, 0.0)
    }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Vector3 {
        let sr = -self.small_radius() * f64::sin(v);
        let lr = sr * Vector2::new(f64::cos(u), f64::sin(u));
        Vector3::new(-lr.y, lr.x, 0.0)
    }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Vector3 {
        let sv = -self.small_radius() * Vector2::new(f64::cos(v), f64::sin(v));
        Vector3::new(sv.x * f64::cos(u), sv.x * f64::sin(u), sv.y)
    }
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        const RANGE: (Bound<f64>, Bound<f64>) = (Bound::Included(0.0), Bound::Excluded(2.0 * PI));
        (RANGE, RANGE)
    }
    #[inline(always)]
    fn u_period(&self) -> Option<f64> { Some(2.0 * PI) }
    #[inline(always)]
    fn v_period(&self) -> Option<f64> { Some(2.0 * PI) }
}

impl ParametricSurface3D for Torus {
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        let sv = Vector2::new(f64::cos(v), f64::sin(v));
        Vector3::new(sv.x * f64::cos(u), sv.x * f64::sin(u), sv.y)
    }
    #[inline(always)]
    fn normal_uder(&self, u: f64, v: f64) -> Vector3 {
        let sv = Vector2::new(f64::cos(v), f64::sin(v));
        Vector3::new(-sv.x * f64::sin(u), sv.x * f64::cos(u), sv.y)
    }
    #[inline(always)]
    fn normal_vder(&self, u: f64, v: f64) -> Vector3 {
        let sv = Vector2::new(-f64::sin(v), f64::cos(v));
        Vector3::new(sv.x * f64::cos(u), sv.x * f64::sin(u), sv.y)
    }
}

impl BoundedSurface for Torus {}

impl SearchParameter<D2> for Torus {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        _: H,
        _: usize,
    ) -> Option<(f64, f64)> {
        let r = point - self.center();
        let rxy = Vector2::new(r.x, r.y);
        let v = f64::asin(f64::clamp(r.z / self.small_radius(), -1.0, 1.0));
        let minus = rxy.magnitude2() < self.large_radius() * self.large_radius();
        let v = match (minus, v < 0.0) {
            (true, _) => PI - v,
            (false, false) => v,
            (false, true) => 2.0 * PI + v,
        };
        let rxy_n = rxy.normalize();
        let u = f64::acos(f64::clamp(rxy_n.x, -1.0, 1.0));
        let u = match rxy_n.y < 0.0 {
            true => 2.0 * PI - u,
            false => u,
        };
        match self.subs(u, v).near(&point) {
            true => Some((u, v)),
            false => None,
        }
    }
}

impl SearchNearestParameter<D2> for Torus {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        _: H,
        _: usize,
    ) -> Option<(f64, f64)> {
        let r = point - self.center();
        let rxy = Vector2::new(r.x, r.y);
        if rxy.so_small() {
            return None;
        }
        let rxy_n = rxy.normalize();
        let large_r = self.large_radius() * rxy_n.extend(0.0);
        let diff = r - large_r;
        if diff.so_small() {
            return None;
        }
        let small_r = diff.normalize();

        let u = f64::acos(f64::clamp(rxy_n.x, -1.0, 1.0));
        let u = match rxy_n.y < 0.0 {
            true => 2.0 * PI - u,
            false => u,
        };
        let v = f64::asin(f64::clamp(small_r.z, -1.0, 1.0));
        let v = match (small_r.dot(large_r) < 0.0, v < 0.0) {
            (true, _) => PI - v,
            (false, false) => v,
            (false, true) => 2.0 * PI + v,
        };
        Some((u, v))
    }
}

impl ParameterDivision2D for Torus {
    fn parameter_division(
        &self,
        (urange, vrange): ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let circle = UnitCircle::<Point2>::new();
        let utol = tol / (self.small_radius() + self.large_radius());
        let (udiv, _) = circle.parameter_division(urange, utol);
        let vtol = tol / self.small_radius();
        let (vdiv, _) = circle.parameter_division(vrange, vtol);
        (udiv, vdiv)
    }
}
